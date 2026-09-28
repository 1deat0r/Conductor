import test from 'node:test';
import assert from 'node:assert/strict';
import { createApp } from '../src/app';
import { parseHealth } from '@conductor/protocol';
test('health discloses scaffold capabilities', async () => { const app = createApp(); try { const r = await app.inject({method:'GET',url:'/healthz'}); assert.equal(r.statusCode,200); assert.equal(parseHealth(r.json()).execution_available,false); } finally { await app.close(); } });
test('commands never claim acceptance or execution', async () => { const app = createApp(); try { const r = await app.inject({method:'POST',url:'/v1/commands',payload:{command_id:'example'}}); assert.equal(r.statusCode,501); assert.equal(r.json().error,'not_implemented'); } finally { await app.close(); } });
