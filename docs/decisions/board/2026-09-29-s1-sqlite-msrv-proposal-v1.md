# S1 SQLite binding and Rust minimum proposal

Status: proposed for Material board review, round 1  
Date: 2026-09-29  
Decision tier: Material — Rust toolchain minimum is a supported platform commitment.

## Goal

Choose the SQLite binding/toolchain policy for the first S1 supervisor journal slice. The repository already specifies SQLite as the local durable store, WAL plus `synchronous=FULL` for critical state, and conservative recovery after an unknowable process-start crash window. This decision concerns only the Rust dependency and declared compiler floor.

## Constraints and acceptance criteria

- Preserve `SPEC.md` R02, R07, and R08 as written; this choice does not reopen journal semantics.
- Use a bundled SQLite build for reproducible local builds, including Windows, unless a reviewer provides evidence that another option better satisfies the repository's constraints.
- Keep extension loading disabled.
- The selected release and its resolved dependencies must compile and pass the supervisor journal checks on the declared Rust minimum. Record the exact toolchain and command as evidence.
- Do not launch real agents or make S0 execution available in this slice. The new journal remains an internal S1 implementation seam.

## Mutually exclusive choice set: SQLite binding and Rust minimum

### `LATEST_RUSQLITE_RAISE_MSRV`

Use exact `rusqlite = 0.40.2` with its `bundled` feature and raise the workspace `rust-version` from 1.85 to 1.88. The upstream 0.40.2 release is marked latest as of this review date and says its MSRV is 1.88.0. The upstream README describes `bundled` as compiling SQLite from source and avoiding reliance on the user's installed SQLite version. Evidence: [rusqlite 0.40.2 release](https://github.com/rusqlite/rusqlite/releases/tag/v0.40.2), [rusqlite README](https://github.com/rusqlite/rusqlite).

### `PRESERVE_RUST_185`

Keep the declared workspace minimum at Rust 1.85 and use exact `rusqlite = 0.39.0` with its `bundled` feature. The 0.39.0 manifest uses edition 2021, exposes the `bundled` feature, and does not declare `rust-version`; compatibility with 1.85 is therefore unproven until the acceptance check succeeds. Require `cargo +1.85.0 check --locked -p conductor-supervisor` plus the journal tests on that toolchain, and add an automated minimum-toolchain compile check so the retained platform commitment stays evidenced. Evidence: [rusqlite 0.39.0 manifest](https://raw.githubusercontent.com/rusqlite/rusqlite/v0.39.0/Cargo.toml).

## Evidence available to reviewers

- `Cargo.toml` currently declares `rust-version = "1.85"`.
- `crates/supervisor/Cargo.toml` currently has no SQLite dependency.
- `SPEC.md` R02 assigns local durable state to SQLite; R07 requires WAL + synchronous FULL and fail-closed acceptance on I/O errors; R08 requires uncertainty after the spawn/persist crash window to remain interrupted/outcome-unknown without automatic relaunch.
- `crates/README.md` limits S1 to durable inbox and supervisor Start/Query/Cancel with fault injection before real agent launch.
- The current `rust-toolchain.toml` and CI workflow are in the review bundle to assess whether the declared minimum is currently enforced.

## Review questions

For the assigned lens, assess which option best balances the existing Rust support commitment, reproducible cross-platform SQLite behavior, dependency freshness, and ongoing verification cost. Report evidence-backed blockers and any exact check needed before the option is adopted. Do not recommend a third option without mapping it to the same mutually exclusive choice set and identifying what repository commitment it changes.
