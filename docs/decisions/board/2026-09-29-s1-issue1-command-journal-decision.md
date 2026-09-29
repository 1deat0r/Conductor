# S1 Issue #1 journal boundary — Critical board outcome

Date: 2026-09-29
Tier: **Critical**
Status: **Adopted — `SEALED-STORE`**

## Outcome

All five mandatory seats returned `BUILD` for `SEALED-STORE` on the same Version 1 proposal. No material blocker was reported. The Critical unanimity threshold is met. All five seats returned `CONDITIONAL` for `ORDERED-RECEIPTS` and `DEFER-HOST-JOURNAL`; those alternatives were not adopted. The conditional votes are not blockers to the selected option.

Adoption chooses a bounded internal storage foundation: validated canonical `CommandV1` identity, no production host-receipt snapshot or admission state projection, no host outbox consumer, and a separate supervisor session store whose start marker cannot authorize process launch. `execution_available` remains false. The full Issue #1 requirements, R07 host acknowledgement semantics, R24 recovery evidence, native lifecycle qualification, and execution gates remain incomplete.

The prior terminal deferral of host-receipt ordering remains active and unchanged. This outcome does not adopt an ordering design, implement generation fencing, allow receipt projection/retry/reconciliation/recovery, or re-open the prior three-round vote.

## Reviewed artifact and evidence

- Proposal: [`2026-09-29-s1-issue1-command-journal-proposal-v1.md`](2026-09-29-s1-issue1-command-journal-proposal-v1.md), SHA-256 `dbdf63bb82c5624a7a7e04e12973862d9f25d12365dbd62b746b66691589b6c2`. The committed proposal bytes are copied exactly from the reviewed artifact.
- Exact filled prompts: [`2026-09-29-s1-issue1-command-journal-prompts.md`](2026-09-29-s1-issue1-command-journal-prompts.md).
- Fixed source revisions: current `main` `3b8cc8eb985ce19ceab005de8b8aa99a37bb48ad`; existing Issue #1 branch `d7b3f7314b1ac00065ae7a9363e69080492442e0`. The proposal embeds Issue #1 scope and acceptance relevant to this decision.
- The facilitator checked the live Issue #1 and PR #2, current `main`, the prior terminal outcome, the current typed contracts, and the existing branch excerpts. The board bundle is public repository/specification/contract material only. Reviews used local Codex subagent contexts; no credentials, production data, private user content, or additional provider routing was used. These are separate agent contexts, not independent human or GitHub reviewers.
- Seats read all 13 files listed in their exact prompts, checked the proposal SHA, and did not run tests. No implementation, live authentication, disk-full, native lifecycle, or R24 recovery qualification is claimed.

### Reviewed bundle SHA-256 manifest

| Bundle file | SHA-256 |
|---|---|
| `proposal-v1.md` | `dbdf63bb82c5624a7a7e04e12973862d9f25d12365dbd62b746b66691589b6c2` |
| `spec-excerpts.txt` | `e442ecd28154cd0d7a3c2eb4089469df029ece8971059ef8ed5de21ead84cadc` |
| `contracts-readme.md` | `60dc7ea4a86aeea1d0c2ab683593a66812ad412107d53494cd03ad41b73f55df` |
| `command-v1.schema.json` | `d2a4c93f2af3f88b489c7eb8019c140f315a9b1e1af5a27696c6e37bc1b9ec8c` |
| `host-receipt-v1.schema.json` | `a281a8b6d174722476f48a9943b70c444eff46b0219211942bd4953791a7e05f` |
| `protocol-rust-excerpts.txt` | `4e2944db16f85c290a21637930db2bc740a5a40b4e71756d891918d192abb37a` |
| `receipt-ordering-outcome.md` | `84a75c10f3673681595df9c9115a9fb3fdc903158a4673366b85f2c512d0fc59` |
| `receipt-ordering-proposal-v3.md` | `1d85880b8ca4cb1c819d110d9d94105f8b9eab8803d62aacb3c381a67b3acda5` |
| `issue1-host-journal-excerpts.txt` | `f6f3533e043924d486ccd77133cbe17c249ed72222970c33a6d2b381a62c2591` |
| `issue1-session-store-excerpts.txt` | `9516eddf15b785a7c69100bde96c5beda76537279ea6c05b38431df49472dda2` |
| `issue1-evidence.md` | `0c8e6ec91cbb5cee9737213f4e529ec06f5ec241eb5673c9ce693564f8cfbd62` |
| `board-policy.md` | `df280594fbaa77848f537499bf7d6136b43d20e937696d7a3612641984ced928` |
| `review-prompt-contract.md` | `673b03dcb4051827486107189db872b673c31e43cc3a452c0759f95737540568` |

### Critical votes

| Mandatory seat | Model/tool | `SEALED-STORE` | `ORDERED-RECEIPTS` | `DEFER-HOST-JOURNAL` | Concise rationale |
|---|---|---|---|---|---|
| Domain/product | `gpt-6-astra`, Codex subagent | `BUILD` (1) | `CONDITIONAL` (2) | `CONDITIONAL` (3) | The sealed scope preserves receipt meanings and leaves Issue #1 visibly incomplete. |
| Systems architecture | `gpt-6-sol`, Codex subagent | `BUILD` (1) | `CONDITIONAL` (2) | `CONDITIONAL` (3) | Canonical commands and hidden host receipt/outbox interfaces give a safe migration path. |
| Security/privacy | `gpt-5.6-sol`, Codex subagent | `BUILD` (1) | `CONDITIONAL` (3) | `CONDITIONAL` (2) | No authority flows from stored commands or `launch_is_new`; receipt and R24 gates remain. |
| Platform/reliability | `gpt-6-luna`, Codex subagent | `BUILD` (1) | `CONDITIONAL` (2) | `CONDITIONAL` (3) | Local storage can advance without overstating untested native lifecycle support. |
| Adversarial | `gpt-5.6-terra`, Codex subagent | `BUILD` (1) | `CONDITIONAL` (2) | `CONDITIONAL` (3) | The selected option blocks stale receipt/outbox delivery from becoming authority. |

All mandatory seats confirmed the local public-material routing and proposal hash. The board returned no numbered blocker; reviewer notes on deferred options identify future prerequisites rather than conditions on the unanimously selected option.

## Adoption event

The approved normative bytes are the proposal file above. Adoption was recorded with commit `e783510a7bf91a744f68910c78a9aabda93670b4`, which commits this outcome and the exact proposal bytes after verifying the reviewed bundle hashes. Implementation may proceed only within this sealed scope; the commit does not complete Issue #1, R07/R24, native lifecycle qualification, or execution gates.
