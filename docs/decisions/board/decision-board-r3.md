# Conductor independent expert board

Version: r3 — proposal text; adoption decision and review criteria: [`decision-board-r3-proposal.md`](../decisions/board/decision-board-r3-proposal.md)

This file is Conductor's complete, repository-controlled board procedure. It does not depend on an optional installed skill or a particular agent orchestration API. Future agents may use an available multi-agent tool, following the [review prompt contract](decision-board-review-prompt.md).

## Purpose and authority

Conductor's autonomous agents handle routine work. An independent expert-agent board decides material product, architecture, privacy, security, reliability, platform and long-horizon choices on the user's behalf, within the user's goal and explicit constraints. Do not ask the user to choose between product or engineering alternatives; the board owns those decisions.

The user's explicit task and constraints remain the product direction. Platform, system, security and repository protections outrank a board vote. The facilitator is a non-voting coordinator: it frames options, classifies the decision, selects seats, checks access before dispatch, verifies claims against primary sources, records the result and implements the board's decision.

A board decision authorizes project design and implementation within the task. It does not authorize external spending, production access, publication, deployment, messaging, or release. It is not security, legal, or platform qualification.

## Decision classification

Classify the complete proposed change by its highest applicable consequence. When classification is uncertain, use the higher tier.

1. **Critical:** the decision changes authorization, trust boundaries, data exposure/retention, credentials, privacy/security posture, externally visible effects, recovery correctness, supported execution authority, production/release capability, or a security/release gate. Use five seats and the critical vote rule.
2. **Material:** it changes product behavior, public API/protocol, data model, architecture boundaries, supported platform commitments, long-lived roadmap, or repository governance, but does not meet a critical criterion. Use three seats and the material vote rule.
3. **Routine:** it is a reversible implementation detail, ordinary bug fix, focused refactor, documentation change, or a choice already settled by SPEC.md, contracts, an ADR, or a valid prior board record. The task agent decides, records any material assumption, verifies, and continues.

When a proposal includes multiple choices, classify and vote on each separately; the highest tier does not force unrelated low-risk choices into the same decision.

## Convening the board

- **Material board:** three mandatory voting seats — the relevant domain/product expert, a systems/platform expert, and an adversarial reviewer.
- **Critical board:** five mandatory voting seats — domain/product, systems architecture, security/privacy, platform/reliability, and adversarial reviewer. Keep all five roles on every critical board. A specialist in the central uncertainty may be added as a sixth voting seat or as an adviser; it never replaces a mandatory seat. If the specialist votes, it is included in the unanimous adoption threshold. Advisers report findings but do not vote.
- Each seat receives a narrow lens, a fresh isolated agent context, the same versioned artifact and options, and no other seats' rationales or votes. The facilitator does not vote. Use different model families/tools when permitted and available; record the model/tool per seat. Same-family agents are correlated, so seat count is not statistical proof of independence or distinct human/GitHub identities.
- Before dispatch, classify the material and check that every chosen model/tool is approved to receive it under SPEC R20/R21 and its access label/residency. Send only the minimum redacted excerpt needed for each lens. Board diversity never authorizes a new provider or data transfer. If approved reviewers cannot receive the material, use authorized local evidence or defer the affected decision; do not lower the quorum or expose protected content.
- Use the orchestration tool available in the current environment. In Codex, start isolated reviewers with `collaboration.spawn_agent` and a fresh context. If the tool's concurrency limit is lower than the seat count, dispatch independent waves against the same immutable artifact hash. Later seats in the same round receive no earlier verdicts. Never edit the reviewed artifact while any seat in that round is active. If any mandatory seat or assigned voting specialist cannot be recruited, the board has no quorum and immediately defers the decision; do not start a partial round or count a later terminal round against the three-round limit. Never substitute the facilitator or a user vote.
- Each proposal defines stable option IDs and groups mutually exclusive alternatives into one choice set; independent choices are split into separate decisions. A reviewer may return `BUILD` for at most one option in a mutually exclusive choice set. Each voter returns a verdict for every option (`BUILD`, `CONDITIONAL`, or `REJECT`), a preferred option/ranking or `NONE`, numbered blockers with precise citations and closure conditions, confidence, and checks performed. No citation means no blocker. A `CONDITIONAL` or `REJECT` never counts as `BUILD`; rankings preserve preference information but do not change the quorum or break a tie. A ballot that gives multiple `BUILD` votes in one mutually exclusive choice set or omits an option verdict is invalid: the facilitator may request one correction privately from that seat without disclosing other votes; a second invalid ballot means no quorum and immediate deferral. Reviewers do not edit artifacts, vote on another seat's findings, or send external messages.

## Vote and terminal rules

The facilitator verifies each claimed blocker against governing requirements and primary evidence. A blocker is **confirmed** only when the cited requirement/evidence applies to the option and the proposed option violates it. A confirmed blocker disqualifies that option until the text or evidence closes it. A newly evidenced conflict may be raised in any adoption-pending round, including one in unchanged text; the reviewer must cite it and explain why it was not raised earlier. A settled item is reopened only when its resolving text is absent or new material evidence changes the ruling.

| Tier | Adoption threshold | If not met |
|---|---|---|
| Material | Adopt an option when at least 2 of the 3 mandatory voting seats return `BUILD` for that option and it has no confirmed open blocker. | Revise and re-vote, up to three total rounds. If no option meets the threshold in round 3, no option has a complete vote, or a required seat is unavailable, defer only that decision and continue safe unrelated work. Do not break a tie by facilitator preference or user vote. |
| Critical | Adopt an option only when all 5 mandatory seats and every assigned voting specialist return `BUILD` for that same option and every material blocker is closed. | Revise and re-vote, up to three total rounds. If no option meets the threshold in round 3, reject/defer the proposal, keep the affected capability disabled, and continue safe unrelated work. No user escalation or approval substitution. |

Rounds 1 and 2 use the stated adoption threshold. The terminal rule applies only after round 3. The facilitator records all dissent. Board inability to agree never becomes a request for the user to make the engineering choice. A missing factual input is handled through repository evidence, conservative scope, or a documented deferred item; the user is not asked to approve a board outcome. The Critical rule is unanimity; no ranking, plurality, tie-break, or facilitator judgment can substitute for a missing `BUILD` vote.

## Round protocol and durable evidence

1. Write a concise proposal with the goal, constraints, acceptance criteria, mutually exclusive choice sets and stable option IDs, decision tier, and evidence needed. Save the exact reviewed bytes and compute SHA-256 before dispatch.
2. Dispatch every required seat using the prompt contract. Keep each seat isolated; use waves only when concurrency requires them. Collect all verdicts before adjudicating, and do not edit the proposal while the round is in flight.
3. Check claims against the live repository/source. Record each blocker verbatim with its citation, check/result or reason no check applies, ruling (`CONFIRMED`, `PARTLY`, `REFUTED`, `UNVERIFIED`), closure text/evidence, and the reviewer/model/tool.
4. For another round, update the proposal, increment its version, save an immutable snapshot of the prior version, and compute a new hash. Prior seats quote their resolving text and return `PASS`/`FAIL`; a cold-read seat in the final round reads the current version without prior rationales. New evidence-backed blockers remain allowed under the rule above.
5. Record the final vote, winning option or terminal defer/reject, dissent, implementation consequences, and links to all reviewed snapshots. A pending or uncommitted proposal is not active policy. After the board approves the exact normative bytes, run `pnpm verify`, review the diff, and atomically commit those same bytes; record the commit SHA as the adoption event before future agents treat the policy as active. Non-normative outcome metadata may be recorded separately, but must not alter the approved normative bytes. If verification or commit cannot preserve the approved bytes, leave the proposal unadopted and reconvene the board for any content change. Synchronize under the active ruleset.

Keep the active procedure here and individual decision records under `docs/decisions/board/`. Each record stores artifact snapshots or immutable Git references and hashes, exact reviewer prompts, seat role/model/tool, isolated verdicts and short rationales (never hidden chain-of-thought), blocker adjudications, quote-based closures, votes, dissent, final ruling, and implementation/evidence links. Link records for durable architecture/product choices from `docs/decisions.md`. Routine decisions need no board record.

## Boundaries

The board may choose an implementation approach, select a conservative product default, reject a risky proposal, or defer a capability. It cannot waive explicit user constraints, invent external authorization, fabricate evidence, create reviewer identities, disable repository protections, or claim that unanimity proves security. Keep capabilities unavailable when specifications, independent qualification, credentials, or actual release prerequisites are missing; continue safe unrelated work.

No board member or facilitator may use production credentials, spend money, provision live infrastructure, send third-party messages, publish, deploy, submit to an app store, or make irreversible data/permission changes unless the user explicitly authorized that specific external action. The board can prepare and verify the local change, then leave only the external action pending.
