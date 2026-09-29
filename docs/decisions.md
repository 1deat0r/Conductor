# Architecture decisions

These accepted design decisions constrain implementation; they are not completed features.

| ID | Decision | Cost and revisit trigger |
|---|---|---|
| ADR01 | Electron/React desktop, Rust execution substrate | Higher memory cost; revisit only with measured renderer bottlenecks and a migration budget |
| ADR02 | React Native/Expo native projects for both phones | Native signing/security/device work still required; do not treat Expo Go as qualification |
| ADR03 | Separate controller and supervisor lifetimes | Extra protocol/storage boundary; justified by UI/control restart survival |
| ADR04 | One writer per aggregate; desired and observed state separate | Offline edits may wait/reject; no hidden last-writer-wins execution state |
| ADR05 | At-least-once commands, durable deduplication, explicit unknown effects | Availability can yield to reconciliation; no universal exactly-once promise |
| ADR06 | Verified isolation profiles outside model control | Trusted local mode has weaker guarantees; unsupported restricted mode cannot downgrade silently |
| ADR07 | SQLite locally, Postgres plus object storage remotely | Cross-store artifact commit protocol needed; no automatic SQLite sync |
| ADR08 | Typed brokered effects and immutable action inputs | Sensitive generic shell effects cannot always be represented for phone approval |
| ADR09 | Modular control application first | Gateway/supervisor split by lifetime; other service splits require measured reason |
| ADR10 | Replaceable model/agent/runner adapters | Capability gaps remain visible; standardized protocols do not erase vendor differences |
| ADR11 | Source search before embeddings; provenance-aware memory | Embeddings permitted after task-level evaluation and deletion/privacy design |
| ADR12 | Managed VM isolation and dedicated/reset native pools | Idle/native compute costs are part of pricing; no hostile shared native pool without reset proof |
| ADR13 | Hosted TLS/KMS trust model, local-only and later self-hosted options | No E2EE marketing until a separate end-to-end design is qualified |
| ADR14 | No publication license selected in scaffold | Public source repository only after owner-authorized unanimous spec approval; packages remain nonpublishable, no open-source license inferred, licensing/name/provenance before product distribution |
| ADR15 | S0 implements only truthful health and nonexecuting shells | This keeps scaffold review distinct from unbuilt security/runtime claims |
| ADR16 | Initial GitHub issue/PR workflow with fixed expert reviews (superseded 2026-09-29 by ADR17) | Historical bootstrap policy; retained here to explain the initial review and protection setup |
| ADR17 | Autonomous, local-first AI-agent development with exact-commit CI-gated main synchronization | Agents verify and review locally, commit atomically, then synchronize without routine user approval; CI, no-force/no-delete rules and product security/release gates remain enforced |
| ADR18 | Material Conductor product and engineering choices are decided by an independent expert-agent board | No user tie-break or approval loop; tiered quorum, evidence record and safe defer outcomes; board design votes do not qualify production security or authorize external actions (see docs/agents/decision-board.md and docs/decisions/board/2026-09-29-autonomy-adoption.md) |
| ADR19 | Supervisor local journal uses rusqlite 0.40.2 with bundled SQLite; workspace Rust minimum is 1.88 and receives an explicit minimum-toolchain CI check | Raises the declared Rust floor from 1.85; chosen to avoid the tainted SAVEPOINT-name SQL injection path in rusqlite 0.39.0; revisit if a securely fixed binding supports the older minimum (see docs/decisions/board/2026-09-29-s1-sqlite-msrv-decision.md) |
| ADR20 | Reject raw JSON number tokens that decode to IEEE-754 negative zero before S1 schema validation or durable command identity calculation | A byte spelling that JCS collapses to `0` must not enter TS/Rust command digests or replay fingerprints; this project rule covers equivalent lexemes and underflow (see docs/decisions/board/2026-09-29-s1-negative-zero-decision.md) |
| ADR21 | Advance host command persistence only as a sealed internal store over canonical `CommandV1` until host receipt ordering and recovery are adopted | Keeps production receipt projection, admission, outbox delivery and execution unavailable while the cross-generation and R24 recovery contract is incomplete (see docs/decisions/board/2026-09-29-s1-issue1-command-journal-decision.md) |

## Deferred board outcomes

- **2026-09-29 — S1 host-receipt ordering:** no option met the Critical board's final-round unanimity rule. No ordering or transition design was adopted; receipts remain parse-only until a future journal/API proposal resolves generation-change reconciliation and restore fencing. See [the board record](decisions/board/2026-09-29-s1-host-receipt-ordering.md).
- **2026-09-29 — S1 Issue #1 journal boundary:** the Critical board unanimously adopted `SEALED-STORE`; this advances canonical command persistence while leaving host receipt consumers and admission deferred. The prior host-receipt ordering outcome remains unchanged. See [the board record](decisions/board/2026-09-29-s1-issue1-command-journal-decision.md).
