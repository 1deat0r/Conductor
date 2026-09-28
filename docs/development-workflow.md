# Local-first development workflow

Conductor is primarily developed by autonomous coding agents. The fast path is local and commit-centered:

    task/context → inspect repository → implement → pnpm verify → review diff → atomic commit

GitHub provides source backup, synchronization, useful long-lived tracking, remote clean-environment checks and release infrastructure. It is not the default inner loop.

## Default loop

1. Inspect the current checkout and git status; preserve user and agent changes. Read the relevant product requirements, contracts, code and ADRs for the task.
2. Implement in the current checkout. Use a short-lived branch/worktree when work must be isolated, when agents need separate ownership, or when preparing an integration PR. Do not create a branch solely for ceremony.
3. Run pnpm verify from the repository root before committing. Add targeted checks during development and platform qualification when relevant. If a required check cannot run, record the exact command and limitation; do not claim a passing result.
4. Inspect the complete diff for scope, accidental files, secrets, generated artifacts and debug code. Make a small, coherent commit that leaves the repository in a valid state. Avoid unrelated edits and avoid rewriting other work.
5. Continue with the next task from the verified local state. Use a durable GitHub Issue only when ongoing backlog, cross-session context, dependencies, external reports or coordination make it valuable. Local task/context files are fine when they are more efficient.

Do not add the entire test suite to Git hooks. Keep hooks, if introduced, fast. pnpm verify is the primary quality gate for agents and contributors.

## Verification

pnpm verify is the one canonical local verification command. It runs spec-structure validation, TypeScript type checks, package tests and builds, Rust formatting, Clippy with warnings denied, and Rust workspace tests using the committed lockfiles. It is intended to be deterministic and local after dependencies are installed with pnpm install --frozen-lockfile and the pinned Rust toolchain.

The repo currently has no separate expensive FULL suite. Use pnpm check:mobile, the mobile bundle export, or pnpm smoke:desktop when the affected surface needs those checks. Desktop smoke needs a display or Xvfb and must keep Electron's sandbox enabled. pnpm verify and source CI do not qualify native device behavior, process isolation, hosted recovery or production security; use the relevant implementation-plan gates and record limitations.

## Issues, branches, PRs and CI

These are tools to use when they materially improve coordination, review, auditability or release confidence:

- **Issues:** optional. Use for persistent backlog, multi-session work, dependencies, external reports, major features or coordination. Small work that can be implemented and verified promptly does not need one.
- **Branches/worktrees:** optional for local work. Use for concurrency, isolation, risky changes, independent review or a protected-branch PR. Work in separate checkouts when concurrent agents would otherwise share mutable files.
- **Pull requests:** optional by default. Use for meaningful remote review, substantial/risky changes, public contributions or whenever protected main requires them. A PR should explain behavior, risk, validation and limitations; do not open throwaway PRs just to make chat progress visible.
- **CI:** an independent safety net, not the primary development loop. Run local verification first. Keep clean-checkout, OS-specific, packaging, security and release checks that add coverage local runs cannot provide. Agents may continue independent work while remote CI runs; wait for results before integration only when those results gate it.

### Current protected-main rules

The live repository ruleset currently requires a PR to update main, an eligible non-author GitHub approval of the latest reviewable push, resolved review conversations, up-to-date required checks on Ubuntu, Windows and macOS, linear history and squash merge. Deletion, force push and bypass are disabled. These rules govern remote integration; they do not require an Issue or PR for every local task. Do not bypass or change live protections as part of routine work. If the owner explicitly changes them, update this paragraph and .github/main-ruleset.json to match the verified live configuration.

For code intended to reach protected main, open a PR when that is the legitimate integration path, satisfy the live ruleset, and report any unavailable eligible reviewer honestly. Never fabricate a human review or treat same-account agents as separate GitHub approvers. A local commit is valid progress but is not a remote merge.

## Review by risk

All substantive changes receive an author/agent diff review and the appropriate local verification. Add independent review when the change's risk warrants it; use a focused specialist, not a fixed three-role quota for every change. Require an independent, security-informed human review before enabling or releasing changes to authority, trust boundaries, credentials, privacy, execution isolation, command acknowledgement/recovery, platform lifecycle or release controls. If such review or required evidence is unavailable, keep the capability disabled or the release blocked.

AI review is useful for finding defects, but it is not independent human approval and cannot establish production security. Preserve reviewer findings as written; do not rewrite another reviewer's verdict. Do not use an AI review result as a substitute for a required GitHub approval.

## Product and release safety

SPEC.md and contracts/ define product behavior. Do not weaken authority, privacy, durable acknowledgement or execution isolation to make checks pass. Keep unsupported S0 execution unavailable until the relevant SPEC requirements and release gates pass. Do not equate source typechecking with five-platform qualification. CI actions remain pinned, permissions minimal, PR jobs receive no release secrets, and publication/signing/store submission stay behind their explicit release gates.

The initial three-role spec review and exact-scope approval records document the 2026-09-28 bootstrap. They are historical evidence, not a recurring check for routine development. Review policy and active GitHub integration gates are documented here and in the live ruleset.
