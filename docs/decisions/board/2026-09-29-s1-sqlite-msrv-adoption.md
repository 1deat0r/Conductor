# Adoption metadata: S1 SQLite binding and Rust minimum

Date: 2026-09-29
Status: ACTIVE

The Critical expert board's `LATEST_RUSQLITE_RAISE_MSRV` decision is active at implementation commit `f00d8079ceeb8a1461c856359f07aa3b9fb86043`, fast-forwarded to protected `main` after exact-commit checks passed.

## Verification evidence

- Local `cargo +1.88.0 test --workspace --locked --offline`: passed.
- Local `pnpm verify`: passed.
- [Exact-SHA CI run 36483779511](https://github.com/1deat0r/Conductor/actions/runs/36483779511): `rust-msrv`, `source (ubuntu-latest)`, `source (windows-latest)`, and `source (macos-latest)` all passed.
- The live `Protected main integration` ruleset (ID `24102312`) required the three `source` contexts, linear history, no deletion, and fast-forward updates. Main was at `a9bcec067ba0a82455a88dd22743cd5af7e265dc`; commit `f00d807` was its fast-forward descendant.
- `conductor-supervisor doctor` still reports `execution_available: false`; an unsupported `start` invocation exits 2.

The earlier candidate `4e37350bb28633388329499e7106ddf123a463cb` failed macOS tests because parallel test fixtures reused a timestamp-derived directory name. The fixture now uses a process-local atomic ID; the corrected exact-SHA run above passed on all platforms.

This adoption activates the binding/MSRV decision only. The S1 durable inbox, full Start/Query/Cancel receipts, process launch, and crash-boundary qualification remain incomplete; the supervisor CLI still does not expose the journal.
