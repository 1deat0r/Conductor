# S1 sealed command/session storage evidence — 2026-09-29

Decision: [Critical board outcome — `SEALED-STORE`](../decisions/board/2026-09-29-s1-issue1-command-journal-decision.md). This slice records a local storage foundation only. Issue #1 remains open; this evidence does not qualify host receipt ordering, R07 acknowledgement/recovery, R24 restore behavior, native process lifecycle, or S1 execution.

Base: current `main` `3b8cc8eb985ce19ceab005de8b8aa99a37bb48ad`, with decision adoption commits `e783510a7bf91a744f68910c78a9aabda93670b4` and `1c8080cbd17b23d399011cf2c91126e2012081fa`.

## Local verification

Environment at verification: Linux 7.0.0-34-generic x86_64; Node v26.8.1; pnpm 11.18.0; Rust/Cargo 1.98.1.

| Command | Result | Limit |
|---|---|---|
| `pnpm verify` | Passed on 2026-09-29: spec structure (25 requirements), typecheck (5 TS packages), 17 TS/API tests, desktop production build, Rust format and clippy, and 40 Rust tests | Turbo reused cached unchanged TS tasks. Rust tests ran on this Linux host. This does not qualify Windows/macOS behavior or external CI. |
| `cargo test --locked -p conductor-supervisor --offline` | Passed on 2026-09-29 after the v1-to-v2 schema migration change: 13 library tests and 5 binary tests | Local supervisor storage and S0 journal tests only. |
| `cargo run --locked -q -p conductor-host -- doctor` and `cargo run --locked -q -p conductor-supervisor -- doctor` | Both reported `status: scaffold` and `execution_available: false` on 2026-09-29 | Linux doctor outputs do not qualify native behavior on other platforms. |
| `git diff --check` | Passed during final review | Whitespace check only. |

## Implemented and tested boundaries

- `conductor-host` has an internal, file-backed SQLite store that accepts only the protocol crate's validated `CommandV1`. It persists canonical JCS JSON, the normative `payload_digest`, and `submission_fingerprint`; identical replay is idempotent and changed identity is rejected. Tests cover reopen, concurrent first-open/migration and duplicate writes, write rollback, shared WAL/FULL configuration, and future-schema refusal.
- Legacy host command and outbox tables from schema versions 1–3 are retained under versioned quarantine names. Their command IDs are tombstoned so they cannot be newly stored as canonical commands. No production API reads their receipts or outbox rows.
- The host store is private to its crate. There is no production host-receipt snapshot, admission-state transition, outbox delivery interface, route, or executor.
- `conductor-supervisor` keeps a separate internal SQLite session store at schema version 2. The v1 S0 launch journal is transactionally renamed into quarantine, preserved unchanged, and its session IDs are tombstoned so they cannot be replayed or reported as absent; unknown or mixed v1 schemas fail closed. Start is named `record_start_intent` and returns stored session state without a `launch_is_new` signal. Query, cancellation intent, restart-to-unknown behavior, duplicate replay, and legacy schema migration have local tests. The library does not export this store to its binary or other crates.
- `conductor-host` and `conductor-supervisor` share `conductor-sqlite` for SQLite durability configuration while retaining separate stores, schemas, and ownership.
- No process is spawned or owned. The control API and health capabilities remain unchanged, with `execution_available=false`.

## Limits and next gates

The tests do not simulate real power loss, disk-full, cloned/restored host generations, native process-tree ownership, authenticated controller IPC, Windows/macOS storage lifecycle, or PTY behavior. Host receipt projection/consumption, admission, reconciliation, and recovery remain deferred until a new Critical design and R24 evidence gate satisfy the adopted board record. Do not report this slice as completion of Issue #1 or as S1 execution readiness.
