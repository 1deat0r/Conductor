# Round 1 — platforms

- Reviewer: independent platform agent `/root/platform_review`
- Role: `platforms`
- Verdict: **REQUEST_CHANGES**
- Reviewed SHA-256: `a6297a772d4e0f0f23e1bf7d9703b96d4057d2557a4fed0961c2f9f5f4cc8d29`
- Scope: every normative file in `docs/reviews/scope.json`, including the scope file in the prescribed hash calculation. Read `AGENTS.md` first; calculated the hash locally before and after review. This is a new implementation-spec review, not carry-forward approval of the earlier architecture proposal.

The Electron/Rust and React Native/Expo selections are implementable. The native lifecycle contract, bounded primary support matrix, separate mobile UI, foreground catch-up model, key/cache requirements and gated 12-person plan are sufficiently concrete to start implementation. One normative contradiction prevents approval. Resolving it should not add scope or weaken the specified restricted/managed boundaries.

## Blocking finding

### PLAT-001 — R15 applies isolation guarantees to trusted native execution without an exception

**Affected requirements:** R15, read with R10–R11 and ADR06. Related acceptance material: `docs/threat-model.md` first threat row and `docs/implementation-plan.md` R15 traceability.

**Conflict:** R15 defines trusted-host execution as having the effective authority of the OS account, then unconditionally states that workloads cannot reach host-control sockets, policy/audit stores, device keys, SSH/Docker agents, metadata or ambient credentials. It also unconditionally requires network policy outside the workload. R11 explicitly recognizes that local IPC permissions do not protect against malicious programs running as the same user. ADR06 and the threat-model gate treat trusted S1 execution as weaker and reserve the isolation claim for qualified profiles, but R15 does not carry that qualification into its normative guarantees.

**Concrete failure case:** S1 launches a trusted native agent/build on Linux, Windows or macOS as the logged-in user. The process can exercise that account's access to files, sockets and environment credentials. Ordinary POSIX process groups or Windows Job Objects provide process management, not the promised account-level data isolation. An implementer following R15 literally must introduce a different account/container/VM boundary before enabling the expressly planned trusted S1 mode, or claim safeguards that mode cannot enforce. The spec's precedence rule means explanatory weaker-mode wording elsewhere cannot resolve the unconditional requirement.

**Required resolution:** Scope the denial guarantees and out-of-workload enforcement requirements to restricted-local and isolated-managed profiles that have actually qualified those capabilities. State explicitly that trusted-host execution retains the OS account's ambient authority and cannot claim those protections; its UI and capability reporting must disclose that fact. Preserve the existing requirements that restricted mode never silently falls back to trusted mode and directly usable workload credentials disqualify strong protected-failover guarantees. Align the threat-model/traceability wording if necessary. This is clarification of the existing three-profile design, not a request to add an S1 sandbox.

## Nonblocking refinement

### PLAT-N01 — The local aggregate check omits two named S0 checks

**Affected requirement:** SPEC section 10, S0 acceptance; implementation support in `package.json` and `.github/workflows/verify.yml`.

**Failure case:** `pnpm verify` can pass despite Rust formatting or Clippy failures, because it only runs Cargo tests; the workflow separately runs formatting and Clippy. This does not block specification implementation, and the separate checks passed during this review. It is a maintenance trap if contributors interpret the root aggregate as all S0 checks.

**Suggested resolution:** Include Rust formatting and Clippy in the local aggregate, or explicitly document the additional mandatory commands next to `pnpm verify`. Do not treat remote CI configuration as an executed platform result.

## Platform and scaffold assessment

- **R03/R10:** A packaged-asset Electron renderer with sandboxing, context isolation and no privileged IPC is appropriate for S0. Source inspection shows no task process ownership in Electron. Future normal-Quit survival depends on the separately implemented supervisor, and is not represented as working today.
- **R08/R10:** POSIX PTY versus Windows ConPTY/Job implementation is properly separated; uncertain starts, interrupted sessions and explicit unattended installation avoid false process-portability promises. Windows work starts in S1 rather than being postponed until after Linux.
- **R10/R23:** The primary OS families are retained while secondary architectures/distributions remain conditional. Minimum versions are qualification targets, not claims of framework support. Signing, updates, installed artifacts and exact platform evidence remain gates.
- **R12–R14:** Both phone sources are present. There is no background-agent assumption, privileged preview, biometric approval or encrypted-cache implementation hiding behind a source-build claim. The requirements allow native modules where actual device security/lifecycle functions need them.
- **Plan/S0:** Inspected desktop/mobile shells, diagnostic Rust CLIs, loopback API and health fixtures consistently advertise unavailable execution. The API binds to loopback and returns 501 for commands; Rust entry points only report diagnostics. The six-week proof and S1/S2/S3 sequencing are planning gates, not production or date guarantees.

## Checks and limits

Executed locally during this review:

- `node scripts/spec-hash.mjs` — reviewed hash matched at start and end.
- `pnpm verify` — passed; several Turbo steps used the local cache, desktop typecheck/build ran, and Cargo tests passed.
- `pnpm check:mobile` — Expo reported aligned dependencies.
- `cargo fmt --all --check` — passed.
- `cargo clippy --workspace --all-targets --locked -- -D warnings` — passed.

`pnpm check:approvals` was run and exited 1 because `docs/reviews/approvals.json` does not yet exist. No aggregate approval is claimed; this record also requests changes. No Windows/macOS native execution, mobile native build/device run, installed signed release, store acceptance, isolation qualification or production operation was tested by this review. Only the two assigned review records were authored.
