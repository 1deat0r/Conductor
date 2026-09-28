# Adoption proposal: Conductor independent expert board

Version: r3 proposal. It is not effective until approved and committed under the procedure it proposes.

- **Option:** `ADOPT-R3` — adopt revision r3 of `docs/agents/decision-board.md` and its linked Conductor workflow integration as the default for autonomous product and engineering decisions.
- **Tier:** Critical, because the policy governs decisions affecting security, privacy, authority and release controls.
- **Acceptance criteria:**
  1. Routine and material product/engineering choices do not require user tie-breaks.
  2. All five named Critical roles remain mandatory; any voting specialist is additive and joins the unanimous threshold; mutually exclusive option sets, votes, revision and terminal outcomes are executable and unambiguous.
  3. Reviewers receive only content permitted by the repository's access label, provider authorization and residency rules.
  4. The facilitator does not vote and seats review in isolated contexts without seeing other votes.
  5. Board decisions cannot waive system, security, platform, repository, external-authorization or release-evidence gates.
  6. Persistent instructions consistently point future agents to this policy.
- **Fail-safe:** If `ADOPT-R3` does not pass the Critical rule within three complete rounds, do not activate the proposal, do not ask the user to break the tie, and continue only work allowed by already-active policy and security controls.
