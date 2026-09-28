# Author response to round 1

Original scope hash: `a6297a772d4e0f0f23e1bf7d9703b96d4057d2557a4fed0961c2f9f5f4cc8d29`.

Intermediate isolation-fix hash: `ba937d7228c6010bfa7b96f08bef878eb90b148294bb9408df3d7ed9014c75b1`.

Final round-2 scope hash after the owner's added GitHub requirement: `bc423cc2df0c3cf2ee1c663d8a47bbfd9ade3374630d16ef085648f5b4b8c598`. R25, the mandatory GitHub workflow, contributor instructions, CI and main-ruleset configuration were added to normative review scope before resubmission. No repository is created until all experts approve that final scope.

- SYS-001 / PLAT-001: R15 now separately defines trusted account authority and the exclusions/egress guarantees required only of qualified restricted/managed profiles. Trusted mode explicitly retains account resources and cannot claim same-user protection. UI disclosure and no silent fallback remain mandatory. R09 now explicitly requires inaccessible broker credentials and policy/operation stores for its strong guarantees. R16 shell approval now names the actual profile authority rather than implying every shell is sandboxed. Threat-model and profile gate wording are aligned.
- SYS-N02: R02 now calls deferred infrastructure a message broker/stream service, distinct from the credentialed effect broker.
- SEC-N02: R18 traceability explicitly gates authorization and URL-expiry tests before S2 team artifact APIs are exposed.
- SEC-N01: CI actions now use full upstream commit SHAs with version comments; the SHAs were resolved from the official repositories. This source hardening happened while the first review was in progress, so the preserved round-1 observation may describe an earlier source read.
- PLAT-N01: `pnpm verify` now includes Rust formatting and Clippy through `check:rust`. This source hardening also happened during round 1; it passed locally.
- SYS-N01: Retained as an S1 schema/conformance implementation note. R05/R06 already require typed transitions before enabling execution, and the reviewer identified this as nonblocking.

All original reviewer verdicts remain unmodified. Every role is asked to review the revised normative bundle independently before an aggregate approval record can be created.
