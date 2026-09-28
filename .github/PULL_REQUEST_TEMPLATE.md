## Why use a PR for this change?

Choose the value it adds: protected-main integration, substantial/risky change, concurrent work, public contribution, or meaningful remote review.

## Change and impact

Describe the behavior or documentation change, relevant SPEC requirements, affected platforms, and any migration/recovery impact.

## Risk review

- Risk level and trust boundaries touched:
- Focused human/specialist review needed (if any):
- AI review or other supporting analysis (optional):

AI review is supplemental and is not human approval or security qualification. Never claim separate reviewer identities for agents sharing one account.

## Verification and limits

- [ ] pnpm verify passed, or the exact limitation is documented.
- [ ] Relevant platform-specific checks/evidence are recorded.
- [ ] Diff reviewed for accidental files, secrets and generated/debug artifacts.

Commands/results and any untested platforms or release gates:

## Integration

For changes targeting protected main, satisfy the live ruleset's required checks and history rules. PRs and human approvals are optional unless the live policy or a genuine product security/release gate requires them. Never bypass protections or fabricate review evidence. Record the final commit/head SHA when describing verification evidence.
