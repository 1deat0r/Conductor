# Critical proposal: S1 host-receipt ordering

Version: 1 — immutable review artifact. It is not active policy unless adopted and committed under `docs/agents/decision-board.md`.

## Decision

Choose how consumers distinguish current from stale `HostReceiptV1` observations for one host and command. This is a single mutually exclusive choice. The host receipt authority is scoped to `host_id`; coordinator receipts remain a separate authority and there is no total order across those authorities.

## Fixed context and constraints

- `SPEC.md` R07 requires durable host inbox/deduplication before `host_accepted`, records effect resolution separately, uses at-least-once delivery, and says retries with the same ID return prior state while changed content fails.
- `SPEC.md` R11 says uncertain elapsed time and clock rollback fail closed for protected work; wall-clock time is not an authoritative expiry source.
- Issue #3 acceptance requires authority-scoped coordinator and host receipt states, no cross-authority total order, and forbids automatic replay/admission of `outcome_unknown`; same-ID replay returns the current receipt only. Journal transaction durability is a later gate.
- The current draft `HostReceiptV1` has `updated_at_unix_ms` but no per-host ordering value. The timestamp is not guaranteed monotonic and cannot distinguish reordered observations.
- This decision does not enable command admission or execution. S0 stays execution-disabled. The chosen contract must preserve fail-closed recovery and exact-content idempotency.

## Options

### `HOST-REVISION`

Add a safe-integer `receipt_revision` to every host receipt. It starts at 1 and increases by exactly one for every durable state transition for `(host_id, command_id)`; `payload_digest` remains fixed. The timestamp is informational. Consumers accept only a higher revision; an identical equal-revision snapshot is an idempotent replay, a conflicting equal revision is an integrity error, and a lower revision is stale. Define the legal transitions as:

- `pending_host_admission` → `host_accepted`, `host_rejected`, or `outcome_unknown`
- `host_accepted` → `effect_resolved` or `outcome_unknown`
- `outcome_unknown` → `effect_resolved` only after reconciliation evidence; it never triggers admission or effect replay
- `host_rejected` and `effect_resolved` are terminal

### `DEFER-TO-JOURNAL`

Keep the receipt envelope as a parse-only snapshot in this contract slice. Treat `updated_at_unix_ms` as descriptive metadata, never as ordering or authority. No projection, retry, or recovery consumer may replace a known receipt from an unordered snapshot. Before a future journal/API consumer uses receipts, its contract must define an authority-scoped durable sequence and legal transition rules; S0 remains unable to consume or act on these receipts.

## Acceptance criteria

1. One option is selected without introducing any cross-authority coordinator/host ordering.
2. A stale or reordered host receipt cannot roll back a newer state or reauthorize execution.
3. `outcome_unknown` is never automatically re-admitted or replayed; any later resolution requires reconciliation evidence.
4. The choice is compatible with R07/R11 and Issue #3 scope, and does not enable S0 execution.
5. Each voter returns a verdict for both options, ranks the options or says `NONE`, cites any blocker with an exact requirement and closure condition, and distinguishes blockers from notes.

Decision tier: **Critical** — this choice governs durable receipt recovery correctness and whether an uncertain command can be replayed.
