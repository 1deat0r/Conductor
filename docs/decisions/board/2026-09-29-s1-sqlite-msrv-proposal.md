# S1 SQLite binding and Rust minimum proposal

Status: proposed for Critical board review, round 1  
Version: 2  
Date: 2026-09-29  
Decision tier: Critical — one option retains a documented SQL injection path in an upstream public API.

## Goal

Choose the SQLite binding/toolchain policy for the first S1 supervisor journal slice. The repository already specifies SQLite as the local durable store, WAL plus `synchronous=FULL` for critical state, and conservative recovery after an unknowable process-start crash window. This decision concerns the Rust dependency, declared compiler floor, and dependency security posture.

## Constraints and acceptance criteria

- Preserve `SPEC.md` R02, R07, and R08 as written; this choice does not reopen journal semantics.
- Use a bundled SQLite build for reproducible local builds, including Windows; keep extension loading disabled.
- The local probes in `2026-09-29-s1-sqlite-probes/` compile the exact candidate bindings and resolved dependency snapshots at their candidate minimums. They establish only the basic file-backed WAL/FULL round trip on x86_64 Linux, not the Conductor journal or native qualification on Windows/macOS.
- Before the implementation commit, run the actual supervisor journal tests with the selected minimum Rust toolchain, add a CI job that tests the exact declared minimum, run `pnpm verify`, and retain the current Ubuntu/Windows/macOS clean-build checks. If the actual locked workspace graph fails at the selected minimum, do not commit or silently change the option; gather the failure and reconvene the board if a different material option is needed.
- Do not launch real agents or make S0 execution available in this slice. The journal remains an internal S1 implementation seam.

## Mutually exclusive choice set: SQLite binding and Rust minimum

### `LATEST_RUSQLITE_RAISE_MSRV`

Use exact `rusqlite = 0.40.2` with its `bundled` feature and raise the workspace `rust-version` from 1.85 to 1.88. The upstream 0.40.2 release is marked latest as of 2026-09-29 and says its MSRV is 1.88.0. Its release history includes the 0.40.1 fix for SQL injection when a caller supplies a tainted SAVEPOINT name; the 0.40.2 source quotes identifiers before constructing SAVEPOINT, RELEASE and ROLLBACK SQL. Run the exact minimum-toolchain CI job on Rust 1.88.0. Sources: [0.40.2 release](https://github.com/rusqlite/rusqlite/releases/tag/v0.40.2), [0.40.1 release](https://github.com/rusqlite/rusqlite/releases/tag/v0.40.1), [fix PR #1854](https://github.com/rusqlite/rusqlite/pull/1854), [0.40.2 transaction source](https://raw.githubusercontent.com/rusqlite/rusqlite/v0.40.2/src/transaction.rs).

### `PRESERVE_RUST_185`

Keep the declared workspace minimum at Rust 1.85 and use exact `rusqlite = 0.39.0` with its `bundled` feature. The isolated probe passes at Rust 1.85.0. However, 0.39.0's public `Savepoint::with_name` path interpolates its caller-provided name into SQL for SAVEPOINT, RELEASE, and ROLLBACK; upstream fixed this in 0.40.1. This option therefore retains a known SQL injection path if future code passes tainted data as a savepoint name. Reviewers must decide whether a durable repository restriction against that API is sufficient or whether the dependency risk disqualifies this option. If selected, add the Rust 1.85.0 minimum-toolchain CI job. Sources: [0.39.0 transaction source](https://raw.githubusercontent.com/rusqlite/rusqlite/v0.39.0/src/transaction.rs), [fix PR #1854](https://github.com/rusqlite/rusqlite/pull/1854).

## Verified candidate probes

The two isolated snapshots are saved under `2026-09-29-s1-sqlite-probes/` and their commands/results are in that directory's README. On local x86_64 Linux, each exact lockfile passed its pinned-minimum `cargo test --locked --offline`; the test enabled WAL and `synchronous=FULL`, committed a launch-intent row, closed/reopened the database, and read the committed row. The existing workspace also passed `cargo +1.85.0 check --workspace --locked` before any SQLite dependency was added. These checks do not qualify platform-native behavior or substitute for tests of the eventual supervisor journal.

## Repository evidence

- `Cargo.toml` currently declares `rust-version = "1.85"`; `rust-toolchain.toml` pins 1.98.1.
- `.github/workflows/verify.yml` installs Rust 1.98.1 for its existing Ubuntu/Windows/macOS matrix and has no minimum-toolchain check.
- `crates/supervisor/Cargo.toml` currently has no SQLite dependency.
- `SPEC.md` R02 assigns local durable state to SQLite; R07 requires WAL + synchronous FULL and fail-closed acceptance on I/O errors; R08 requires uncertainty after the spawn/persist crash window to remain interrupted/outcome-unknown without automatic relaunch.
- `crates/README.md` limits S1 to durable inbox and supervisor Start/Query/Cancel with fault injection before real agent launch.

## Review questions

For the assigned lens, assess whether the existing Rust 1.85 declaration should be preserved given the current 1.98 toolchain and CI, the verified candidate probes, and the difference in SAVEPOINT-name injection handling. Return a verdict for every option, identify any confirmed Critical blocker, and state the evidence needed to close each blocker. No option may be adopted without all five mandatory seats returning BUILD for that same option and every material blocker being closed.
