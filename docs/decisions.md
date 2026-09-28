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
| ADR16 | Mandatory GitHub issue/PR development with independent expert reviews and protected main | Extra review latency; no unilateral bypass or fabricated distinct GitHub identities |
