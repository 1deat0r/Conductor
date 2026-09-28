# Expert-board adoption record: Conductor autonomy policy

## Decision

Status: ACTIVE. The Critical expert board unanimously approved the exact normative policy bytes; implementation commit `971265461313968ecdf009e3d82bdc95b646007e` records activation.

- Option: ADOPT-R3 — adopt the expert-board procedure and integrated Conductor workflow.
- Tier: Critical. The policy governs decision paths affecting privacy, security, authority, platform support, and release controls.
- Outcome: BUILD from all five mandatory Critical roles and the assigned sixth agent-operations voting specialist; no open blockers.
- Facilitator: non-voting.
- User input: none used to settle the policy decision. The user explicitly asked for a board of independent expert agents and authorized continuation.
- Scope: public repository policy. GitHub visibility was checked as PUBLIC. The review bundle contained no secrets or personal data.

## Reviewed immutable bundle

Aggregate SHA-256: 347fb34e2cec36037e28dbeb99548931d2683e1f5f835c902ad03d31917ac1a7

The aggregate was computed by hashing the ordered output of sha256sum over these paths:

| Path | SHA-256 |
|---|---|
| AGENTS.md | 115647377b53e5713428a2e037add40b923ce587d3d038add770d5dfe505c359 |
| CONTRIBUTING.md | f5402c9d3486000d8cfd0cd0cda57bb8f44b4a45c9eae07872c04a9af67083ff |
| SPEC.md | 95004071158164f03b3f00308b9fbf071ae8987cea858ab3e6141da3c742e73c |
| STATE.md | cecffb9e2e479fe60939260705f7ffcdeb94fa63d3328e0a1b900f66a2959016 |
| docs/agents/README.md | 70463015ba92f47118ac6c0805c0bea30c0829e70ebcd399d53e810f7d5fb790 |
| docs/agents/autonomy.md | 86e9eb618337759ff1610e4aa0ad43bdb041725778617104faf0b4362345b416 |
| docs/agents/domain.md | 1fcba01455578a52b0a484d41ad9e6592a16949b8efeca1e20153a8acca9b98b |
| docs/agents/triage-labels.md | 0ff967cc11d384156e5268de30aa853f04304cf60face82040205bb404421f89 |
| docs/agents/decision-board.md | df280594fbaa77848f537499bf7d6136b43d20e937696d7a3612641984ced928 |
| docs/agents/decision-board-review-prompt.md | 673b03dcb4051827486107189db872b673c31e43cc3a452c0759f95737540568 |
| docs/decisions/board/decision-board-r3-proposal.md | 967abd44e7a035b05561426d1eb3f895ecb0abd2ed0b268c89ff78c5fc7f8a14 |
| docs/development-workflow.md | 9c39605216abeb61f06b900a26b2b6c52e4ffc98fc7b3bc62e9e2704497a4d8b |
| docs/decisions.md | 0c0ee7a7f19a4d1721778e675a39e7ff2ca545ac0920e5089a32c3bfa3c74e8f |
| docs/implementation-plan.md | 8370ca6b30ffad7a7271059f6e855c352cbbf20b1cce8f7292a8aae282ee1c84 |

Procedure snapshot: docs/decisions/board/decision-board-r3.md
Procedure SHA-256: df280594fbaa77848f537499bf7d6136b43d20e937696d7a3612641984ced928
Proposal: docs/decisions/board/decision-board-r3-proposal.md
Proposal SHA-256: 967abd44e7a035b05561426d1eb3f895ecb0abd2ed0b268c89ff78c5fc7f8a14

## Critical board composition and votes

| Seat | Model | Context | R3 vote | Closure |
|---|---|---|---|---|
| Domain/product | gpt-5.6-terra | Isolated re-review; received only its own DOM-1 finding | BUILD | DOM-1 PASS |
| Systems/governance | gpt-6-astra | Isolated re-review; received only SYS-1..3 | BUILD | SYS-1..3 PASS |
| Security/privacy | gpt-6-sol | Isolated re-review; received only SEC-1 | BUILD | SEC-1 PASS |
| Platform/reliability | gpt-6-astra | Fresh cold read; no earlier findings or votes | BUILD | No blockers |
| Adversarial | gpt-6-luna | Isolated re-review; received only ADV-1..4 | BUILD | ADV-1..4 PASS |
| Agent operations specialist | gpt-5.6-sol | Additional sixth voting seat; isolated re-review; received only OPS-1..4 and OPS-N1 | BUILD | OPS-1..4 and OPS-N1 PASS |

Each context independently read the same 14-file bundle. The six seats were dispatched in three waves because of orchestration concurrency limits. Earlier votes were not sent to later seats. The facilitator did not vote. Models share a platform; separate contexts and model variants are useful review diversity, not proof of statistical independence or separate human/GitHub identities.

## Findings and adjudication

R1 issues were recorded in 2026-09-29-autonomy.md. R2 review is recorded in 2026-09-29-autonomy-r2.md. R3 closed every previously open item:

| Finding | R3 closure evidence | Result |
|---|---|---|
| SYS-1: total choices, ballots and quorum | Systems quoted: “A reviewer may return BUILD for at most one option in a mutually exclusive choice set”; a missing mandatory or assigned voting seat “immediately defers the decision”; one invalid ballot may be corrected privately, and a second invalid ballot means no quorum. All five Critical roles remain mandatory. | PASS |
| SYS-2: newly evidenced blockers | Systems quoted: “A newly evidenced conflict may be raised in any adoption-pending round, including one in unchanged text”; settled items reopen only when resolving text is absent or new material evidence changes the ruling. | PASS |
| SYS-3: user choice path | Systems verified AGENTS.md says: “The board decides for the user; never route product choices, dissent or a split vote back to the user.” SPEC R25 and the workflow/autonomy adapter say the same. | PASS |
| OPS-1: discovery and durability | Operations quoted: “A pending or uncommitted proposal is not active policy.” Activation requires verification and an atomic commit of “those same bytes” with the commit SHA recorded. AGENTS.md, README and workflow link the procedure. | PASS |
| OPS-2: orchestration availability | Operations verified fresh isolated collaboration.spawn_agent contexts, fixed-hash concurrency waves, no earlier-vote disclosure, and immediate no-quorum deferral if a seat is unavailable. | PASS |
| OPS-3: total decision rules | Operations verified ordered tiers, one BUILD maximum per mutually exclusive option set, a single correction attempt for invalid ballots, unanimity for Critical decisions and bounded deferral without user escalation. | PASS |
| OPS-4 / OPS-N1: durable evidence | Operations verified snapshots/hashes, exact prompt storage, isolated verdicts, adjudications, closures and final commit reference under docs/decisions/board/. | PASS |
| DOM-1: mandatory role coverage | Domain quoted: “Keep all five roles on every critical board. A specialist ... never replaces a mandatory seat.” Specialists are an additional voting seat or adviser. | PASS |
| SEC-1: reviewer-data authorization | Security quoted the pre-dispatch approval and minimum-redaction rule and confirmed unavailable reviewers cause local evidence use or safe deferral. | PASS |
| ADV-1..4: escalation, data, revisions, seat safety | Adversarial verified no user tie-break, explicit external authorization boundaries, new-evidence blockers in any pending round, and permanent security/adversarial seats. | PASS |
| Platform/reliability cold read | No blockers. The reviewer checked that exact-commit Ubuntu/Windows/macOS CI remains independent, native platform and release qualification remain separate, and blocked work can defer safely. | BUILD; no findings |

No new blocker was raised in R3. No unresolved condition or dissent remains.

## Exact prompt capture

The common reviewer contract is docs/agents/decision-board-review-prompt.md. The full filled messages are preserved in 2026-09-29-autonomy-r3-prompts.md. Every prompt identified ADOPT-R3, the aggregate and individual hashes above, the ordered 14-file list above, the public-content classification, the assigned lens, isolated/no-edit/no-contact/no-user-question rules, criteria, response shape, and Critical unanimity rule. Re-review prompts included only that seat's prior blockers; the platform/reliability cold-read prompt included none.

Seat-specific R3 prompt instructions were:

- Domain/product: re-review only DOM-1; quote the all-five-mandatory-role closure and report any newly evidenced blocker.
- Systems/governance: re-review only SYS-1..3; check option-set ballots, invalid ballot handling, missing-seat timing, workflow consistency, and activation.
- Security/privacy: re-review only SEC-1; check provider authorization/access label/residency and confirm additions do not weaken security gates.
- Platform/reliability: cold read all 14 files; check cross-platform CI, native qualification, release evidence, deferral, and activation.
- Adversarial: re-review only ADV-1..4; stress user escalation, routing, revision, mandatory seats, and authority boundaries.
- Agent operations specialist: re-review only OPS-1..4 and OPS-N1 as an additional voting specialist; verify discoverability, uncommitted-draft behavior, orchestration, quorum, records, and immutable references.

No hidden chain-of-thought is recorded. The concise final votes and closure statements are preserved above; all reviewers reported high confidence and no new blocker.

## Adoption gate and limits

This board approves the policy design. CI=true pnpm verify passed on the exact policy working tree: spec structure, TypeScript checks, package tests/builds, Rust formatting, Clippy, and Rust tests all succeeded. A bare pnpm verify first stopped before checks because pnpm needed a noninteractive confirmation to recreate the ignored node_modules directory; CI mode safely completed that reset from the committed lockfile. The complete diff was reviewed, and the exact reviewed normative bytes were atomically committed as `971265461313968ecdf009e3d82bdc95b646007e`; that SHA is the activation event. This record's status and activation-SHA updates are outcome metadata and do not alter the reviewed policy rules. The initial commit's cross-platform required checks remain the synchronization gate for protected main.

This is governance-document review only. It is not runtime verification, human security qualification, legal advice, native-platform qualification, release authorization, or proof that agents are independent. The board cannot authorize external spending, production access, publication, deployment, messaging, or irreversible external changes.
