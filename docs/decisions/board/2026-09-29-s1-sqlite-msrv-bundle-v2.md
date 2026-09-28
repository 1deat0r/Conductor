# S1 SQLite/MSRV Critical board review bundle

Version: 2  
Review artifact: [`2026-09-29-s1-sqlite-msrv-proposal.md`](2026-09-29-s1-sqlite-msrv-proposal.md)

The review input consists of the proposal and the following immutable-at-dispatch files. SHA-256 values identify the exact bytes reviewed.

| Path | SHA-256 |
|---|---|
| `docs/decisions/board/2026-09-29-s1-sqlite-msrv-proposal.md` | `58920ee517b234937678044a36db8492732df97245b6749a9c9fc5085a6c0b34` |
| `SPEC.md` | `95004071158164f03b3f00308b9fbf071ae8987cea858ab3e6141da3c742e73c` |
| `Cargo.toml` | `ca4f90d97a5b67a3a7b6565859dfbf3c61022fcb8eb473b72a791c66d1d3b1aa` |
| `crates/supervisor/Cargo.toml` | `139d4996fb94131ede712fe63c33a2c9134c1a9d4a72dc377988fa16d45cc158` |
| `crates/README.md` | `b2f6dc4bb074e5280bb2611baa78f4e350da610e661674195a774c5fb1edc467` |
| `rust-toolchain.toml` | `7553618f800829444096beea3dce84c3d07585efda504c19adb6787369bbd9e8` |
| `.github/workflows/verify.yml` | `aa1f026414342fbf09d2e68f1bc03455a28da02d1a646adad478152eca959055` |
| `docs/agents/decision-board.md` | `df280594fbaa77848f537499bf7d6136b43d20e937696d7a3612641984ced928` |
| `docs/agents/decision-board-review-prompt.md` | `673b03dcb4051827486107189db872b673c31e43cc3a452c0759f95737540568` |
| `docs/decisions/board/2026-09-29-s1-sqlite-probes/README.md` | `01852e62bfdb2eba2e3ab5e8b99f348a2ddc397160717aaa2dc5177ff7428540` |
| `docs/decisions/board/2026-09-29-s1-sqlite-probes/preserve/Cargo.toml` | `07fb1538c684fbd4d22116e1d382e347fffb43a0422876e1419d8dc70755b6a8` |
| `docs/decisions/board/2026-09-29-s1-sqlite-probes/preserve/Cargo.lock` | `234463a0d080a24cd7f65431f96d92990129abd0495cb4bc0eb5b624456b9061` |
| `docs/decisions/board/2026-09-29-s1-sqlite-probes/preserve/src/main.rs` | `524d8e01efa71a150b11f53cbe6e2044b8ce2297b1d7d3d5bf176aec5cf28f86` |
| `docs/decisions/board/2026-09-29-s1-sqlite-probes/latest/Cargo.toml` | `06e87f91d33a416cd6006ac414969b72128062b2a627fbc1582b50b5f12c6589` |
| `docs/decisions/board/2026-09-29-s1-sqlite-probes/latest/Cargo.lock` | `3be6bc4246e9649bb3980278fff5c659f95f668777792e6580d69e76f276efb4` |
| `docs/decisions/board/2026-09-29-s1-sqlite-probes/latest/src/main.rs` | `524d8e01efa71a150b11f53cbe6e2044b8ce2297b1d7d3d5bf176aec5cf28f86` |

## Exact external primary sources permitted for this review

- `https://github.com/rusqlite/rusqlite/releases/tag/v0.40.2`
- `https://github.com/rusqlite/rusqlite/releases/tag/v0.40.1`
- `https://github.com/rusqlite/rusqlite/pull/1854`
- `https://raw.githubusercontent.com/rusqlite/rusqlite/v0.39.0/src/transaction.rs`
- `https://raw.githubusercontent.com/rusqlite/rusqlite/v0.40.2/src/transaction.rs`
- `https://raw.githubusercontent.com/rusqlite/rusqlite/v0.39.0/Cargo.toml`
- `https://raw.githubusercontent.com/rusqlite/rusqlite/v0.40.2/Cargo.toml`
- `https://github.com/rusqlite/rusqlite`

## Data routing

All local review content is from the publicly visible `1deat0r/Conductor` repository; all permitted external sources are public rusqlite upstream source and release metadata. The bundle contains no credentials, personal data, or production data. Seats run as Codex subagents on this task's local execution host; no additional model provider or third-party communication is introduced. Each seat must confirm this content is authorized for its assigned model/tool and execution context before reading; stop with a routing limitation if it cannot.
