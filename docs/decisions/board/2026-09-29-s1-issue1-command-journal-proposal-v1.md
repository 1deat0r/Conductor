# Critical decision: Carrying Issue #1 forward under the receipt-ordering deferral

Version: 1 — immutable first-round artifact.
Decision tier: **Critical** — this choice governs durable command identity, receipt projection, recovery, and whether a stored command can lead to admission or execution.

## Decision to make

Choose the safe next implementation slice for the existing Issue #1 command/session journal after the typed S1 protocol contracts landed. The previous Critical board terminally deferred host-receipt ordering. This is a new, concrete journal proposal: either keep the next commit strictly internal and receipt-free, define the complete ordered/recovery-aware host receipt contract first, or defer host journaling while continuing independent supervisor work. No prior terminal vote is reopened or treated as an adoption.

## Fixed context

- Current development target `feat/issue-1-command-journal` is commit `d7b3f7314b1ac00065ae7a9363e69080492442e0`; it was authored before the typed S1 contracts and is not on current `main`. Its host `CommandEnvelope` is bespoke: it hashes an arbitrary raw payload plus envelope fields and has no `submission_fingerprint`. The current `CommandV1` contract requires both payload digest and submission fingerprint.
- Current `main` is `3b8cc8eb985ce19ceab005de8b8aa99a37bb48ad`. `contracts/README.md` says `HostReceiptV1` is only a typed snapshot, `updated_at_unix_ms` is descriptive, and no consumer may project, retry, reconcile, or recover from unordered snapshots. Before a journal/API consumer is added, the design must cover generation-change reconciliation, atomic state/revision/outbox ordering, stale-writer fencing, and SPEC R24 recovery.
- The preceding Critical outcome (`docs/decisions/board/2026-09-29-s1-host-receipt-ordering.md`) ended after three rounds with no adopted option. Its identified cross-generation case: a restored generation could publish an older receipt after a terminal receipt from the prior generation was observed. No fourth round was started.
- Issue #1 requests a durable host command inbox and supervisor session store, idempotent same-ID/same-content replay, mismatch rejection, WAL/FULL, atomic outbox, conservative stale/expired rejection, durable Start/Query/Cancel, and no automatic relaunch when a spawn outcome is uncertain. Its non-goals keep process launch and execution disabled.
- `SPEC.md` R04–R08, R10, R15, and R24 are normative. S0 must keep `execution_available=false`. This decision does not satisfy the full R07/R24 acceptance gates, qualify platform behavior, or authorize a runtime API, process launch, tool execution, credentials, or repository mutation. Issue #1 remains open until its full requirements and evidence are met.
- The current Issue #1 body also names `pnpm check:approvals`; the current R25 workflow and repository policy define `pnpm verify` as the local gate. Do not restore the obsolete approval command as a gate.

## Mutually exclusive options

### `SEALED-STORE` — advance only the internal storage foundation

Rebase the storage work on current `main` and align host command persistence/deduplication with validated canonical `CommandV1`, including its normative `payload_digest` and `submission_fingerprint`; do not keep a competing `CommandEnvelope` identity scheme. Keep the host journal internal to its owning crate. It may persist a command and test persistence/replay/quarantine, but its production interface returns no host receipt snapshot or acceptance state. Host admission transitions stay test-only; do not expose outbox reads/delivery, `HostReceiptV1` projection, retry, reconciliation, or recovery consumers. Preserve safe quarantine of legacy/ambiguous rows.

Keep the supervisor session store separate and internal. It may persist Start intent and support local Query/Cancel state as required by R08, but `launch_is_new` is not execution authorization; no caller may spawn or own a process from this slice. Preserve no-relaunch-on-uncertainty, keep all CLI/API/renderer routes unavailable, and keep `execution_available=false`. Do not claim host-receipt recovery or S1 execution readiness.

This is a deliberately incomplete storage foundation, not completion of Issue #1. A later Critical design and R24 evidence gate are required before host receipt snapshots or production host admission can be consumed.

### `ORDERED-RECEIPTS` — define host receipt ordering/recovery before integrating a consumer

Before integrating the host journal as an admission/receipt producer or exposing any receipt consumer, adopt a complete authority-scoped host receipt and recovery contract. It must include: a non-reused authenticated host/recovery generation established outside the restorable snapshot (or credential rotation/re-enrollment); per-command durable ordering tied atomically to state and outbox under one writer; fencing of stale generations; legal transitions and terminal-state rules; write-freeze and old-grant invalidation during restore; a coordinator/recovery authority that reconciles old-generation receipts and unresolved effects before new-generation publication; preservation of previously known terminal receipts across generation changes; conflict and unavailable-authority behavior that fails closed; and crash/restore/clone evidence satisfying R24. A restored snapshot is a claim to reconcile, never sufficient proof to replace a previously observed terminal receipt.

Do not enable S0 execution or claim these requirements from schema review alone. The current S1 local slice may not expose a consumer until the concrete generation authority and required recovery path exist and are tested.

### `DEFER-HOST-JOURNAL` — keep host command journaling out of this integration

Do not carry the Issue #1 host command-journal module forward yet. Continue only independent supervisor-session work and other S1 tasks that do not consume host receipt snapshots. Keep command admission and execution unavailable until a later journal proposal satisfies the ordering/recovery conditions above.

## Acceptance criteria for this decision

1. Select exactly one stable option, or apply the Critical terminal defer rule after at most three rounds. Every seat votes on every option; ranking never substitutes for unanimity.
2. Any adopted option preserves `execution_available=false`, no production process spawn, no unsupported API route, and no automatic replay/relaunch of unknown outcomes.
3. Under `SEALED-STORE`, no production caller can observe a host receipt snapshot, host acceptance/rejection state, or outbox delivery cursor. The stored command identity is validated `CommandV1`; legacy non-normative payloads cannot be newly admitted or silently replayed. The supervisor database remains a separate owner and its Start result is not launch authority.
4. Under `ORDERED-RECEIPTS`, no journal/API consumer is integrated until all listed generation, cross-generation, atomicity, fencing, write-freeze and R24 recovery requirements are designed and have concrete test/evidence gates.
5. Under `DEFER-HOST-JOURNAL`, the host journal stays out; other safe independent work may continue.
6. The complete Issue #1 acceptance is not marked done by any option in this proposal. After adoption, verify the exact approved artifact, inspect the diff, and commit the same normative bytes before implementing its choice.

## Review bundle

Review only the immutable files listed in the matching prompt. Source commits: current `main` `3b8cc8eb985ce19ceab005de8b8aa99a37bb48ad`; Issue #1 code `d7b3f7314b1ac00065ae7a9363e69080492442e0`. The bundle contains public repository/specification material only: no credentials, production data, private user content, or new provider routing. Assigned seats are local Codex subagent contexts. Each seat must confirm the content is authorized for its model/tool and execution region before reading; if it cannot, it must stop with a routing limitation.
