import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
export const root = fileURLToPath(new URL('../',import.meta.url));
export function specHash() {
  const scopeBytes=readFileSync(path.join(root,'docs/reviews/scope.json'));
  const scope=JSON.parse(scopeBytes);
  const hash=createHash('sha256');
  hash.update('docs/reviews/scope.json\0');hash.update(scopeBytes);hash.update('\0');
  for(const name of [...scope.files].sort()) {
    const full=path.resolve(root,name);
    if(!full.startsWith(root)||name.includes('..'))throw new Error('Invalid review scope path');
    hash.update(name+'\0');hash.update(readFileSync(full));hash.update('\0');
  }
  return hash.digest('hex');
}
if(process.argv[1] && path.resolve(process.argv[1])===fileURLToPath(import.meta.url))console.log(specHash());
