import test from 'node:test';
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';
import canonicalize from 'canonicalize';
import {
  JsonInputError,
  type JsonInputLimits,
  canonicalizeJsonText,
  computeCommandV1DigestFromJson,
  computeSubmissionFingerprintV1Json,
  deadlineRejectionAt,
  parseCommandActionV1Json,
  parseCommandEventV1Json,
  parseCommandSubmissionV1Json,
  parseCommandV1Json,
  parseCoordinatorReceiptV1Json,
  parseHostReceiptV1Json,
} from '../src/index';

const fixtureRoot = new URL('../../../contracts/fixtures/commands/', import.meta.url);
const rawFixture = (name: string): string => readFileSync(new URL(name, fixtureRoot), 'utf8');
const jsonFixture = <T>(name: string): T => JSON.parse(rawFixture(name)) as T;
const JSON_LIMITS: JsonInputLimits = { maxBytes: 16_384, maxDepth: 128 };

function fixtureCommandDigest(input: string): string {
  const { payload_digest: _digest, ...withoutDigest } = JSON.parse(input) as Record<string, unknown> & { payload_digest: string };
  const canonical = canonicalize(withoutDigest);
  assert.equal(typeof canonical, 'string');
  return createHash('sha256')
    .update(Buffer.concat([Buffer.from('Conductor.CommandV1\0'), Buffer.from(canonical!)]))
    .digest('hex');
}

test('validates all operation actions and submission shapes across the shared fixtures', () => {
  const submissions = [
    'submission.valid.json',
    'submission.task-update.valid.json',
    'submission.attempt-start.valid.json',
    'submission.attempt-cancel.valid.json',
  ];
  for (const name of submissions) {
    const submission = parseCommandSubmissionV1Json(rawFixture(name), JSON_LIMITS);
    assert.doesNotThrow(() => parseCommandActionV1Json(JSON.stringify(submission.action), JSON_LIMITS), name);
  }
  for (const name of [
    'submission.invalid.json',
    'submission.task-create-targeted.invalid.json',
    'submission.attempt-start-claimed-attempt.invalid.json',
    'submission.bad-revision-target.invalid.json',
    'submission.attempt-cancel-bad-revision.invalid.json',
  ]) {
    assert.throws(() => parseCommandSubmissionV1Json(rawFixture(name), JSON_LIMITS), name);
  }
  assert.throws(() => parseCommandActionV1Json(rawFixture('action.invalid.json'), JSON_LIMITS));
});

test('checks every canonical operation, digest omission, and target/revision binding', () => {
  const validCommands = [
    'command.valid.json',
    'command.task-create.valid.json',
    'command.task-update.valid.json',
    'command.attempt-cancel.valid.json',
    'command.integer-float.valid.json',
  ];
  for (const name of validCommands) {
    const raw = rawFixture(name);
    const parsed = parseCommandV1Json(raw, JSON_LIMITS);
    assert.equal(computeCommandV1DigestFromJson(raw, JSON_LIMITS), parsed.payload_digest, name);
  }
  const validValue = JSON.parse(rawFixture('command.valid.json')) as Record<string, unknown> & { payload_digest: string };
  const originalDigest = validValue.payload_digest;
  validValue.payload_digest = 'f'.repeat(64);
  const digestFieldChanged = JSON.stringify(validValue);
  assert.equal(computeCommandV1DigestFromJson(digestFieldChanged, JSON_LIMITS), originalDigest);
  assert.throws(() => parseCommandV1Json(digestFieldChanged, JSON_LIMITS), /digest mismatch/);
  assert.throws(() => parseCommandV1Json(rawFixture('command.invalid.json'), JSON_LIMITS), /digest mismatch/);

  const wrongRevision = rawFixture('command.bad-revision-target.invalid.json');
  const wrongRevisionValue = JSON.parse(wrongRevision) as Record<string, unknown>;
  assert.equal(fixtureCommandDigest(wrongRevision), wrongRevisionValue.payload_digest);
  assert.throws(() => parseCommandV1Json(wrongRevision, JSON_LIMITS), /Revision target does not match/);

  const parsed = parseCommandV1Json(rawFixture('command.valid.json'), JSON_LIMITS);
  assert.throws(() => {
    (parsed as unknown as { target: { task_id: string } }).target.task_id = 'tampered-task';
  });
  assert.equal(parsed.target.task_id, 'task-7');
});

test('rejects duplicate JSON property names before parsing, including escaped-equivalent names', () => {
  for (const name of ['duplicate-key.raw.json', 'jcs-duplicate-escaped-key.raw.json']) {
    assert.throws(
      () => canonicalizeJsonText(rawFixture(name), JSON_LIMITS),
      (error: unknown) => error instanceof JsonInputError && error.kind === 'duplicate_property',
      name,
    );
  }
});

test('rejects every negative-zero spelling before canonicalization', () => {
  const vectors = JSON.parse(readFileSync(new URL('../../../contracts/fixtures/command-json-inputs.v1.json', import.meta.url), 'utf8')) as Array<{
    id: string;
    json: string;
    expected: string;
  }>;
  for (const vector of vectors.filter((item) => item.expected === 'negative_zero')) {
    assert.throws(
      () => canonicalizeJsonText(vector.json, JSON_LIMITS),
      (error: unknown) => error instanceof JsonInputError && error.kind === 'negative_zero',
      vector.id,
    );
  }
});

test('matches the shared byte-for-byte RFC 8785 canonicalization vectors', () => {
  const vectors = jsonFixture<Array<{ name: string; input_json: string; canonical_json: string }>>('jcs-vectors.json');
  for (const vector of vectors) {
    assert.equal(canonicalizeJsonText(vector.input_json, JSON_LIMITS), vector.canonical_json, vector.name);
  }
  for (const name of ['jcs-fractional-number.raw.json', 'jcs-unsafe-integer.raw.json', 'jcs-i64-min.raw.json']) {
    assert.throws(() => canonicalizeJsonText(rawFixture(name), JSON_LIMITS), name);
  }
  assert.throws(() => parseCommandEventV1Json(rawFixture('event.unsafe-integer.invalid.json'), JSON_LIMITS));
  assert.throws(() => parseCommandEventV1Json(rawFixture('event.fractional-number.invalid.json'), JSON_LIMITS));
});

test('uses authenticated context for create retries and retains the original assigned ID', () => {
  const replay = jsonFixture<{
    principal: { tenant_id: string; actor_id: string; device_id: string };
    submission: object;
    same_retry: object;
    changed_retry: object;
    changed_principal: { tenant_id: string; actor_id: string; device_id: string };
    original_receipt: object;
    expected_submission_fingerprint: string;
    expected_changed_content_fingerprint: string;
    expected_changed_principal_fingerprint: string;
    expected_assigned_task_id: string;
  }>('create-replay.json');
  const fingerprint = computeSubmissionFingerprintV1Json(JSON.stringify(replay.submission), replay.principal, JSON_LIMITS);
  assert.equal(fingerprint, replay.expected_submission_fingerprint);
  assert.equal(computeSubmissionFingerprintV1Json(JSON.stringify(replay.same_retry), replay.principal, JSON_LIMITS), fingerprint);
  const reorderedSubmission = {
    action: { goal: 'Prove local command recovery', title: 'S1 proof', kind: 'task.create' },
    deadline_unix_ms: 2000,
    command_id: 'cmd-create-01',
    schema_version: 1,
  };
  assert.equal(computeSubmissionFingerprintV1Json(JSON.stringify(reorderedSubmission), replay.principal, JSON_LIMITS), fingerprint);
  assert.equal(computeSubmissionFingerprintV1Json(JSON.stringify(replay.changed_retry), replay.principal, JSON_LIMITS), replay.expected_changed_content_fingerprint);
  assert.notEqual(replay.expected_changed_content_fingerprint, fingerprint);
  assert.equal(computeSubmissionFingerprintV1Json(JSON.stringify(replay.submission), replay.changed_principal, JSON_LIMITS), replay.expected_changed_principal_fingerprint);
  assert.notEqual(replay.expected_changed_principal_fingerprint, fingerprint);
  const receipt = parseCoordinatorReceiptV1Json(JSON.stringify(replay.original_receipt), JSON_LIMITS);
  const canonicalCommand = parseCommandV1Json(rawFixture('command.task-create.valid.json'), JSON_LIMITS);
  assert.equal(canonicalCommand.submission_fingerprint, fingerprint);
  assert.equal(receipt.payload_digest, canonicalCommand.payload_digest);
  assert.equal(receipt.command_id, (replay.submission as { command_id: string }).command_id);
  assert.equal(receipt.submission_fingerprint, fingerprint);
  assert.notEqual(receipt.submission_fingerprint, replay.expected_changed_content_fingerprint);
  assert.notEqual(receipt.submission_fingerprint, replay.expected_changed_principal_fingerprint);
  assert.equal(receipt.assigned_target.task_id, replay.expected_assigned_task_id);
  assert.throws(() => parseCoordinatorReceiptV1Json(rawFixture('coordinator-receipt.invalid.json'), JSON_LIMITS));
  const withAttemptState = { ...replay.original_receipt, attempt_state: 'completed' };
  assert.throws(() => parseCoordinatorReceiptV1Json(JSON.stringify(withAttemptState), JSON_LIMITS));
});

test('classifies the fixed-clock expiry boundary', () => {
  const vectors = jsonFixture<Array<{ deadline_unix_ms: number; now_unix_ms: number; expected_rejection: 'expired' | null }>>('deadline-vectors.json');
  for (const vector of vectors) {
    assert.equal(deadlineRejectionAt(vector.deadline_unix_ms, vector.now_unix_ms), vector.expected_rejection ?? undefined);
  }
  assert.throws(() => deadlineRejectionAt(-1, 0));
});

test('validates all authority-scoped host receipt states and rejects mixed state fields', () => {
  for (const name of [
    'host-receipt.pending.valid.json',
    'host-receipt.accepted.valid.json',
    'host-receipt.valid.json',
    'host-receipt.stale-revision.valid.json',
    'host-receipt.permission-changed.valid.json',
    'host-receipt.resolved.valid.json',
    'host-receipt.resolved-no-result.valid.json',
    'host-receipt.unknown.valid.json',
  ]) {
    assert.doesNotThrow(() => parseHostReceiptV1Json(rawFixture(name), JSON_LIMITS), name);
  }
  assert.throws(() => parseHostReceiptV1Json(rawFixture('host-receipt.invalid.json'), JSON_LIMITS));
  const accepted = JSON.parse(rawFixture('host-receipt.accepted.valid.json')) as Record<string, unknown>;
  accepted.attempt_state = 'running';
  assert.throws(() => parseHostReceiptV1Json(JSON.stringify(accepted), JSON_LIMITS));
});

test('types known event payloads and requires snapshot recovery for unknown informational events', () => {
  const attemptEvent = parseCommandEventV1Json(rawFixture('event.valid.json'), JSON_LIMITS);
  assert.equal(attemptEvent.kind, 'known');
  if (attemptEvent.kind === 'known' && attemptEvent.event.event_type === 'attempt.state_changed') {
    assert.equal(attemptEvent.event.payload.to, 'running');
  } else {
    assert.fail('Expected typed attempt state event');
  }
  for (const name of ['event.task-created.valid.json', 'event.task-updated.valid.json', 'event.integer-float.valid.json']) {
    assert.equal(parseCommandEventV1Json(rawFixture(name), JSON_LIMITS).kind, 'known', name);
  }
  for (const name of ['event.unknown-informational.valid.json', 'event.constructor-informational.valid.json']) {
    const parsed = parseCommandEventV1Json(rawFixture(name), JSON_LIMITS);
    assert.equal(parsed.kind, 'unknown_informational', name);
    if (parsed.kind === 'unknown_informational') assert.equal(parsed.refreshSnapshotBeforeAdvance, true);
  }
  for (const name of ['event.invalid.json', 'event.unknown-no-cursor.invalid.json', 'event.unknown-required.invalid.json']) {
    assert.throws(() => parseCommandEventV1Json(rawFixture(name), JSON_LIMITS), name);
  }
});
