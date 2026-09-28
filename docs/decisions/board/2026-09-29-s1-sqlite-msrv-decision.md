# S1 SQLite binding and Rust minimum decision

Status: unanimously approved by the Critical expert board; implementation awaits exact-commit CI before protected-main adoption.  
Date: 2026-09-29

## Decision

Select `LATEST_RUSQLITE_RAISE_MSRV`: pin `rusqlite` 0.40.2 with bundled SQLite and raise the workspace Rust minimum from 1.85 to 1.88. Keep extension loading disabled. Preserve the existing Linux, Windows, and macOS clean-build checks, and add a CI check that runs the Rust workspace tests on Rust 1.88.0.

This selection preserves SPEC R02, R07, and R08. It does not enable process launch or expose the journal through the S0 CLI/API. Before local commit, the real supervisor journal must pass with Rust 1.88.0 and `pnpm verify`. The Rust 1.88 CI check and existing platform matrix run against the exact committed candidate, and all required statuses must pass before that commit is synchronized to protected `main`. This follows the exact-commit gate in `docs/development-workflow.md`; a candidate branch commit is not the decision's adoption event.

The Rust 1.85-preserving option was rejected because rusqlite 0.39.0's public named-savepoint API interpolates the caller-provided name into executable SQL for `SAVEPOINT`, `RELEASE`, and `ROLLBACK TO`. Upstream fixed that SQL injection path in rusqlite 0.40.1. A prose-only restriction on how future callers use this API was not accepted as sufficient containment.

## Critical board round 1

Reviewed artifact: proposal v2, SHA-256 `58920ee517b234937678044a36db8492732df97245b6749a9c9fc5085a6c0b34`.  
Bundle manifest: [`2026-09-29-s1-sqlite-msrv-bundle-v2.md`](2026-09-29-s1-sqlite-msrv-bundle-v2.md), SHA-256 `a0481323b373149963c3b78917cab2a8fcebe26f13ceb9bc7c31d4fb8d55768d`.  
Repository source files were read from immutable base commit `a9bcec067ba0a82455a88dd22743cd5af7e265dc` and checked against the bundle's per-file hashes.  
Exact isolated prompts: [`2026-09-29-s1-sqlite-msrv-critical-prompts.md`](2026-09-29-s1-sqlite-msrv-critical-prompts.md).

| Mandatory seat | Model/tool | Latest 0.40.2 / Rust 1.88 | Preserve 0.39.0 / Rust 1.85 | Concise rationale |
|---|---|---|---|---|
| Domain/product | gpt-6-sol, Codex subagent | BUILD (rank 1) | CONDITIONAL (rank 2) | No product requirement was found for Rust 1.85; the repository's pinned toolchain and CI use 1.98.1. The older API needs an enforceable named-savepoint containment rule. |
| Systems architecture | gpt-6-astra, Codex subagent | BUILD (rank 1) | CONDITIONAL (rank 2) | Latest preserves the SQLite ownership boundary. The preserve option does not define effective containment for the unsafe named-savepoint API. |
| Security/privacy | gpt-5.6-sol, Codex subagent | BUILD (rank 1) | REJECT (rank 2) | A future tainted savepoint name could alter durable journal SQL; a prose-only prohibition is insufficient. |
| Platform/reliability | gpt-6-luna, Codex subagent | BUILD (rank 1) | REJECT (rank 2) | Latest avoids the injection path. Keep the 3-OS matrix; the Linux probes alone do not qualify Windows/macOS. |
| Adversarial | gpt-5.6-terra, Codex subagent | BUILD (rank 1) | REJECT (rank 2) | The vulnerable API is public and the board has no enforceable API restriction for the older binding. |

All five mandatory seats returned BUILD for the same option, satisfying the Critical unanimity rule. No confirmed blocker remains for the selected option. The reviewers treated the actual supervisor journal tests, minimum-toolchain CI, `pnpm verify`, and current 3-OS CI as precommit implementation gates, not as a reason to defer the dependency decision.

The board used five fresh Codex subagent contexts across different model variants, all on this task's local execution host. These are distinct agent contexts, not independent human reviewers or GitHub identities.

## Earlier Material round

The initial Material proposal v1 (`2026-09-29-s1-sqlite-msrv-proposal-v1.md`, SHA-256 `56e8e68361b3d378b8db2862f567b82e91b945212f4c9335bbf6436cc5f54e6b`) received three CONDITIONAL ballots for both options. All three preferred preserving 1.85 but required minimum-toolchain evidence before adoption. No option was adopted. Round 1 prompts and the reviewed bundle are retained in [`2026-09-29-s1-sqlite-msrv-round1-prompts.md`](2026-09-29-s1-sqlite-msrv-round1-prompts.md) and [`2026-09-29-s1-sqlite-msrv-bundle.md`](2026-09-29-s1-sqlite-msrv-bundle.md). The later verified probes and upstream injection-fix evidence changed the proposal and raised its classification to Critical.

## Evidence and data routing

- The exact isolated candidate probes and results are in [`2026-09-29-s1-sqlite-probes/`](2026-09-29-s1-sqlite-probes/). On x86_64 Linux, rusqlite 0.39.0 passed its locked probe with Rust 1.85.0, and rusqlite 0.40.2 passed its locked probe with Rust 1.88.0. These were binding/WAL/FULL round-trip checks, not tests of Conductor's implementation or native cross-platform qualification.
- The workspace before adding SQLite passed `cargo +1.85.0 check --workspace --locked`.
- Upstream identifies 0.40.2 as latest and records its 1.88.0 MSRV in the [0.40.2 release](https://github.com/rusqlite/rusqlite/releases/tag/v0.40.2). The [0.40.1 release](https://github.com/rusqlite/rusqlite/releases/tag/v0.40.1) includes the named-savepoint SQL injection fix; compare the [0.39.0 transaction source](https://raw.githubusercontent.com/rusqlite/rusqlite/v0.39.0/src/transaction.rs), [0.40.2 transaction source](https://raw.githubusercontent.com/rusqlite/rusqlite/v0.40.2/src/transaction.rs), and [fix PR #1854](https://github.com/rusqlite/rusqlite/pull/1854).
- All local review inputs were from the public `1deat0r/Conductor` repository; upstream inputs were public rusqlite source/release metadata. No credentials, personal data, or production data were shared. Routing was authorized under SPEC R20/R21 for the local Codex subagents; no new provider or third-party communication was introduced.

## Adoption record

The selected implementation must be locally verified, reviewed, and committed atomically with this decision record. The activation SHA and verification evidence are recorded in the separate [adoption metadata](2026-09-29-s1-sqlite-msrv-adoption.md). That metadata does not alter the reviewed proposal, option, or vote.
