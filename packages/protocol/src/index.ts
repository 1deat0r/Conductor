import Ajv from 'ajv';
import Ajv2020 from 'ajv/dist/2020';
import canonicalize from 'canonicalize';
import { sha256 } from '@noble/hashes/sha2.js';
import actionSchema from '../../../contracts/command-action-v1.schema.json';
import submissionSchema from '../../../contracts/command-submission-v1.schema.json';
import commandSchema from '../../../contracts/command-v1.schema.json';
import coordinatorReceiptSchema from '../../../contracts/coordinator-receipt-v1.schema.json';
import hostReceiptSchema from '../../../contracts/host-receipt-v1.schema.json';
import eventSchema from '../../../contracts/command-event-v1.schema.json';
import healthSchema from '../../../contracts/health-v1.schema.json';
import { JsonInputError, parseJsonInput, type JsonInputLimits } from './json-input';

export type Service = 'control-api' | 'host' | 'supervisor';
export interface HealthV1 {
  schema_version: 1;
  service: Service;
  status: 'scaffold';
  execution_available: false;
}

export interface TaskCreateActionV1 {
  kind: 'task.create';
  title: string;
  goal: string;
}
export interface TaskUpdateActionV1 {
  kind: 'task.update';
  changes: { title?: string; goal?: string };
}
export interface AttemptStartActionV1 {
  kind: 'attempt.start';
  launch_digest: string;
}
export interface AttemptCancelActionV1 {
  kind: 'attempt.cancel';
  reason?: string;
}
export type CommandActionV1 = TaskCreateActionV1 | TaskUpdateActionV1 | AttemptStartActionV1 | AttemptCancelActionV1;
export interface CommandTargetV1 {
  task_id: string;
  attempt_id?: string;
}
export interface RevisionTargetV1 {
  aggregate_kind: 'task' | 'attempt';
  aggregate_id: string;
  expected_revision: number;
}
interface CommandSubmissionBaseV1 {
  schema_version: 1;
  command_id: string;
  deadline_unix_ms: number;
}
type TaskRevisionV1 = { aggregate_kind: 'task'; aggregate_id: string; expected_revision: number };
type AttemptRevisionV1 = { aggregate_kind: 'attempt'; aggregate_id: string; expected_revision: number };
type TaskTargetV1 = { task_id: string; attempt_id?: never };
type AttemptTargetV1 = { task_id: string; attempt_id: string };
export type CommandSubmissionV1 = CommandSubmissionBaseV1 & (
  | { action: TaskCreateActionV1; target?: never; revision_target?: never }
  | { action: TaskUpdateActionV1; target: TaskTargetV1; revision_target: TaskRevisionV1 }
  | { action: AttemptStartActionV1; target: TaskTargetV1; revision_target: TaskRevisionV1 }
  | { action: AttemptCancelActionV1; target: AttemptTargetV1; revision_target: AttemptRevisionV1 }
);
interface CommandBaseV1 {
  schema_version: 1;
  command_id: string;
  payload_digest: string;
  submission_fingerprint: string;
  tenant_id: string;
  actor_id: string;
  device_id: string;
  deadline_unix_ms: number;
}
export type CommandV1 = CommandBaseV1 & (
  | { action: TaskCreateActionV1; target: TaskTargetV1; revision_target?: never }
  | { action: TaskUpdateActionV1; target: TaskTargetV1; revision_target: TaskRevisionV1 }
  | { action: AttemptStartActionV1; target: AttemptTargetV1; revision_target: TaskRevisionV1 }
  | { action: AttemptCancelActionV1; target: AttemptTargetV1; revision_target: AttemptRevisionV1 }
);
export type DeepReadonly<T> = T extends readonly (infer Item)[]
  ? readonly DeepReadonly<Item>[]
  : T extends object
    ? { readonly [Key in keyof T]: DeepReadonly<T[Key]> }
    : T;
export type ImmutableCommandV1 = DeepReadonly<CommandV1>;
export interface AuthenticatedPrincipalV1 {
  tenant_id: string;
  actor_id: string;
  device_id: string;
}
export interface CoordinatorReceiptV1 {
  schema_version: 1;
  authority: { kind: 'coordinator'; tenant_id: string };
  command_id: string;
  payload_digest: string;
  submission_fingerprint: string;
  receipt_state: 'coordinator_stored';
  stored_at_unix_ms: number;
  assigned_target: CommandTargetV1;
}
export type HostRejectionCodeV1 = 'stale_revision' | 'expired' | 'permission_changed';
export interface HostReceiptBaseV1 {
  schema_version: 1;
  authority: { kind: 'host'; host_id: string };
  command_id: string;
  payload_digest: string;
  updated_at_unix_ms: number;
}
export type HostReceiptV1 = HostReceiptBaseV1 & (
  | { receipt_state: 'pending_host_admission' }
  | { receipt_state: 'host_accepted' }
  | { receipt_state: 'host_rejected'; rejection: { code: HostRejectionCodeV1 } }
  | { receipt_state: 'effect_resolved'; resolution: HostResolutionV1 }
  | { receipt_state: 'outcome_unknown'; unknown_reason: 'ambiguous_history'; reconciliation_required: true }
);
export type HostResolutionV1 =
  | { resolution_id: string; outcome: 'succeeded' | 'failed' | 'cancelled'; result_digest: string }
  | { resolution_id: string; outcome: 'succeeded' | 'failed' | 'cancelled'; no_result: true };
interface CommandEventBaseV1 {
  schema_version: 1;
  event_id: string;
  command_id: string;
  causation_id: string | null;
  actor_id: string;
  ownership_generation: number;
  occurred_at_unix_ms: number;
}
export type AttemptStateV1 = 'queued' | 'provisioning' | 'running' | 'waiting_for_input' | 'waiting_for_approval' | 'validating' | 'review_ready' | 'completed' | 'failed' | 'cancelled' | 'interrupted' | 'outcome_unknown';
export type KnownCommandEventV1 =
  | (CommandEventBaseV1 & { event_type: 'task.created' | 'task.updated'; event_class: 'required'; aggregate: { kind: 'task'; id: string; sequence: number }; payload: { task_revision: number } })
  | (CommandEventBaseV1 & { event_type: 'attempt.state_changed'; event_class: 'required'; aggregate: { kind: 'attempt'; id: string; sequence: number }; payload: { from?: AttemptStateV1 | null; to: AttemptStateV1 } });
export interface UnknownInformationalEventV1 extends CommandEventBaseV1 {
  event_type: string;
  event_class: 'informational';
  aggregate: { kind: 'task' | 'attempt' | 'session'; id: string; sequence: number };
  snapshot_cursor: string;
  payload: Record<string, unknown>;
}
export type CommandEventV1 = KnownCommandEventV1 | UnknownInformationalEventV1;
export type ParsedCommandEventV1 =
  | { kind: 'known'; event: KnownCommandEventV1 }
  | { kind: 'unknown_informational'; event: UnknownInformationalEventV1; refreshSnapshotBeforeAdvance: true };

const ajv = new Ajv2020({ strict: true, strictRequired: false, allErrors: true });
const ajvDraft7 = new Ajv({ strict: true, strictRequired: false, allErrors: true });
const validateHealth = ajvDraft7.compile<HealthV1>(healthSchema);
const validateAction = ajv.compile(actionSchema);
const validateSubmission = ajv.compile(submissionSchema);
const validateCommand = ajv.compile(commandSchema);
const validateCoordinatorReceipt = ajv.compile(coordinatorReceiptSchema);
const validateHostReceipt = ajv.compile(hostReceiptSchema);
const validateEvent = ajv.compile(eventSchema);
const COMMAND_DIGEST_DOMAIN = 'Conductor.CommandV1\0';
const SUBMISSION_FINGERPRINT_DOMAIN = 'Conductor.CommandSubmissionV1\0';

export function parseHealth(value: unknown): HealthV1 {
  if (!validateHealth(value)) throw new Error('Invalid or unsupported health envelope');
  return value;
}
export function scaffoldHealth(service: Service): HealthV1 {
  return parseHealth({ schema_version: 1, service, status: 'scaffold', execution_available: false });
}

function assertWellFormedUnicode(value: string): void {
  for (let index = 0; index < value.length; index += 1) {
    const code = value.charCodeAt(index);
    if (code >= 0xd800 && code <= 0xdbff) {
      const next = value.charCodeAt(index + 1);
      if (!(next >= 0xdc00 && next <= 0xdfff)) throw new Error('Invalid Unicode surrogate pair');
      index += 1;
    } else if (code >= 0xdc00 && code <= 0xdfff) {
      throw new Error('Invalid Unicode surrogate pair');
    }
  }
}

function assertJsonNumbersAndStrings(value: unknown): void {
  if (typeof value === 'number') {
    if (Object.is(value, -0)) throw new JsonInputError('negative_zero');
    if (!Number.isSafeInteger(value)) throw new JsonInputError('unsafe_number');
    return;
  }
  if (typeof value === 'string') {
    assertWellFormedUnicode(value);
    return;
  }
  if (Array.isArray(value)) {
    for (const item of value) assertJsonNumbersAndStrings(item);
    return;
  }
  if (value !== null && typeof value === 'object') {
    for (const [key, item] of Object.entries(value)) {
      assertWellFormedUnicode(key);
      assertJsonNumbersAndStrings(item);
    }
  }
}

function utf8ByteLength(value: string): number {
  let length = 0;
  for (let index = 0; index < value.length; index += 1) {
    const code = value.charCodeAt(index);
    if (code <= 0x7f) length += 1;
    else if (code <= 0x7ff) length += 2;
    else if (code >= 0xd800 && code <= 0xdbff && value.charCodeAt(index + 1) >= 0xdc00 && value.charCodeAt(index + 1) <= 0xdfff) {
      length += 4;
      index += 1;
    } else length += 3;
  }
  return length;
}

function utf8Bytes(value: string): Uint8Array {
  const bytes = new Uint8Array(utf8ByteLength(value));
  let offset = 0;
  for (const character of value) {
    const code = character.codePointAt(0)!;
    if (code <= 0x7f) {
      bytes[offset++] = code;
    } else if (code <= 0x7ff) {
      bytes[offset++] = 0xc0 | (code >> 6);
      bytes[offset++] = 0x80 | (code & 0x3f);
    } else if (code <= 0xffff) {
      bytes[offset++] = 0xe0 | (code >> 12);
      bytes[offset++] = 0x80 | ((code >> 6) & 0x3f);
      bytes[offset++] = 0x80 | (code & 0x3f);
    } else {
      bytes[offset++] = 0xf0 | (code >> 18);
      bytes[offset++] = 0x80 | ((code >> 12) & 0x3f);
      bytes[offset++] = 0x80 | ((code >> 6) & 0x3f);
      bytes[offset++] = 0x80 | (code & 0x3f);
    }
  }
  return bytes;
}

function parseStrictJson(input: string, limits: JsonInputLimits): unknown {
  if (typeof input !== 'string') throw new Error('Protocol input must be raw JSON text');
  return parseJsonInput(input, limits);
}

function requireSchema<T>(value: unknown, validator: (input: unknown) => boolean, name: string): T {
  if (!validator(value)) throw new Error(`Invalid or unsupported ${name}`);
  return value as T;
}
function deepFreeze<T>(value: T): DeepReadonly<T> {
  if (value !== null && typeof value === 'object' && !Object.isFrozen(value)) {
    for (const item of Object.values(value as Record<string, unknown>)) deepFreeze(item);
    Object.freeze(value);
  }
  return value as DeepReadonly<T>;
}

function validateTargetSemantics(
  action: CommandActionV1,
  target: CommandTargetV1 | undefined,
  revisionTarget: RevisionTargetV1 | undefined,
  canonical: boolean,
): void {
  const requireRevision = (kind: 'task' | 'attempt', id: string): void => {
    if (!revisionTarget || revisionTarget.aggregate_kind !== kind || revisionTarget.aggregate_id !== id) {
      throw new Error('Revision target does not match the command target');
    }
  };
  switch (action.kind) {
    case 'task.create':
      if (!canonical && target !== undefined) throw new Error('Task creation cannot target an existing task');
      if (canonical && (!target || target.attempt_id !== undefined)) throw new Error('Canonical task creation requires its assigned task ID');
      if (revisionTarget !== undefined) throw new Error('Task creation cannot require an existing revision');
      return;
    case 'task.update': {
      if (!target || target.attempt_id !== undefined) throw new Error('Task update requires a task target only');
      requireRevision('task', target.task_id);
      return;
    }
    case 'attempt.start': {
      if (!target) throw new Error('Attempt start requires a task target');
      if (canonical && !target.attempt_id) throw new Error('Canonical attempt start requires its assigned attempt ID');
      if (!canonical && target.attempt_id !== undefined) throw new Error('Attempt start submission cannot claim its assigned attempt ID');
      requireRevision('task', target.task_id);
      return;
    }
    case 'attempt.cancel': {
      if (!target || !target.attempt_id) throw new Error('Attempt cancellation requires a task and attempt target');
      requireRevision('attempt', target.attempt_id);
      return;
    }
  }
}
export function parseCommandActionV1Json(input: string, limits: JsonInputLimits): CommandActionV1 {
  const value = parseStrictJson(input, limits);
  return requireSchema<CommandActionV1>(value, validateAction, 'CommandActionV1');
}
export function parseCommandSubmissionV1Json(input: string, limits: JsonInputLimits): CommandSubmissionV1 {
  const value = parseStrictJson(input, limits);
  const submission = requireSchema<CommandSubmissionV1>(value, validateSubmission, 'CommandSubmissionV1');
  validateTargetSemantics(submission.action, submission.target, submission.revision_target, false);
  return submission;
}
function parseCommandV1WithoutDigestCheck(input: string, limits: JsonInputLimits): CommandV1 {
  const value = parseStrictJson(input, limits);
  const command = requireSchema<CommandV1>(value, validateCommand, 'CommandV1');
  validateTargetSemantics(command.action, command.target, command.revision_target, true);
  return command;
}
function sha256Hex(bytes: Uint8Array): string {
  return Array.from(sha256(bytes), (byte) => byte.toString(16).padStart(2, '0')).join('');
}
function canonicalBytes(value: unknown): Uint8Array {
  assertJsonNumbersAndStrings(value);
  const canonical = canonicalize(value as Parameters<typeof canonicalize>[0]);
  if (typeof canonical !== 'string') throw new Error('JCS canonicalization failed');
  return utf8Bytes(canonical);
}
export function canonicalizeJsonText(input: string, limits: JsonInputLimits): string {
  const canonical = canonicalize(parseStrictJson(input, limits) as Parameters<typeof canonicalize>[0]);
  if (typeof canonical !== 'string') throw new Error('JCS canonicalization failed');
  return canonical;
}
function commandDigest(command: CommandV1): string {
  const { payload_digest: _excluded, ...withoutDigest } = command;
  const domain = utf8Bytes(COMMAND_DIGEST_DOMAIN);
  const canonical = canonicalBytes(withoutDigest);
  const preimage = new Uint8Array(domain.length + canonical.length);
  preimage.set(domain);
  preimage.set(canonical, domain.length);
  return sha256Hex(preimage);
}
export function computeCommandV1DigestFromJson(input: string, limits: JsonInputLimits): string {
  return commandDigest(parseCommandV1WithoutDigestCheck(input, limits));
}
export function parseCommandV1Json(input: string, limits: JsonInputLimits): ImmutableCommandV1 {
  const command = parseCommandV1WithoutDigestCheck(input, limits);
  if (commandDigest(command) !== command.payload_digest) throw new Error('CommandV1 digest mismatch');
  return deepFreeze(command);
}
export function computeSubmissionFingerprintV1Json(
  input: string,
  principal: AuthenticatedPrincipalV1,
  limits: JsonInputLimits,
): string {
  const submission = parseCommandSubmissionV1Json(input, limits);
  const value = {
    tenant_id: principal.tenant_id,
    actor_id: principal.actor_id,
    device_id: principal.device_id,
    submission,
  };
  for (const id of [principal.tenant_id, principal.actor_id, principal.device_id]) {
    if (typeof id !== 'string' || !/^[A-Za-z0-9][A-Za-z0-9._:-]{0,127}$/.test(id)) throw new Error('Invalid authenticated principal context');
  }
  const domain = utf8Bytes(SUBMISSION_FINGERPRINT_DOMAIN);
  const canonical = canonicalBytes(value);
  const preimage = new Uint8Array(domain.length + canonical.length);
  preimage.set(domain);
  preimage.set(canonical, domain.length);
  return sha256Hex(preimage);
}
export function deadlineRejectionAt(deadlineUnixMs: number, nowUnixMs: number): 'expired' | undefined {
  if (!Number.isSafeInteger(deadlineUnixMs) || !Number.isSafeInteger(nowUnixMs) || deadlineUnixMs < 0 || nowUnixMs < 0) throw new Error('Deadline values must be nonnegative safe integers');
  return deadlineUnixMs <= nowUnixMs ? 'expired' : undefined;
}
export function parseCoordinatorReceiptV1Json(input: string, limits: JsonInputLimits): CoordinatorReceiptV1 {
  return requireSchema<CoordinatorReceiptV1>(parseStrictJson(input, limits), validateCoordinatorReceipt, 'CoordinatorReceiptV1');
}
export function parseHostReceiptV1Json(input: string, limits: JsonInputLimits): HostReceiptV1 {
  return requireSchema<HostReceiptV1>(parseStrictJson(input, limits), validateHostReceipt, 'HostReceiptV1');
}

const KNOWN_EVENT_CLASSES = new Map<string, 'required'>([
  ['task.created', 'required'],
  ['task.updated', 'required'],
  ['attempt.state_changed', 'required'],
]);
export function parseCommandEventV1Json(input: string, limits: JsonInputLimits): ParsedCommandEventV1 {
  const event = requireSchema<CommandEventV1>(parseStrictJson(input, limits), validateEvent, 'CommandEventV1');
  const expectedClass = KNOWN_EVENT_CLASSES.get(event.event_type);
  if (expectedClass !== undefined) {
    if (event.event_class !== expectedClass) throw new Error('Known event has an unsupported event_class');
    return { kind: 'known', event: event as KnownCommandEventV1 };
  }
  if (event.event_class !== 'informational' || !event.snapshot_cursor) {
    throw new Error('Unknown events require informational classification and snapshot recovery');
  }
  return { kind: 'unknown_informational', event: event as UnknownInformationalEventV1, refreshSnapshotBeforeAdvance: true };
}

export { JsonInputError, parseJsonInput };
export type { JsonInputErrorKind, JsonInputLimits, JsonInputValue } from './json-input';
