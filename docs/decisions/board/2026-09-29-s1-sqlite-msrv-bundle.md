# S1 SQLite/MSRV board review bundle

Version: 1  
Review artifact: [`2026-09-29-s1-sqlite-msrv-proposal.md`](2026-09-29-s1-sqlite-msrv-proposal.md)

The review input consists of the proposal and these immutable-at-dispatch files. Reviewers must read these paths only, plus the governing board policy below. SHA-256 values identify the exact bytes reviewed.

| Path | SHA-256 |
|---|---|
| `docs/decisions/board/2026-09-29-s1-sqlite-msrv-proposal.md` | `56e8e68361b3d378b8db2862f567b82e91b945212f4c9335bbf6436cc5f54e6b` |
| `SPEC.md` | `95004071158164f03b3f00308b9fbf071ae8987cea858ab3e6141da3c742e73c` |
| `Cargo.toml` | `ca4f90d97a5b67a3a7b6565859dfbf3c61022fcb8eb473b72a791c66d1d3b1aa` |
| `crates/supervisor/Cargo.toml` | `139d4996fb94131ede712fe63c33a2c9134c1a9d4a72dc377988fa16d45cc158` |
| `crates/README.md` | `b2f6dc4bb074e5280bb2611baa78f4e350da610e661674195a774c5fb1edc467` |
| `rust-toolchain.toml` | `7553618f800829444096beea3dce84c3d07585efda504c19adb6787369bbd9e8` |
| `.github/workflows/verify.yml` | `aa1f026414342fbf09d2e68f1bc03455a28da02d1a646adad478152eca959055` |

Governing review policy (not a choice artifact): `docs/agents/decision-board.md`, SHA-256 `df280594fbaa77848f537499bf7d6136b43d20e937696d7a3612641984ced928`.

## Data routing

All review content is from the publicly visible `1deat0r/Conductor` repository and public rusqlite release/manifest metadata. The bundle contains no credentials, personal data, or production data. Seats run as Codex subagents on this task's local execution host; no additional model provider or third-party communication is introduced. Each seat must confirm this content is authorized for its assigned model/tool and execution context before reading; stop with a routing limitation if it cannot.
