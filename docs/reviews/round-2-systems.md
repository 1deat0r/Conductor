# Round 2 — independent systems review

- Reviewer role: systems
- Reviewed scope hash: `bc423cc2df0c3cf2ee1c663d8a47bbfd9ade3374630d16ef085648f5b4b8c598`
- Verdict: **APPROVE**
- Blocking findings: none

I independently read the entire expanded scope in `docs/reviews/scope.json`, including R25, the development workflow, contributor rules, CI workflow, CODEOWNERS and main ruleset. I computed the scope hash locally rather than relying on the supplied digest. This approves readiness to implement the specification; it is not runtime, native-platform, security or production certification.

## Previous findings

**SYS-001 is resolved.** R15 explicitly separates trusted-host account authority from qualified restricted/managed isolation. Its UI/capability reporting requirement now states the limitations before enabling trusted execution. R09 conditions strong publication/failover guarantees on an enforced inaccessible credential/policy boundary, rather than voluntary broker use. R16 describes the selected execution profile instead of implying universal sandboxing. The threat model and milestone gates are consistent with those boundaries.

**SYS-N02 is resolved.** R02 now calls the deferred infrastructure a message broker/stream service, distinguishing it from the required credentialed effect broker.

**SYS-N01 remains a nonblocking implementation reminder.** R05/R06 require the future executable contracts and state-machine implementation to preserve the no-resurrection and uncertainty rules. Enumerating terminal states and authenticated reconciliation transitions in S1 conformance vectors remains appropriate; no new design decision is needed to begin that work.

## Systems and governance assessment

R04 and R07–R09 retain coherent authority and delivery semantics: one writer per aggregate, separate desired/observed state, transactional receipts/outboxes, distinct acknowledgement levels, ambiguous-launch reconciliation and no unsupported exactly-once guarantee. R11, R18, R22 and R24 retain the required partition, artifact-cleanup, budget-escrow and restore constraints. I found no new contradiction among those requirements or their release gates. R18's S2 artifact-authorization gate is now explicit.

R25 and the development workflow define a workable integration process with an explicit availability tradeoff:

- Separate worktrees/branches and an accountable integrator prevent parallel contributors from racing in one checkout.
- Each expert approval names the actual PR head; new commits invalidate earlier approvals, and changed bases require assessment of the new merge result and renewed CI. Normative approval by scope digest is explicitly separate from implementation review.
- Shared-account agents are not represented as distinct native GitHub reviewers. An eligible non-author GitHub approval remains mandatory; without one the PR stays unmerged. That is a deliberate gate, not an instruction to fabricate identity or bypass protection.
- The one-time seed is constrained to after unanimous approval of the full specification. Protection read-back and initial CI inspection follow it; subsequent fixes require PRs. This does not create an open-ended bootstrap bypass.
- The proposed active ruleset has no bypass actors, requires an up-to-date branch and named Actions checks, dismisses stale reviews, requires latest-push approval, resolves review threads and restricts merges to squash. CODEOWNERS is used for accountability, not claimed to implement three distinct expert principals.
- The workflow honestly assigns checking of the three expert records to the integrator and eligible reviewer. Neither unsigned review JSON nor a green unchanged-spec check is presented as proof that new code received independent review.

The GitHub rule semantics used here are consistent with the primary documentation on required review, latest-push approval, stale-review dismissal, strict required checks and source-app binding. This checks the proposed configuration semantics, not live repository enforcement. [GitHub ruleset documentation](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-rulesets/available-rules-for-rulesets), [protected branch documentation](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-protected-branches/about-protected-branches).

## Verification and limits

`pnpm verify` passed locally for this review: specification structure, TS checks/tests/build, Rust formatting, Clippy across all targets with warnings denied, and Rust tests. Turbo reused local JavaScript caches; Rust checks/tests executed. The pinned CI workflow and ruleset were inspected as configuration, not reported as a completed remote CI run or applied GitHub protection. Public creation/protection is still subsequent bootstrap work.

`pnpm check:approvals` was run after writing this record. It currently fails because the aggregate `docs/reviews/approvals.json` has not yet been created; this does not claim or supply the other reviewers' approvals. A final hash check confirmed that the reviewed scope was unchanged.

Future execution, concurrency, restore, isolation and native-device scenarios remain release gates. I have not treated the scaffold's passing checks as evidence for those unimplemented guarantees. My approval has no remaining blocking condition and applies only to the exact scope digest above.
