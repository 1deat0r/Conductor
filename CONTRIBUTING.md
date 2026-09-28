# Contributing to Conductor

**All development must use the mandatory [GitHub workflow](docs/development-workflow.md).** Read AGENTS.md, SPEC.md, STATE.md and the implementation gates first.

Start with an issue, use an isolated feature branch/worktree, open a draft PR, run `pnpm verify` and `pnpm check:approvals`, then obtain independent systems/security/platform reviews on the exact head. Resolve all blockers and refresh stale reviews. Protected main requires green checks and an eligible non-author GitHub approval; shared-account agents cannot self-approve through GitHub. Never push directly to main after the initial approved bootstrap or disable a gate to merge.

Keep changes focused and evidence honest. Native/device tests, hosted recovery and security enforcement must be tested before their capabilities are enabled. Follow the public PR template and maintainers' scope. No production credentials, third-party private code, unsanitized transcripts or unapproved releases.
