# Round 1 — independent security review

**Verdict: APPROVE — readiness to implement, not production readiness.**

- Role: security
- Reviewer: independent security review agent (`/root/security_review`)
- Reviewed scope SHA-256: `a6297a772d4e0f0f23e1bf7d9703b96d4057d2557a4fed0961c2f9f5f4cc8d29`
- Scope: every normative file named by `docs/reviews/scope.json`, after reading `AGENTS.md`; implementation inspection limited to checking S0's claimed boundary and verification wiring.
- Blocking findings: none.

This is a fresh review of the implementation specification. The verdict does not inherit the earlier architecture review and does not certify unimplemented security controls. The review hash was computed independently before and after inspection and matched both times.

## Security assessment

**Trust and authority (R02–R04, R11, R15).** The specification distinguishes trusted account-level execution from verified restricted/managed profiles and does not mistake structured adapters, worktrees, same-user socket ACLs or Job Objects for hostile-code isolation. Workload identities cannot impersonate approving clients. Host, coordinator, supervisor and broker authorities are separately assigned. Same-user/root/control-service compromise is stated as a trust limitation. Unsupported isolation cannot silently downgrade.

**Approval and credentials (R06–R07, R13–R16, R23).** Identity comes from authenticated state, action schemas fail closed, and immutable canonical action inputs bind actor/device/audience/ownership/policy/target. Single-use approval consumption and durable intent registration are atomic. Mutable filesystem/environment changes cannot retain an unrelated approval. Mobile user-presence keys, trusted approval UI, current-state refresh and uncertain-response handling are requirements before activation. Downstream tokens have separate audiences and scope; secrets cannot enter ordinary logs, argv or Git URLs. Authenticated browser sessions expose their actual coarse authority.

**Replay, revocation and recovery (R07–R11, R18, R22, R24).** At-least-once delivery has payload-bound deduplication, expiry and retained tombstones. Unknown launches and provider effects cannot be silently replayed. The text distinguishes preventing new admissions from recalling an already-dispatched effect. Revocation closes active input/stream authority, disconnected grants are bounded, and uncertain time fails closed for protected work. Disaster restore freezes writes, fences old authority, establishes a fresh recovery generation and reconciles lost receipts/approvals/budgets before retries. A restored counter is not treated as sufficient fencing.

**Hostile content and tenants (R03, R14–R19, R21, R23).** Git configuration, imports, plugins, previews, active artifacts and test output are untrusted at their boundaries. Preview routing binds service instances and generations, with origin/CSRF and private-network controls. Artifact identifiers/hashes are not access tokens, authorization follows derived operations, and evidence distinguishes workload claims from controller observations. Native pool reuse and update/supply-chain controls have explicit gates.

## S0 claims checked

The inspected source supports the narrower S0 claims: the desktop has a static packaged renderer, strict CSP, sandbox/context isolation, no Node integration or privileged IPC, denied navigation/popups/permissions, and confined custom-scheme asset reads. Mobile contains static UI and no enrollment, credential collection or host connection. Rust entry points only print diagnostics/version. Fastify binds loopback, exposes nonprivate scaffold health and returns 501 for commands with no execution or persistence. The executable health contract reports `execution_available: false` and rejects contrary or unknown envelopes through its intended parsers.

`pnpm verify` completed successfully: spec structure, TypeScript checking, protocol/API tests, renderer build and Rust tests. Turbo reused cached TypeScript/test/build results, which is recorded here rather than presented as a fresh adversarial run. No runtime isolation, encrypted cache, real device, hosted recovery, update-signing or native lifecycle qualification was inferred from those results.

`pnpm check:approvals` was run after writing this record and correctly could not complete because the author has not yet assembled `docs/reviews/approvals.json` (ENOENT). This individual approval is not a claim of three-role consensus or a passing aggregate approval check.

## Nonblocking implementation notes

**SEC-N01 — Pin CI action implementations immutably.** Affected requirement: R23 and the threat model's instruction to pin actions. `.github/workflows/verify.yml` currently uses mutable major tags such as `actions/checkout@v4`, `actions/setup-node@v4` and `pnpm/action-setup@v4`. If a tag moves or its upstream is compromised, the action's executable code can change without a repository diff despite application lockfiles. Resolve by using reviewed full commit SHAs with version comments and an explicit update path. This is scaffold hardening, not a blocking specification gap: immutable update admission is already required, the current CI has read-only repository permissions and no release signing/production credentials, and this review does not approve distribution or production use.

**SEC-N02 — Schedule artifact authorization tests at first exposure.** Affected requirement: R18 and the implementation-plan traceability row. That row groups local checkpoints under S1 and managed cleanup under S3; its shared acceptance-evidence cell also contains cross-tenant lookup and URL-expiry cases. When implementing S2 team artifact APIs, run those authorization cases before exposing them rather than waiting for the S3 cleanup milestone. R04/R18 already require authorization at use and the threat model places the relevant boundary in S2/S3, so this is a sequencing reminder, not permission to defer access control or a contradictory requirement needing a spec rewrite.

All future security assertions remain conditional on their declared executable release gates. No normative or implementation source was changed by this reviewer.
