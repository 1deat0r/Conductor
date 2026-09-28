# Conductor independent expert board

Version: r2 — submitted for independent re-review; board decision record: [`docs/decisions/board/2026-09-29-autonomy.md`](../decisions/board/2026-09-29-autonomy.md)

## Adoption decision under review

- **Option:** `ADOPT-R2` — adopt this procedure and its linked Conductor workflow integration as the default for autonomous product and engineering decisions.
- **Tier:** Critical, because this policy governs decisions affecting security, privacy, authority and release controls.
- **Acceptance criteria:** (1) routine and material product/engineering choices do not require user tie-breaks; (2) board tiers, mandatory seats, vote thresholds, revision and terminal outcomes are executable and unambiguous; (3) reviewers receive only content permitted by the repository's access label, provider authorization and residency rules; (4) the facilitator does not vote and seats review in isolated contexts without seeing other votes; (5) board decisions cannot waive system, security, platform, repository, external-authorization or release-evidence gates; and (6) persistent instructions consistently point future agents to this policy.
- **Fail-safe:** if `ADOPT-R2` does not pass the Critical rule within three rounds, do not activate the proposal, do not ask the user to break the tie, and continue only work allowed by already-active policy and security controls.

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

- **Material board:** three voting seats — the relevant domain/product expert, a systems/platform expert, and an adversarial reviewer.
- **Critical board:** five voting seats — domain/product, systems architecture, security/privacy, platform/reliability, and adversarial reviewer. A specialist in the central uncertainty may replace only the least relevant non-security, non-adversarial seat. Keep security/privacy and adversarial seats on every critical board.
- Each seat receives a narrow lens, a fresh isolated agent context, the same versioned artifact and options, and no other seats' rationales or votes. The facilitator does not vote. Use different model families/tools when permitted and available; record the model/tool per seat. Same-family agents are correlated, so seat count is not statistical proof of independence or distinct human/GitHub identities.
- Before dispatch, classify the material and check that every chosen model/tool is approved to receive it under SPEC R20/R21 and its access label/residency. Send only the minimum redacted excerpt needed for each lens. Board diversity never authorizes a new provider or data transfer. If approved reviewers cannot receive the material, use authorized local evidence or defer the affected decision; do not lower the quorum or expose protected content.
- Use the orchestration tool available in the current environment. In Codex, start isolated reviewers with `collaboration.spawn_agent` and a fresh context. If the tool's concurrency limit is lower than the seat count, dispatch independent waves against the same immutable artifact hash. Later seats in the same round receive no earlier verdicts. Never edit the reviewed artifact while any seat in that round is active. If a distinct required seat cannot be recruited, the board has no quorum; apply the terminal rule, never substitute the facilitator or a user vote.
- Every seat returns a verdict for each option (`BUILD`, `CONDITIONAL`, or `REJECT`), a preferred option/ranking or `NONE`, numbered blockers with precise citations and closure conditions, confidence, and checks performed. No citation means no blocker. A `CONDITIONAL` or `REJECT` never counts as `BUILD`; the reviewer can still rank an alternative. Reviewers do not edit artifacts, vote on another seat's findings, or send external messages.

## Vote and terminal rules

The facilitator verifies each claimed blocker against governing requirements and primary evidence. A blocker is **confirmed** only when the cited requirement/evidence applies to the option and the proposed option violates it. A confirmed blocker disqualifies that option until the text or evidence closes it. A newly evidenced conflict may be raised in any adoption-pending round, including one in unchanged text; the reviewer must cite it and explain why it was not raised earlier. A settled item is reopened only when its resolving text is absent or new material evidence changes the ruling.

| Tier | Adoption threshold | If not met |
|---|---|---|
| Material | Adopt an option when at least 2 of 3 seats return `BUILD` for that option and it has no confirmed open blocker. | Revise and re-vote, up to three total rounds. If no option meets the threshold in round 3, no option has a complete vote, or a required seat is unavailable, defer only that decision and continue safe unrelated work. Do not break a tie by facilitator preference or user vote. |
| Critical | Adopt an option only when all 5 seats return `BUILD` for that same option and every material blocker is closed. | Revise and re-vote, up to three total rounds. If no option meets the threshold in round 3 or a required seat is unavailable, reject/defer the proposal, keep the affected capability disabled, and continue safe unrelated work. No user escalation or approval substitution. |

Rounds 1 and 2 use the stated adoption threshold. The terminal rule applies only after round 3. The facilitator records all dissent. Board inability to agree never becomes a request for the user to make the engineering choice. A missing factual input is handled through repository evidence, conservative scope, or a documented deferred item; the user is not asked to approve a board outcome. The Critical rule is unanimity; no ranking, plurality, tie-break, or facilitator judgment can substitute for a missing `BUILD` vote.

## Round protocol and durable evidence

1. Write a concise proposal with the goal, constraints, acceptance criteria, options (with stable IDs), decision tier, and evidence needed. Save the exact reviewed bytes and compute SHA-256 before dispatch.
2. Dispatch every required seat using the prompt contract. Keep each seat isolated; use waves only when concurrency requires them. Collect all verdicts before adjudicating, and do not edit the proposal while the round is in flight.
3. Check claims against the live repository/source. Record each blocker verbatim with its citation, check/result or reason no check applies, ruling (`CONFIRMED`, `PARTLY`, `REFUTED`, `UNVERIFIED`), closure text/evidence, and the reviewer/model/tool.
4. For another round, update the proposal, increment its version, save an immutable snapshot of the prior version, and compute a new hash. Prior seats quote their resolving text and return `PASS`/`FAIL`; a cold-read seat in the final round reads the current version without prior rationales. New evidence-backed blockers remain allowed under the rule above.
5. Record the final vote, winning option or terminal defer/reject, dissent, implementation consequences, and links to all reviewed snapshots. Only then implement; run `pnpm verify`, review the diff, commit atomically, and synchronize under the active ruleset.

Keep the active procedure here and individual decision records under `docs/decisions/board/`. Each record stores artifact snapshots or immutable Git references and hashes, exact reviewer prompts, seat role/model/tool, isolated verdicts and short rationales (never hidden chain-of-thought), blocker adjudications, quote-based closures, votes, dissent, final ruling, and implementation/evidence links. Link records for durable architecture/product choices from `docs/decisions.md`. Routine decisions need no board record.

## Boundaries

The board may choose an implementation approach, select a conservative product default, reject a risky proposal, or defer a capability. It cannot waive explicit user constraints, invent external authorization, fabricate evidence, create reviewer identities, disable repository protections, or claim that unanimity proves security. Keep capabilities unavailable when specifications, independent qualification, credentials, or actual release prerequisites are missing; continue safe unrelated work.

No board member or facilitator may use production credentials, spend money, provision live infrastructure, send third-party messages, publish, deploy, submit to an app store, or make irreversible data/permission changes unless the user explicitly authorized that specific external action. The board can prepare and verify the local change, then leave only the external action pending.
