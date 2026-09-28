import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { parseHealth, scaffoldHealth } from '../src/index';
const fixture = (name: string): unknown => JSON.parse(readFileSync(new URL(`../../../contracts/fixtures/${name}`, import.meta.url), 'utf8'));
test('accepts the cross-language valid fixture', () => assert.deepEqual(parseHealth(fixture('health.valid.json')), scaffoldHealth('host')));
test('rejects a false execution claim', () => assert.throws(() => parseHealth(fixture('health.invalid.json'))));
test('rejects protocol drift and unknown properties', () => {
  assert.throws(() => parseHealth({...scaffoldHealth('host'), schema_version: 2}));
  assert.throws(() => parseHealth({...scaffoldHealth('host'), token: 'should-not-exist'}));
});
