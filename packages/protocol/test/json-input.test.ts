import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { JsonInputError, parseJsonInput } from '../src/index';

interface Vector {
  id: string;
  json: string;
  expected: 'valid' | JsonInputError['kind'];
}

const vectors = JSON.parse(readFileSync(new URL('../../../contracts/fixtures/command-json-inputs.v1.json', import.meta.url), 'utf8')) as Vector[];
const limits = { maxBytes: 16_384, maxDepth: 128 };

test('parses shared protocol JSON input vectors consistently', () => {
  for (const vector of vectors) {
    try {
      parseJsonInput(vector.json, limits);
      assert.equal(vector.expected, 'valid', `${vector.id} should be rejected`);
    } catch (error) {
      if (!(error instanceof JsonInputError)) throw error;
      assert.equal(error.kind, vector.expected, vector.id);
    }
  }
});

test('enforces the caller-provided input byte and nesting limits', () => {
  assert.throws(
    () => parseJsonInput('[0,0,0]', { maxBytes: 6, maxDepth: 128 }),
    (error: unknown) => error instanceof JsonInputError && error.kind === 'input_too_large'
  );
  assert.throws(
    () => parseJsonInput('[[[0]]]', { maxBytes: 16_384, maxDepth: 2 }),
    (error: unknown) => error instanceof JsonInputError && error.kind === 'nesting_limit'
  );
  const nestedAtLimit = `${'['.repeat(128)}0${']'.repeat(128)}`;
  assert.doesNotThrow(() => parseJsonInput(nestedAtLimit, { maxBytes: 1024, maxDepth: 128 }));
  assert.throws(
    () => parseJsonInput('0', { maxBytes: 1, maxDepth: 129 }),
    (error: unknown) => error instanceof JsonInputError && error.kind === 'invalid_limits'
  );
  const unicode = '{"é":"😀"}';
  assert.throws(
    () => parseJsonInput(unicode, { maxBytes: new TextEncoder().encode(unicode).byteLength - 1, maxDepth: 128 }),
    (error: unknown) => error instanceof JsonInputError && error.kind === 'input_too_large'
  );
});

test('preserves JavaScript number semantics for integer-valued decimal tokens', () => {
  assert.deepEqual(parseJsonInput('{"integer":1.0,"zero":0.0}', limits), { integer: 1, zero: 0 });
});
