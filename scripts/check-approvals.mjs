import { readFileSync } from 'node:fs';
import path from 'node:path';
import assert from 'node:assert/strict';
import { root,specHash } from './spec-hash.mjs';
const manifest=JSON.parse(readFileSync(path.join(root,'docs/reviews/approvals.json'),'utf8'));
assert.equal(manifest.spec_sha256,specHash(),'Specification changed after review');
for(const role of ['systems','security','platforms']){
  const file=manifest.reviews[role];assert.equal(typeof file,'string',`Missing ${role} approval`);
  assert(file.startsWith('docs/reviews/')&&!file.includes('..'));
  const review=JSON.parse(readFileSync(path.join(root,file),'utf8'));
  assert.equal(review.role,role);assert.equal(review.spec_sha256,manifest.spec_sha256);
  assert.equal(review.verdict,'APPROVE',`${role} has not approved`);
  assert.deepEqual(review.blocking_findings,[],`${role} retains blocking findings`);
}
console.log(JSON.stringify({spec_approval:'passed',sha256:manifest.spec_sha256,roles:3}));
