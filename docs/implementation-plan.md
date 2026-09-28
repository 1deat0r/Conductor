# Implementation gates and acceptance evidence

S0 is scaffold readiness. S1/S2/S3 below are future product gates and cannot be marked complete by the S0 source/build checks. No gate authorizes an external deployment, real payment, publication, enrollment or account changes.

## S0 — Current deliverable

- Desktop packaged-code shell with two local views and explicit unavailable execution. Build and smoke on this Linux host when the available display/sandbox permits.
- Android/iPhone Expo source, SDK-aligned versions, configuration and JS/native bundle export where available. No claim of native build/device execution without Android SDK or macOS/Xcode evidence.
- Loopback Fastify health and 501 command route; TS/Rust shared positive/negative health fixtures; Rust doctor/version binaries, no listeners or execution.
- Committed lockfiles, reproducible scripts, public CI configuration and precise local evidence. A configured CI matrix is not a completed remote CI run.
- The initial normative spec was approved by all three independent roles before repository creation; this is historical bootstrap evidence, not recurring implementation qualification.

## Requirement traceability

| Requirements | First implementation gate | Required acceptance evidence |
|---|---|---|
| R01–R02 | S0 then S2 functional parity | Truthful capabilities now; same create/review/clarify/approve/cancel flows on all primary clients before S2 GA |
| R03 | S0 shell; S2 IPC/preview | Sandbox/Node/CSP smoke now; adversarial IPC sender/frame, navigation, custom protocol and preview tests before enabling privileged surfaces |
| R04–R05 | S1 | Partition local/team conflicting intent; replay cancellation; assert sole writer, monotonic revisions and no attempt resurrection |
| R06 | S0 health; S1 execution contracts | Cross-language fixtures now; capability negotiation, old client/new host, unknown action rejection and cursor expiry before S2 |
| R07–R08 | S1 | Crash every inbox/start/exit receipt boundary; short-lived child; payload mismatch; disk full; output saturation; no hidden duplicate or automatic uncertain input replay |
| R09 | S2 protected effects | Delay a provider write across handoff; lose response; reject stale admission; block conflicting publication until reconciliation |
| R10 | S1 each desktop; S2 unattended | Packaged UI close/quit/crash, controller update, supervisor drain/crash, sleep/wake, logout/reboot, child cancellation and uninstall with explicit data handling |
| R11 | S2 | Pairing replay/wrong tenant, revoke active stream, stale queue after reconnect, host clock rollback, offline grant expiry and partition handoff |
| R12–R13 | S2 both phones | Physical oldest/current devices plus second Android vendor; push denied/force-quit, low memory, account switch, large text/screen readers, deep links, key invalidation/restore, draft retention and ambiguous approval reply |
| R14 | S2 preview | Malicious page bridge/loopback/private network, sibling origin/CSRF, port reuse, expired ticket, route generation and no auth-header leak |
| R15 | S1 trusted authority disclosure; before any restricted profile; S3 managed isolation | Malicious build tries host sockets, metadata, SSH/Docker agents, direct egress and role impersonation; per-platform boundary tests; clean native reset before shared pools |
| R16 | S2 protected actions | Concurrent approval consume; mutate untracked script/cwd/env/symlink after approval; wrong device/audience/epoch; canonicalization vectors |
| R17 | S1 trusted; S3 hostile | Git fsmonitor/hooks/textconv/config, path aliases/traversal/symlinks, submodule/LFS auth, external Git race, release-cache poisoning |
| R18 | S1 local checkpoints; S2 before team artifact exposure; S3 cleanup | Crash upload/manifest/GC boundaries; source retained until complete published manifest; cross-tenant ID and URL expiry before S2 artifact APIs; import/export and referenced-key/object restore |
| R19 | S1 review; later queue | Forged workload success, dirty candidate/evidence invalidation, integrated commit tests, branch protection and separate publication authorization |
| R20 | S1 two adapters plus PTY | Capability/resume/usage truth across adapter versions, cancellation, privacy-approved fallback and no invented subscription entitlement |
| R21 | S1 source retrieval | Cross-tenant search denial, stale provenance, deleted-source derived index removal, prompt injection cannot promote authority |
| R22 | S1 local bounds; S3 funded work | Restart/DST/occurrence deduplication, bounded fanout, partition spend/restore/clock rollback, worst-case in-flight overshoot and orphan inventory cleanup |
| R23 | S2 remote; before distributed updates | Audience/SSRF/token leak tests; signed downgrade/payload substitution/revocation; interrupted install, supported version pairs, separate migration writers |
| R24 | S2 hosted; S3 managed | Restore before effect/revocation/spend while old host lives; fresh generation, uncertainty quarantine, full manifest/object/key recovery and measured SLO/restore drill |
| R25 | Initial bootstrap; ongoing development | Agents complete routine work autonomously; the independent expert board decides material choices without user approval; run local pnpm verify, review diffs, commit atomically and synchronize; Issues/branches/PRs are optional; exact-commit Ubuntu/Windows/macOS checks gate main; preserve no-force/no-delete protections, security/release gates and honest platform evidence |

## Delivery sequencing and ownership

Six-week proof: one real task on each desktop OS, both phones reading current state, one stale action denied, surviving UI/controller restarts and signed internal test distributions. Reassess scope and staffing afterward. Proposed 12 engineers: runtime/platform 3, desktop 2, mobile 2, coordination/integrations 3, release/reliability/QA 2, with design/security support. Named individuals and on-call coverage are staffing prerequisites before a production commitment.

S1: local command journal, supervisor reconciliation and typed execution contracts → owned workspace → two adapters and PTY → review/evidence. Start Windows process tests immediately; do not port after Linux features are finished. Avoid implementing hosted orchestration before ownership and local recovery are proved.

S2: owned-host enrollment, gateway, team authority, resume/revocation and phone inbox → protected Action representation and mobile keys/cache → preview isolation and private beta → install/update/support/store gates. Begin mobile account/signing/demo-host qualification early. Select owned bundle/package identifiers, territories, privacy/account deletion and purchase flows; no App Store/Play exception is assumed. No mobile executable plugin marketplace or unreviewed native updates.

S3: one managed Linux provider, qualified isolated profile, escrow/resource meter, checkpoint publication and cleanup, restore/orphan drills. Shared native pools, self-hosted distribution, integration queues and a general plugin registry are separately funded/gated. Native Apple work requires suitable licensed/platform-compliant capacity. Cloud design uses one tenant home region and ordinary managed services before active-active global writes.

## Definition of evidence

Every result identifies commit/tree or digest, command, tool version, OS/architecture/device, execution profile, timestamp, result and limitation. Store local evidence in docs/evidence and reviewer decisions in docs/reviews. Release artifact qualification must use installed signed binaries, not just development servers. Future tests remain unchecked in this document; do not fill their completion from compilation success.
