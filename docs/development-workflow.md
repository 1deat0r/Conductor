# Mandatory GitHub development workflow

Effective 28 September 2026. This is a normative project rule, requested by the owner, and applies to every contributor and agent. GitHub is the authoritative collaboration and integration record; local worktrees are execution environments. Do not keep completed development only in a chat or an unpushed local branch.

## Work and agent ownership

1. Start from a GitHub issue with problem, scope, SPEC requirement IDs, acceptance criteria, platform impact and an accountable integrator. Read the issue and related PRs before changing code. Use milestones for S1/S2/S3 and labels for area/risk/platform; do not invent a delivery commitment.
2. Use a short-lived issue-linked feature branch from current protected main. Give concurrent implementers separate worktrees/branches and explicit file ownership; do not have parallel agents race in one mutable checkout. Coordinate interface changes before implementation. The integrator owns combining changes and resolving conflicts; subagents do not independently merge to main.
3. Make focused conventional commits, preserve user work, and open a draft PR early. Link the issue and requirements; record behavior, design decisions, test evidence, limits, migrations and rollback/recovery impact. A PR is the durable handoff record. Never publish credentials, private source from another project or provider transcripts.
4. Run relevant local checks, then required GitHub Actions on the proposed merge state. Add failure-oriented tests for authority, concurrency, recovery and security changes. Do not equate compilation with device, isolation or production qualification. Relevant five-platform gates may not be waived to make CI green.

## Independent expert review

Every PR receives independent parallel reviews from systems/runtime, security/trust and client/platform experts. Each reviewer reads the actual diff and relevant surrounding code/spec, identifies their role and agent/session, and records findings, evidence, exact reviewed head commit SHA and APPROVE or REQUEST_CHANGES in the PR. An author may answer findings but may not author, edit or relabel an independent verdict. Experts may conclude their area is unaffected only after inspecting the diff. Do not request a predetermined approval or use majority voting: every role must approve and all blockers must be resolved.

New commits invalidate approvals for the old head. Rebase/base changes require the reviewers to assess the new merge result and CI to rerun; preserve prior rounds. Store substantial reports under docs/reviews or link retained artifacts from the PR, and always put the final reviewed head SHA in the PR review record. For normative changes, also refresh the three file-based SPEC approvals at the exact scope hash. A green spec approval check on an unchanged spec does not review new implementation code.

Agent independence is independent analysis, not a claim of separate GitHub accounts or infallibility. Agents sharing one user's GitHub credentials do not count as separate GitHub approving principals and must never forge native GitHub approvals. In addition to the three expert records, protected main requires one eligible non-author GitHub approval and approval of the latest reviewable push. If no eligible reviewer/integration is available, leave the PR unmerged; do not weaken protection. This is an integration gate, not a request for approval before ordinary local work.

## GitHub enforcement

After the one-time approved bootstrap, main requires a pull request, all required status checks on an up-to-date branch, stale-review dismissal, resolved review conversations, at least one eligible approving review and approval of the latest reviewable push. Disable force push, deletion and merge commits; use squash merges. No normal bypass actors, including the owner or automation. CODEOWNERS identifies the accountable maintainer for sensitive paths; specialist agent roles are process roles, not fictitious GitHub teams.

Initial required checks are `source (ubuntu-latest)`, `source (windows-latest)` and `source (macos-latest)` from the GitHub Actions app, each including exact spec approval validation, TypeScript checks/tests/build and Rust checks. Linux additionally exports both mobile bundles and runs the desktop smoke. Keep check names stable. Native mobile and signed/installed desktop gates are added as the product becomes executable; source jobs cannot substitute for them.

Branch rules enforce PR/CI/native review mechanics. They cannot prove that an agent was independent or that a prose review was competent. The integrator and eligible GitHub reviewer must inspect the three exact-head records. A separate authenticated review-attestation service can automate this later, but no unsigned JSON field is represented as that trust boundary. Administrators can change repository settings; strict policy forbids doing so to route around a failing gate. Emergency security changes use an expedited issue/PR and the same review/check gates; any genuinely necessary policy change requires explicit owner authorization and a public rationale, never an agent's unilateral bypass.

Actions use pinned commit SHAs, minimal permissions and frozen dependency locks. Untrusted PR jobs receive no release secrets; never combine pull_request_target authority with execution of PR code. Signing/publication is a separate protected workflow. Dependency/action updates arrive through reviewed PRs. Do not install apps, invite people, purchase capacity, publish releases or submit stores merely because an issue exists; those actions require the applicable authority.

## Bootstrap exception and completion

The only direct-main bootstrap is the initial new repository seed, after all three experts approve the full final spec including this workflow. Push that approved scaffold, immediately apply/read back main protections, and inspect the first CI run. This exception expires once the initial seed exists. Fixes after the seed use ordinary PRs; do not use bootstrap wording for later direct commits.

A task is complete when the linked PR has all exact-head reviews and required checks, has actually merged through the allowed path, and its issue/evidence accurately reflect the result. If not merged, say so. Releases have separate installed-artifact, security, signing and store gates. Keep SPEC, ADRs, threat model and acceptance evidence in the same PR as changes that invalidate them. Reassess versions/support assumptions during 2027 through explicit ADRs and measured evidence, not speculative vendor promises.

References checked on 2026-09-28: [GitHub rules](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-rulesets/available-rules-for-rulesets), [rules REST API](https://docs.github.com/en/rest/repos/rules), [protected branches](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-protected-branches/about-protected-branches).
