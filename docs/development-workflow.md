# Autonomous, local-first development

Conductor is developed by autonomous coding agents. For ordinary repository work, the agent owns the task from inspection through verified commit and synchronization; do not make the user supervise routine choices or repeat a skill's interactive checkpoints.

The user has delegated material product and engineering choices to Conductor's [independent expert board](agents/decision-board.md). Board decisions replace clarification/approval loops for those choices.

The normal loop is:

    task/context → inspect → implement → pnpm verify → review diff → atomic commit → synchronize

GitHub provides backup, synchronization, cross-platform clean-environment checks, useful long-lived tracking and release infrastructure. It is not the primary development loop.

## Autonomous execution

- Read git status first and preserve all existing user/agent work. Read the relevant SPEC sections, contracts, implementation gates and decisions; do not reread unrelated material.
- Resolve routine ambiguity with the safest reversible choice. Record a short assumption in the task notes or commit when useful, then continue. Send any new material product/engineering choice to the expert board; do not ask the user to break a tie, choose a design, approve a plan, or accept a risk. If the board's terminal rule defers a critical choice, keep only that capability disabled and continue unrelated safe work.
- Ask for no user input to settle a product or engineering decision. An external action requiring account-owner authority, production access, spending or publication remains outside the board's authority; prepare and verify locally, then leave only that specifically unauthorized action pending.
- Use Matt Pocock skills as tools, not as a reason to pause. Their default instructions to ask, wait for direction, get confirmation, or create Issues/PRs are overridden for Conductor by this policy. Infer ordinary decisions from the request and repository, perform all locally possible steps, and keep going. Do not invoke an interactive skill when its purpose is only to collect approval for work that can safely proceed.
- If a skill encounters a true human-only step (for example, unavailable credentials, an account-owner decision, an irreversible external effect, or required independent release review), isolate that step. Complete unrelated safe work, record the precise blocker, and leave only the affected action or capability pending.
- Make routine reversible repository changes, run commands, use focused local notes, commit, push and integrate without asking for another confirmation. Do not send third-party messages, create public announcements, provision paid resources, use production credentials, publish, deploy, submit to app stores, or delete user data unless the task explicitly authorizes that action.

## Implementation and verification

1. Work in the current checkout for ordinary changes. Use a branch/worktree when isolation, parallel ownership, risk, or remote check staging makes it useful; do not branch just for ceremony.
2. Implement the smallest coherent change that satisfies the task and the normative product contract. Do not invent success, reduce security, or make unsupported S0 execution available.
3. Run `pnpm verify` from the repository root before committing. Add targeted checks during development and relevant platform qualification when needed. If a required check cannot run, record the command and limitation; never claim it passed.
4. Review the complete diff for scope, accidental/generated files, secrets, debug output, and unrelated work. Fix issues attributable to the change, then create a small, descriptive atomic commit that leaves the repository valid.
5. Continue useful independent work while remote checks run. Do not wait idle for CI unless its result is needed for the next integration action.

`pnpm verify` is the one canonical local verification command. It runs spec-structure validation, TypeScript checks, package tests/builds, Rust formatting, Clippy with warnings denied, and Rust workspace tests using committed lockfiles. Use `pnpm check:mobile`, mobile bundle export, or `pnpm smoke:desktop` when the affected surface needs them. Desktop smoke requires a display or Xvfb and Electron sandbox support. Source checks do not qualify native device behavior, process isolation, hosted recovery, or production security.

## Issues, branches, pull requests and CI

- **Issues are optional.** Use one for durable backlog, multi-session context, dependencies, external reports, major features, or coordination. A task that can be completed and committed now needs no Issue.
- **Branches/worktrees are optional.** Use them for isolation, concurrent work, risk review, or staging an exact commit for remote checks. Do not create disposable branches only to satisfy convention.
- **Pull requests are optional.** Use a PR when remote review, public contribution, concurrent work, or a complicated integration materially helps. If used, the agent may prepare, update, and merge it once the applicable evidence and rules are satisfied; do not wait for routine user approval.
- **CI is an independent safety net.** Keep the clean-checkout Ubuntu, Windows and macOS verification jobs plus valuable Linux mobile and desktop checks. Local verification is the inner-loop gate; CI results are required before protected-main integration because the live ruleset requires the three OS checks.

### Main integration

The active repository ruleset requires successful Ubuntu, Windows and macOS checks, linear history, no branch deletion, and no non-fast-forward updates. It has no required PR, review, or bypass rule. Direct fast-forward pushes to `main` are allowed only when the exact commit has the required successful statuses. Check the live ruleset before changing this description.

For an ordinary change, commit locally after `pnpm verify`. If the exact commit already has required green statuses, fast-forward `main` and push it. Otherwise push the same commit on a temporary integration branch so CI can evaluate that SHA; continue independent work while it runs, then fast-forward that exact checked commit to `main` when green. If main advanced in the meantime, rebase or replay the change, rerun local verification, and obtain checks for the new commit. Never bypass the ruleset, force-push, delete protected refs, fabricate a status/review, or wait for a human PR approval that the live policy does not require.

The no-PR path is normal, not mandatory: choose a PR when it provides real review or coordination value. Required CI is retained because it adds clean-environment and cross-platform evidence that local checks cannot provide.

## Review, product security and release

Review every substantive diff and verify it locally. Scale additional review to the risk; do not impose a fixed number of reviewers on routine work. AI reviewers may analyze independently when available, but never claim that agents sharing an account are distinct GitHub reviewers or that AI review is human security qualification.

Keep the SPEC's independent security-informed review and evidence gates before enabling or releasing authority, trust-boundary, privacy, credential, execution-isolation, durable-acknowledgement/recovery, platform-lifecycle, or release-control capabilities. Those gates protect users and are not routine maintainer checkpoints. If qualified review or required evidence is unavailable, keep the affected capability disabled or release blocked, document the specific missing gate, and continue safe work elsewhere. The S0 scaffold cannot execute agents, authorize tools, enroll devices, store credentials, or mutate repositories.

Do not weaken authority, privacy, durable acknowledgement, or execution isolation to make checks pass. Keep CI actions pinned and permissions minimal; PR jobs receive no release secrets. Publication, signing, deployment, and store submission remain behind their explicit release gates and task authorization.

The initial exact-scope specification reviews are historical bootstrap evidence, not a recurring gate for routine work. Preserve historical reviewer records as written. Governance prose does not prove reviewer independence or qualify product security.
