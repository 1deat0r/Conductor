# Round 2 — independent security and governance review

**Verdict: APPROVE for implementation readiness. Blocking findings: none.**

- Reviewer: independent security agent `/root/security_review`
- Reviewed SHA-256: `bc423cc2df0c3cf2ee1c663d8a47bbfd9ade3374630d16ef085648f5b4b8c598`
- Scope: all 15 files named by the expanded `docs/reviews/scope.json`, including R25, governance instructions, GitHub workflow, CODEOWNERS and ruleset configuration.
- This is a new review of the expanded bundle, not a carried-forward approval. I computed the scope hash independently twice; both results matched. I did not edit normative/implementation files or earlier verdicts.

## Product security requirements

**R09/R15/R16 now state enforceable distinctions.** Trusted execution inherits account-accessible credentials, sockets, files and network authority; it no longer promises the exclusions intended for restricted/managed workloads. Strong publication/failover claims require an independently enforced inaccessible credential/policy/operation-store boundary, not voluntary broker usage by a same-user CLI. The shell-approval language correctly grants only the disclosed profile authority. Threat-model and qualification-gate language are aligned with these distinctions.

**R06–R14, R17–R24 remain ready to implement.** I rechecked identity separation, canonical input binding, fresh approval authority, durable deduplication, active-stream revocation, offline time uncertainty, provider in-flight effects, restore generations, artifact authorization/provenance, hostile Git and preview handling, secret storage and update trust. No new contradictory guarantee was introduced. The R18 gate explicitly requires cross-tenant access and URL-expiry testing before S2 team artifact exposure, resolving SEC-N02. These remain requirements and future falsification gates, not assertions that S0 implements isolation, encryption, enrollment or effects.

## New GitHub governance requirements

**R25 defines a usable trust model.** Issue/isolated-branch/PR development, accountable integration, independent parallel role review, exact-current-head records, stale-review refresh and normative scope-hash approvals are distinct obligations. Shared-account agents are expressly not separate GitHub approving principals. An eligible non-author native GitHub review is additionally required; an unavailable eligible reviewer leaves the PR unmerged. Unsigned JSON does not purport to authenticate independence, and administrator/settings trust is disclosed. This avoids confusing local integrity checks with authoritative external approvals.

**The proposed main ruleset matches the stated mechanics.** It is active for main, has no bypass actors, requires PRs, stale dismissal, latest-push approval, one eligible approving review, resolved threads, strict current-base checks, squash-only merging, linear history and no deletion/force push. CODEOWNERS is accountability routing rather than an assertion that the owner can approve their own PR. The one-time bootstrap exception is bounded to the initial seed after unanimous specification approval, followed immediately by applying/reading back protection and inspecting CI; it cannot justify later direct-main fixes. Repository creation, protection installation and CI have not happened in this review.

I checked the rule semantics against [GitHub rules documentation](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-rulesets/available-rules-for-rulesets) and [the rules REST API](https://docs.github.com/en/rest/repos/rules). The local policy intentionally remains stricter about agent-review evidence than native GitHub enforcement alone. Successful API read-back and actual check/review states remain bootstrap/merge gates.

**CI separates untrusted work from release authority.** Its token is read-only, checkout does not persist credentials, action references are immutable object IDs, dependencies use frozen locks, and PR execution is not combined with `pull_request_target` privilege. Signing/publication are separate. I independently queried GitHub and confirmed app ID `15368` identifies `github-actions`; both checkout and setup-node pins resolve to upstream commits. One pin refinement is noted below. The prior mutable-major-tag risk in SEC-N01 is removed.

## Nonblocking note

**SEC-N03 — Use the peeled commit for pnpm/action-setup.** Affected requirements: R23/R25 and the workflow's full-commit-SHA convention. `f40ffcd9367d9f12939873eb1018b921a783ffaa` is an annotated tag object, not a commit. Independent GitHub API results show `git/tags/f40ffcd9367d9f12939873eb1018b921a783ffaa` points to commit `b906affcce14559ad1aafd4ab0e942779e9f58b1`; `contents/action.yml?ref=f40ffcd9367d9f12939873eb1018b921a783ffaa` resolves successfully, while the commits endpoint rejects that tag-object ID. Prefer the verified peeled commit in the workflow so tooling and audit statements consistently identify a commit. Failure case: tooling that validates a commit-only pin rejects this object even though content lookup succeeds. This does not presently undermine immutable content identity and is not a claimed exploit or proven CI failure, so it does not block design approval. Any change to this normative workflow still requires approval of the resulting new scope hash.

## Verification and limits

`pnpm verify` passed, including the newly incorporated Rust formatting and all-target clippy checks plus Rust tests. TypeScript checking/tests/build used Turbo cache; I do not represent that as new adversarial qualification. `pnpm check:approvals` returned ENOENT because `docs/reviews/approvals.json` has not yet been assembled. This review is one individual approval, not a claim of unanimous approval or permission to create the public repository before the other required final approvals exist.

No production readiness, remote CI success, active repository protection, native-device qualification or implemented sandbox/credential guarantees are certified by this verdict.
