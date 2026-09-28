# Round 1 — independent systems review

- Reviewer role: systems
- Reviewed scope hash: `a6297a772d4e0f0f23e1bf7d9703b96d4057d2557a4fed0961c2f9f5f4cc8d29`
- Verdict: **REQUEST_CHANGES**
- Scope: every normative file in `docs/reviews/scope.json`, with `AGENTS.md` and relevant S0 source inspected. This is a new implementation-specification review, not reuse of the earlier architecture approval.

The durability and authority design is substantially ready to implement: R04 assigns writers, R07/R08 separate durable acceptance from process launch, R09 accounts for in-flight effects, R18 gates destructive cleanup on published evidence, R22 prevents partitioned budget reuse, and R24 freezes restored authority pending reconciliation. I did not find a new blocker in those protocols. One normative trust-boundary contradiction should be resolved before implementation.

## Blocking finding

### SYS-001 — R15 applies an isolation guarantee to trusted-host execution that the spec explicitly cannot enforce

**Affected requirements:** R15, R11 and R01; `docs/threat-model.md` first abuse case; implementation-plan R15 gate.

**Failure case:** R15 first defines trusted-host mode as having the effective authority of the OS account, then states without a profile qualifier that workloads cannot reach host-control sockets, policy/audit stores, device keys, SSH agents or ambient credentials. R11 and the threat model correctly admit that same-user socket ACLs do not protect against same-user programs, and S1 deliberately provides trusted execution without S3 isolation. A trusted-host agent running under the user's account can reach account-accessible resources. That implementation would satisfy the declared trusted-host profile while violating the unconditional R15 requirement. Conversely, applying all of R15's exclusions to S1 would silently replace the promised trusted execution profile with an unplanned isolation requirement.

**Required resolution:** explicitly scope resource exclusion, externally enforced egress and the isolated workload identity/broker boundary to qualified restricted/managed profiles. State that trusted-host execution inherits account-accessible resources and has no same-user protection guarantee. Keep R09's strong publication/failover guarantees contingent on an actually enforced credentialed broker boundary, not on a trusted CLI's structured adapter or willingness to use that broker. Align the threat-model and implementation-gate wording if necessary. No runtime implementation is requested by this finding; the normative profile boundary must be unambiguous.

**Why blocking now:** the first execution milestone must know which authority guarantees it is required to implement and which it must visibly disclaim. SPEC explicitly requires contradictory normative requirements to be resolved before implementation. The honest threat-model caveat should be carried into the governing requirement.

## Nonblocking implementation notes

### SYS-N01 — Make the S1 state-machine schema enumerate reconciliation transitions

**Affected requirements:** R05, R06, R08, R09.

**Failure case to guard against:** different components independently classify `outcome_unknown` as terminal or retryable, or disagree about a revised `review_ready` attempt, causing UI projection and runtime transition divergence.

**Concrete next step:** when implementing the execution schemas required by R06, include the terminal-state set and explicit authenticated reconciliation transitions in the conformance vectors. R05 already prohibits replay resurrection and automatic uncertain relaunch, so this is ordinary implementation detail rather than a new design blocker.

### SYS-N02 — Distinguish a message broker from the credentialed effect broker

**Affected requirements:** R02 and R09.

**Failure case to guard against:** R02's deferred “network broker” is mistaken for the mandatory credentialed effect broker in R09.

**Concrete refinement:** call the deferred infrastructure a “message broker/stream service.” The ownership table and R09 already specify the required effect authority, so this terminology issue is nonblocking.

## Scaffold and verification observations

Inspected the loopback Fastify listener and its 501 command route, both Rust diagnostic entry points, TS/Rust health validation and fixtures, the shared state names, and desktop/mobile nonexecuting shells. The reviewed paths make no execution or durable-acceptance claim and do not contain an implemented command journal, launch supervisor or relay. Those absences are consistent with S0, not defects against future milestones.

`pnpm verify` passed locally: specification structure, TypeScript checks, TS tests/build and Rust tests. Turbo reused local caches for the JavaScript checks/build; Rust tests executed. These results establish the scaffold checks only. They do not demonstrate future crash recovery, concurrency, native device qualification, isolation, encryption, hosted delivery or recovery targets. The review-scope hash was computed independently with `node scripts/spec-hash.mjs`.

The contributor-required `pnpm check:approvals` was also run. It failed because `docs/reviews/approvals.json` does not yet exist; no collective approval is claimed. The scope hash remained unchanged after writing this review.

Approval remains withheld only for SYS-001. Fix the normative profile boundary, freeze the new scope hash and request an independent recheck; do not modify this verdict.
