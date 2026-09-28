# Independent specification review

Each expert reviews the same bytes named by scope.json and produces a Markdown analysis plus a JSON verdict. roles: systems, security, platforms. Verdict is APPROVE or REQUEST_CHANGES; blocking_findings must be empty for approval. Every finding gets a stable ID, affected requirement, failure case and concrete resolution. Optional implementation refinements do not block a spec that is ready to implement.

Reviewer JSON: schema_version, role, reviewer, round, spec_sha256, verdict, blocking_findings, nonblocking_notes. The reviewer writes it; the author may not alter it. approvals.json points to the final independent records and binds them to `node scripts/spec-hash.mjs`. `pnpm check:approvals` fails for missing/rejected/stale records. Old rounds remain preserved. This is independent model-agent review, not a human or implementation security audit.
