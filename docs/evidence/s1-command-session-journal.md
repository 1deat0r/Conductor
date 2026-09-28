# S1 command/session journal foundation — 2026-09-28

Issue: [#1](https://github.com/1deat0r/Conductor/issues/1)

Base commit: `ffab4b3e9e5e03d63d222e4263791ef70347ea4f`

Source tree tested locally (evidence file excluded): `535b9603d0bb86180500ba7c410ed1893ae3bb59`

Verification completed by 2026-09-28 08:50 UTC

## Environment

Ubuntu 26.04.1 LTS, Linux 7.0.0-34-generic, x86_64; Node v26.8.1; pnpm 11.18.0; Rust/Cargo 1.98.1. Execution profile: trusted developer host. Rust SQLite uses rusqlite 0.40.2 with bundled SQLite; Cargo.lock was updated with the new dependencies and used for verification.

## Results

| Command | Result | Limit |
|---|---|---|
| `pnpm install --frozen-lockfile` | Passed; lockfile unchanged | Fresh registry/platform availability was not tested |
| `pnpm verify` | Passed: spec check, all five TS package typechecks, 5 TS/API tests, desktop renderer build, Rust format/clippy, and 27 Rust tests (14 host, 3 protocol, 10 supervisor) | Turbo reused cached unchanged TS tasks; Rust tests ran on this Linux host |
| `pnpm check:approvals` | Passed for all 3 existing roles at spec SHA `46bd998630729de759eecb2e7b10ceca6ab482220c52b5122d4ce98b43ae1b01` | No normative bundle files changed; this does not review implementation correctness |
| `cargo run --locked -q -p conductor-host -- doctor` | Returned the scaffold health envelope with `execution_available: false` | Diagnostic only |
| `cargo run --locked -q -p conductor-supervisor -- doctor` | Returned the scaffold health envelope with `execution_available: false` | Diagnostic only |
| `git diff --check` | Passed | Static whitespace check only |
| GitHub Actions source matrix | Ubuntu, Windows, and macOS passed on `c63aa825c5f49b31dbe5fa9b29ead78a8c3b6563` in [run 36394341721](https://github.com/1deat0r/Conductor/actions/runs/36394341721), on `eb36f22a2810a8165c0f4f7d4d5397a5bcb2533d` in [run 36396629895](https://github.com/1deat0r/Conductor/actions/runs/36396629895), and on `6f9cc562099d542bd73b37b71eb59ffba4e99e95` in [run 36398736738](https://github.com/1deat0r/Conductor/actions/runs/36398736738) | These runs predate the final outcome-unknown quarantine fix. They are historical platform evidence only; current-head required checks must pass after that fix is pushed. |

Rust tests exercise SQLite WAL/FULL settings, command identity/digest deduplication, concurrent duplicate delivery, stale/expired admission, atomic host-admission/outbox rollback, durable state after reopen, cancellation request/confirmation, controller identity matching, and uncertain-start reconciliation without a second launch. They also cover unsupported journal envelope versions. Admission-transition coverage is test-only: production builds do not expose a path to `host_accepted` until a typed R06 command schema exists. Migration coverage keeps v1 `coordinator_stored` rows pending, quarantines unverified v1 `host_accepted` rows as `outcome_unknown`, and drops their undelivered acceptance events while preserving `effect_resolved` as terminal with its event. A schema-version-255 fixture proves unsupported legacy payloads cannot retain admission. The v2→v3 migration quarantines both pending and accepted rows because v2 may already have erased the distinction, preventing automatic replay after ambiguous history. Other fixes add a single-live-supervisor lease check and deterministic start/cancel interleavings that keep a spawn receipt cancellable until tree-stop confirmation. Failure injection used SQLite triggers; the host filesystem was not filled to test a real `SQLITE_FULL` condition.

## Scope and limitations

This change adds storage libraries only. The host journal and supervisor store are separate and are not wired to the CLI, API, renderer, or any launcher. The host no longer emits or stores a `coordinator_stored` acknowledgement; the coordinator must provide that durable provenance before delivery. No production host-admission transition is exposed before a typed R06 command schema and validation boundary exist. Supervisor `Start` persists intent but does not spawn a process; PID receipts remain diagnostic, and process-owner transitions are crate-private. Caller-supplied controller identity is stored and compared, not authenticated by these libraries. No real process crash, POSIX process group, Windows Job Object, macOS lifecycle, PTY, controller IPC/adoption, or native qualification was exercised.

Execution remains unavailable. This evidence does not qualify S1 process ownership or complete the S1 goal. Current-head required checks and expert review status are recorded on [draft PR #2](https://github.com/1deat0r/Conductor/pull/2).
