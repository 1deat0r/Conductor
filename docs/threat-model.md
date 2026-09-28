# Trust model and abuse cases

Scope: ordinary user code, hostile repositories/PRs, model/tool output, network peers, malicious previews/plugins and another tenant are untrusted. Product client/controller, credentialed broker, selected identity service, verified platform and provider isolation boundary are trusted. Host/root/control-service compromise, hardware compromise and provider policy violations are residual assumptions, not fixed by metadata. A permitted destination/session can still exfiltrate allowed data; least privilege and data-release scope bound but do not remove intent risk.

| Threat | Required prevention / honest residual | Gate |
|---|---|---|
| Guest connects to same-user host socket or credential helper | R15 qualified restricted/managed isolation excludes these resources; trusted-host code retains account-accessible resources, and R11 same-user socket ACL alone is insufficient | S3; trusted S1 labels account authority |
| Mutable file/symlink changes after approval | R16 canonical immutable inputs, safe handles and single-use consume | S2 protected action enablement |
| Old host/API request writes after transfer | R09 closes admissions and reconciles in-flight effects; conditional provider writes where available | S2 |
| Restored DB resurrects keys/approvals/spend | R24 write freeze, external recovery generation and reconciliation; no blind restart | S2 hosted control; S3 managed recovery |
| Hostile Git config/archive triggers code/traversal | R17 confined inspection/import, sanitized metadata and safe extraction | S1 before unknown-repo import |
| Preview impersonates approval or reaches another workspace | R03/R13/R14 separate UI, route identity, origin/CSRF and network enforcement | S2 before previews |
| Cross-tenant artifact lookup or cache poisoning | R18 authenticated bindings; R17 separates release/contribution cache trust | S2/S3 |
| Offline phone retains revoked data | R13 bounded read/cache policy, key/backup lifecycle; no immediate erasure promise | S2 |
| Old signed update or malicious plugin payload | R23 hash/freshness/anti-rollback and separated signing; capability admission | Before distributed updates/plugins |
| Process started twice in a crash window | R07/R08 durable inbox and supervisor receipts; explicit uncertainty without relaunch | S1 |
| Reallocated offline budget spends twice | R22 escrow remains encumbered until settled, new attempt/new allocation | S3 funded jobs |
| Test agent forges success | R19 producer provenance and independent validation; model claims are not attestations | S1 onward |

S0 attack surface is intentionally smaller: immutable desktop renderer without IPC/remote content, credential-free mobile shell, diagnostic Rust CLIs, and a loopback-only API without execution/persistence. Local health has no private data; command requests return 501 without storage. This is not permission to expose the same unauthenticated service on a network in S2. The CI configuration runs on untrusted contributions without production secrets or signing authority. Pin actions/dependencies and separate future release workflows.
