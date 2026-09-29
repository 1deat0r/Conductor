# Critical proposal: S1 host-receipt ordering

Version: 3 — final-round immutable review artifact. Round-one artifact: [`2026-09-29-s1-host-receipt-ordering-proposal-v1.md`](2026-09-29-s1-host-receipt-ordering-proposal-v1.md), SHA-256 `8717abe7821b97cee0f42ef9e964ded918306dc9eb0b35dc3ce1f0b546784901`. Round-two artifact: [`2026-09-29-s1-host-receipt-ordering-proposal-v2.md`](2026-09-29-s1-host-receipt-ordering-proposal-v2.md), SHA-256 `d4f6c9a9481d3cb88960d615c81ed521dd8b6743ec865aa229c755f177e358c3`. This proposal is not active policy unless adopted and committed under `docs/agents/decision-board.md`.

## Decision

For the Issue #3 schema-only slice, decide whether to define host receipt ordering and legal transitions now, or explicitly defer receipt consumption and its ordering contract to the later durable-journal/API slice. Choosing deferral settles the present scope: Issue #3 validates receipt shapes and states but provides no consumer that orders or projects them. The stable option IDs from Version 1 remain unchanged. Host receipts are scoped to `host_id`; coordinator receipts remain a separate authority and there is no total order across those authorities.

## Fixed context and constraints

- `SPEC.md` R07 requires durable host inbox/deduplication before `host_accepted`, records effect resolution separately, uses at-least-once delivery, and says retries with the same ID return prior state while changed content fails. State and outbox changes commit in one transaction.
- `SPEC.md` R11 says uncertain elapsed time and clock rollback fail closed for protected work; wall-clock time is not an authoritative expiry source.
- `SPEC.md` R24 requires recovery to start write-frozen, fence the old deployment, establish a non-reused recovery generation outside the restored snapshot or rotate/re-enroll credentials, invalidate old grants, reconcile lost intents/receipts before retries, and prevent restored counters from overriding live fencing. Clones receive new identities.
- Issue #3 acceptance requires authority-scoped coordinator and host receipt states, no cross-authority total order, forbids automatic replay/admission of `outcome_unknown`, and defers storage-level durability to the journal slice. This issue defines and validates the typed envelope; it does not wire consumers or execution.
- The current draft `HostReceiptV1` has `updated_at_unix_ms` but no per-host ordering value. The timestamp is not guaranteed monotonic and cannot distinguish reordered observations.
- No option enables command admission or execution. S0 stays execution-disabled. The chosen contract must preserve fail-closed recovery and exact-content idempotency.

## Options

### `HOST-REVISION`

Add `host_generation` and safe-integer `receipt_revision` to every host receipt. `host_generation` is an authenticated, non-reused host incarnation/recovery generation established outside the restorable receipt snapshot, or is changed by deliberate credential rotation/re-enrollment. Consumers compare the receipt generation against the currently authenticated host authority; a fenced/old generation is rejected. A restored host starts write-frozen, fences the old writer, invalidates old grants, and reconciles lost intents/receipts before any retry or admission. Clones use new identities. Within the active `(host_id, host_generation, command_id)` scope, revision starts at 1 and increments exactly once for every durable state transition; state, revision, and outbox change commit atomically under the sole authoritative host writer. `payload_digest` remains fixed. The timestamp is informational. Consumers accept only a higher revision; an identical equal-revision snapshot is an idempotent replay, a conflicting equal revision is an integrity error, and a lower revision is stale.

Legal transitions:

- `pending_host_admission` → `host_accepted`, `host_rejected`, or `outcome_unknown`
- `host_accepted` → `effect_resolved` or `outcome_unknown`
- `outcome_unknown` → `effect_resolved` only after reconciliation evidence; it never triggers admission or effect replay
- `host_rejected` and `effect_resolved` are terminal

### `DEFER-TO-JOURNAL`

Keep the receipt envelope as a parse-only snapshot in this contract slice. Treat `updated_at_unix_ms` as descriptive metadata, never as ordering or authority. No projection, retry, reconciliation, or recovery consumer may replace a known receipt from an unordered snapshot. Before a future journal/API consumer uses receipts, its contract must define an authority-scoped durable sequence, legal state transitions, atomic state/outbox updates under the sole writer, stale-writer fencing, and R24 restore/recovery handling (write-freeze, non-reused generation or credential rotation/re-enrollment, invalidation, and reconciliation before retries). S0 remains unable to consume or act on these receipts.

## Acceptance criteria

1. Select exactly one of the two options: either define ordering now, or explicitly adopt the Issue #3 scope deferral with a future-consumer gate. Do not leave the choice to the user.
2. Under `HOST-REVISION`, a stale, reordered, restored, or cloned host receipt cannot roll back newer state or reauthorize execution. Under `DEFER-TO-JOURNAL`, no S0 or Issue #3 consumer may order, project, retry, reconcile, or recover from receipt snapshots before the later contract satisfies the listed journal and R24 requirements.
3. `outcome_unknown` is never automatically re-admitted or replayed; any later resolution requires reconciliation evidence.
4. The choice is compatible with R07/R11/R24 and Issue #3 scope, and does not enable S0 execution.
5. Each voter returns a verdict for both stable option IDs, ranks them or says `NONE`, cites any blocker with an exact requirement and closure condition, and distinguishes blockers from notes.

Decision tier: **Critical** — this choice governs durable receipt recovery correctness and whether an uncertain command can be replayed.
