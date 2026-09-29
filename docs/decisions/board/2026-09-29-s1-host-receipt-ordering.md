# S1 host-receipt ordering — Critical board outcome

Date: 2026-09-29

Status: **Deferred after final round; no option adopted**
Tier: Critical

## Outcome

No option received the required unanimous BUILD vote from all five mandatory seats and the assigned cold-read voting specialist in the final round. The board therefore terminally deferred the ordering design under the Critical rule. This record does not activate either proposal option and is not an adoption event.

For the Issue #3 schema-only work, host receipts remain typed, parse-only snapshots. `updated_at_unix_ms` is not an ordering value. No consumer may project, retry, reconcile, or recover from unordered receipt snapshots. Keep S0 execution unavailable. Reopen this design only with a future journal/API proposal that includes cross-generation receipt reconciliation and durable recovery evidence; do not route the decision back to the user.

The final cold-read seat identified an unresolved cross-generation case in `HOST-REVISION`: after a consumer observed a terminal receipt in generation A, a restored generation B could provide an older receipt; generation comparison alone did not specify when to install or publish the replacement. R07 and R24 require prior-state semantics and reconciliation, but the proposal only gated retries/admissions. This blocker remained open at the end of round 3. `DEFER-TO-JOURNAL` was safe as a scope option, but it did not reach unanimity either. The terminal rule closes this proposal after three rounds; no fourth round was started.

## Scope and evidence

The decision concerned only ordering/projection semantics for `HostReceiptV1`. It did not change the receipt schemas, authorize command admission, enable retries, or prove storage durability or recovery. The worktree’s live Issue [#3](https://github.com/1deat0r/Conductor/issues/3) says storage-level durability and coordinator/client/host integration are separate non-goals for that slice. The facilitator independently read the live issue body. Reviewers assessed the issue acceptance as summarized in each immutable proposal.

The public GitHub repository and public SPEC/contracts were reviewed through local Codex collaborator contexts; no credentials, private user content, production data, or external provider routing was involved. Reviewers received only the four listed bundle files. No reviewer edited files or ran project checks. Each round used a fixed artifact hash and later seats received no earlier ballots. The cold-read reviewer received Version 3 only and did not follow earlier proposal links. Codex subagents are isolated model contexts, not independent humans or GitHub reviewers.

Review-time source hashes, unchanged across all three rounds:

- `SPEC.md`: `95004071158164f03b3f00308b9fbf071ae8987cea858ab3e6141da3c742e73c`
- `contracts/README.md`: `8366b0080628d5b0fef4878c431c251fb9621fdb5d9fb6c11efb85647c1c4808`
- `docs/agents/decision-board.md`: `df280594fbaa77848f537499bf7d6136b43d20e937696d7a3612641984ced928`

The three immutable proposals and hashes:

| Round | Artifact | SHA-256 |
|---|---|---|
| 1 | [`2026-09-29-s1-host-receipt-ordering-proposal-v1.md`](2026-09-29-s1-host-receipt-ordering-proposal-v1.md) | `8717abe7821b97cee0f42ef9e964ded918306dc9eb0b35dc3ce1f0b546784901` |
| 2 | [`2026-09-29-s1-host-receipt-ordering-proposal-v2.md`](2026-09-29-s1-host-receipt-ordering-proposal-v2.md) | `d4f6c9a9481d3cb88960d615c81ed521dd8b6743ec865aa229c755f177e358c3` |
| 3 | [`2026-09-29-s1-host-receipt-ordering-proposal-v3.md`](2026-09-29-s1-host-receipt-ordering-proposal-v3.md) | `1d85880b8ca4cb1c819d110d9d94105f8b9eab8803d62aacb3c381a67b3acda5` |

Exact prompts, seat roles, and model assignments are preserved in [`2026-09-29-s1-host-receipt-ordering-prompts.md`](2026-09-29-s1-host-receipt-ordering-prompts.md).

## Critical board votes

### Round 1 — Version 1

| Mandatory seat | Model/tool | `HOST-REVISION` | `DEFER-TO-JOURNAL` | Concise rationale |
|---|---|---|---|---|
| Product/domain | gpt-6-astra, Codex subagent | BUILD (1) | CONDITIONAL (2) | Preferred to define ordering and transitions now; deferral was a safe but less useful fallback. |
| Systems architecture | gpt-6-sol, Codex subagent | CONDITIONAL (2) | BUILD (1) | A revision can order observations, but the draft lacked atomic writer serialization and restore fencing; parse-only deferral was safe. |
| Security/privacy | gpt-5.6-sol, Codex subagent | CONDITIONAL (2) | BUILD (1) | Counter rollback without a non-reused authenticated recovery generation could accept stale receipts. |
| Platform/reliability | gpt-6-luna, Codex subagent | CONDITIONAL (2) | BUILD (1) | Restore can roll back the counter; deferred parse-only snapshots avoid cross-platform recovery claims. |
| Adversarial | gpt-5.6-terra, Codex subagent | CONDITIONAL (2) | BUILD (1) | A restored/cloned host could reuse an old counter and process an at-least-once duplicate. |

No option met the Critical unanimity threshold.

### Round 2 — Version 2

| Mandatory seat | Model/tool | `HOST-REVISION-FENCED` | `DEFER-TO-JOURNAL` | Concise rationale |
|---|---|---|---|---|
| Product/domain | gpt-6-astra, Codex subagent | BUILD (1) | CONDITIONAL (2) | Fencing made the design coherent; preferred to define semantics now. |
| Systems architecture | gpt-6-sol, Codex subagent | BUILD (1) | CONDITIONAL (2) | Atomic state/revision/outbox updates and restore fencing closed the prior blocker. |
| Security/privacy | gpt-5.6-sol, Codex subagent | CONDITIONAL (2) | BUILD (1) | Technical recovery blocker closed, but the option ID changed from Version 1, violating stable-ID procedure. |
| Platform/reliability | gpt-6-luna, Codex subagent | BUILD (1) | CONDITIONAL (2) | Fenced revisions were implementable; deferral remained safe until a future consumer exists. |
| Adversarial | gpt-5.6-terra, Codex subagent | BUILD (1) | CONDITIONAL (2) | Fenced generation prevented restore/clone counter reuse; the defer option was safe but less useful. |

No option met the Critical unanimity threshold.

### Round 3 — Version 3 plus cold-read specialist

| Voting seat | Model/tool | `HOST-REVISION` | `DEFER-TO-JOURNAL` | Concise rationale |
|---|---|---|---|---|
| Product/domain | gpt-6-astra, Codex subagent | BUILD (1) | CONDITIONAL (2) | Current-generation semantics were coherent; explicit deferral was safe but leaves consumers for later. |
| Systems architecture | gpt-6-sol, Codex subagent | BUILD (1) | CONDITIONAL (2) | Atomic revisions and R24 fencing close the identified stale-writer and restore paths. |
| Security/privacy | gpt-5.6-sol, Codex subagent | CONDITIONAL (2) | BUILD (1) | Stable ID and generation blockers were closed; preferred leaving receipt consumption to the journal/API contract. |
| Platform/reliability | gpt-6-luna, Codex subagent | REJECT (2) | BUILD (1) | The journal/API should own these semantics; deferral was safe for schema-only scope. |
| Adversarial | gpt-5.6-terra, Codex subagent | BUILD (1) | CONDITIONAL (2) | Fencing closed restore/clone attacks; deferral was safe but postponed useful consumer semantics. |
| Cold-read recovery specialist | gpt-6-astra, Codex subagent | CONDITIONAL (2) | BUILD (1) | A generation switch could install a stale terminal receipt without a reconciled baseline; parse-only deferral was safe. |

The final tally was 3 BUILD, 2 CONDITIONAL, and 1 REJECT for `HOST-REVISION`; and 3 BUILD plus 3 CONDITIONAL for `DEFER-TO-JOURNAL`. Neither option met unanimity.

## Blocker adjudication

| Finding | Ruling | Evidence and closure |
|---|---|---|
| Version 1 `HOST-REVISION` had no non-reused recovery generation; a restored/cloned host could reuse a counter. | CONFIRMED; closed in v2 and v3 | SPEC R24, `SPEC.md:125`; acceptance criteria 2–3. V2/v3 bind receipts to an authenticated non-reused generation, fence old writers, write-freeze restores, invalidate grants, and reconcile before retry/admission. |
| Version 1 did not require atomic revision/state/outbox serialization or stale-writer fencing. | CONFIRMED; closed in v2 and v3 | SPEC R07/R24, `SPEC.md:47,125`; criterion 2. V2/v3 make the state, revision, and outbox one atomic transition under the sole fenced host writer. |
| Version 2 renamed `HOST-REVISION` to `HOST-REVISION-FENCED`. | CONFIRMED; closed in v3 | Board policy requires stable option IDs (`docs/agents/decision-board.md:32,47-50`). V3 restores `HOST-REVISION` while retaining the fencing text. |
| Version 2's `DEFER-TO-JOURNAL` did not clearly resolve whether Issue #3 had selected deferral as its scope. | PARTLY CONFIRMED; clarified in v3 | V3 explicitly states that choosing deferral settles the present scope and updates acceptance criteria to prohibit all receipt consumers until the later gate. The adversarial seat passed its own prior blocker on this text. |
| Version 3 `HOST-REVISION` does not specify safe receipt installation across host-generation changes. A new generation could replace a previously observed terminal receipt with state restored from an older snapshot. | CONFIRMED; OPEN at terminal round 3 | Cold-read specialist cited SPEC R07/R24 and v3 acceptance criterion 2. Closure requires retaining last-known state, forbidding regressive replacement/publication during recovery, and requiring an authenticated reconciled baseline preserving command identity, digest, and terminal/unknown outcomes before switching generations. No proposal option received final unanimous BUILD; terminal rule defers this design. |

The final-round `DEFER-TO-JOURNAL` option had no blocker and was safe while no receipt consumer exists, but only three of six seats returned BUILD. It is not adopted by this record.

## Implementation boundary

- Continue Issue #3's shared schemas, fixtures, and TypeScript/Rust validators as parse-only protocol infrastructure.
- Treat `updated_at_unix_ms` as descriptive, never as state ordering.
- Do not wire a receipt projection, retry, reconciliation, recovery, or admission consumer from these snapshots.
- A future consumer design must return with evidence for authenticated non-reused host generation, cross-generation reconciliation that preserves terminal/unknown outcomes, atomic state/outbox sequencing under the sole writer, stale-writer fencing, and R24 write-freeze/recovery behavior.
- Keep `execution_available=false`; no API/IPC route, host admission, process launch, tool authorization, credentials, or repository mutation is enabled by this schema work.

The final `pnpm verify` gate passed after the typed-contract, replay-vector, and README/index documentation edits. No adoption commit SHA exists because no normative option was adopted.
