import { readFileSync } from 'node:fs';
import path from 'node:path';
import assert from 'node:assert/strict';
import { root,specHash } from './spec-hash.mjs';
const spec=readFileSync(path.join(root,'SPEC.md'),'utf8');
const plan=readFileSync(path.join(root,'docs/implementation-plan.md'),'utf8');
for(let i=1;i<=25;i++){
  const id='R'+String(i).padStart(2,'0');
  assert.equal(spec.split('**'+id+' —').length-1,1,`Expected one definition for ${id}`);
}
assert(plan.includes('R25')&&plan.includes('S0')&&plan.includes('S3'));
console.log(JSON.stringify({spec_structure:'passed',requirements:25,sha256:specHash()}));
