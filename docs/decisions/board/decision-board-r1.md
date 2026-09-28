# Conductor independent expert board

Version: r1 — proposal pending initial board review

## Purpose and authority

Conductor's autonomous agents make routine implementation choices themselves. A small, independent expert board makes material product, architecture, privacy, security, reliability and platform decisions on the owner's behalf, within the user's stated goal and constraints. A board decision authorizes project design and implementation work; it does not authorize external spending, production access, publication, deployment, or release.

The user's explicit task and constraints remain the product direction. Platform, system, security and repository protections outrank a board vote. The facilitator is a non-voting coordinator: it frames the decision, selects relevant seats, verifies cited claims against primary project sources, records the result and implements the board's decision.

## When to convene the board

The task agent proceeds without a board for routine, reversible implementation choices, ordinary bug fixes, documentation, focused refactors, and decisions already settled by SPEC.md, contracts, an ADR, or a prior valid board record.

Convene the board before committing to a new or materially changed decision that affects product behavior, public protocol/API, data ownership or retention, architecture boundaries, supported platforms, security/privacy posture, execution authority, recovery guarantees, release criteria, or long-lived project direction. Do not ask the user to settle these choices. If a governing source already decides the question, follow it and do not reopen it casually.

## Seats and independence

- Use three seats for material but reversible decisions: the relevant product/domain expert, a systems/platform expert, and an adversarial reviewer.
- Use five seats for security-, privacy-, authority-, recovery-, release-, or cross-platform-critical decisions: domain/product, systems architecture, security/privacy, platform/reliability, and adversarial reviewer. Add a product/UX or data specialist when that is the main uncertainty, replacing the least relevant seat.
- Each reviewer gets one narrow lens, a fresh isolated context, the same versioned artifact and no other reviewers' rationales. The adversarial seat must argue the strongest case against the proposal and identify what would make it fail.
- Use different model families/tools when available. Record each seat's role and model/tool. Separate contexts are useful but same-model agents are correlated; never describe seat count as statistical proof of independence or as distinct human/GitHub reviewers.
- Reviewers return `BUILD`, `CONDITIONAL`, or `REJECT`, with numbered blockers, file/section citations, a closure condition, confidence, and any checks performed. No citation means no blocker. Reviewers do not edit the proposal or each other's records.

## Decision rules

1. **Routine:** the task agent selects the safest reversible option, records material assumptions, verifies, and continues.
2. **Material/reversible:** three seats review the options. Adopt an option when at least two seats recommend it and no reviewer identifies an unclosed safety or normative conflict. Record dissent and the reason the selected option wins.
3. **Critical/sensitive:** five seats review a written proposal before implementation. Require all five `BUILD` votes and closure of every material blocker before adopting any option that expands authority, trust, data exposure, external effects, recovery risk, or release capability. A board vote is a design gate, never proof of production security or platform qualification.
4. **After a conditional/reject:** the facilitator verifies each claim against live repository sources, logs it as confirmed/partly confirmed/refuted/unverified with evidence, revises the artifact, increments its version, and asks reviewers to verify prior closures by quoting the new text. Reviewers may raise new blockers only for defects introduced by the revision.
5. **No user escalation for a split vote:** allow at most three rounds. At the limit, a material/reversible choice defaults to the safest reversible option supported by at least two seats; if none exists, defer only that decision. A critical/sensitive proposal without unanimous `BUILD` is rejected or deferred, and the affected capability remains unavailable. Record the unresolved point and continue independent safe work. The board must not turn its inability to agree into a routine request for the user to decide.

The decision record must state which rule applied, the vote, dissent, the evidence that closes blockers, the chosen option, rejected alternatives, implementation consequences, and any deferred gate. A `CONDITIONAL` vote is not approval until its conditions are verified. Do not relitigate settled items without quoting a concrete gap in the current artifact.

## Records and implementation

Record material decisions in a concise Markdown file under `docs/decisions/board/`, with a date/title, artifact version and SHA-256, decision tier, alternatives, board seats/models, isolated verdicts, blocker adjudications, final ruling, dissent, evidence and implementation follow-up. Link it from `docs/decisions.md` when it changes a durable architecture or product decision. Routine decisions need no board record.

Board review is for plans, specifications and consequential choices before implementation. Use focused code review for code diffs. After a board decision, the implementation agent runs `pnpm verify`, reviews the diff, commits atomically and synchronizes through the active main ruleset without asking for routine user approval.

## Boundaries

The board may choose among implementation approaches, set conservative product defaults, reject a risky proposal, or defer a capability. It cannot invent user authorization, waive an explicit user constraint, fabricate evidence, create reviewer identities, disable active repository protections, or certify security by unanimity. Keep capabilities disabled when a specification, independent qualification, credential, account-owner action or release prerequisite is genuinely missing; continue all safe unrelated work.

No board member or facilitator may use production credentials, spend money, provision live infrastructure, send third-party messages, publish, deploy, submit to an app store, or make irreversible data/permission changes unless the user explicitly authorized that specific external action. If such authorization is absent, the board can prepare and verify the change locally, then leave only the external action pending.
