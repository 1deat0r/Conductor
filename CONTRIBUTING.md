# Contributing to Conductor

Conductor is developed locally first. For ordinary changes, work in the current checkout, run the canonical verification command, review the diff and make a focused commit. Do not create an Issue, branch, worktree or PR just to satisfy process. The complete policy and exceptions are in [docs/development-workflow.md](docs/development-workflow.md).

## Verify before committing

From the repository root, run:

    pnpm verify

This is the canonical pre-commit check. It validates the spec structure, TypeScript workspace, tests and build, then Rust formatting, Clippy and tests. Use targeted commands while iterating; run relevant platform checks for changes that need them. If verification cannot run, report the exact command and reason, and do not describe the change as verified.

## GitHub and review

Issues and PRs are optional unless durable tracking, coordination, material review or protected-branch integration makes them useful. The active main ruleset currently requires remote changes to arrive through a PR with required cross-platform checks and an eligible non-author approval. Never push directly to protected main, self-approve or bypass a gate. Check the live ruleset if its requirements may have changed.

Use independent human review in proportion to risk. Changes to authority, trust boundaries, privacy, durability, credentials, platform lifecycle, release controls or execution isolation need focused specialist review and relevant adversarial/platform evidence before the affected capability is enabled or released. AI reviews can find issues but do not establish human approval or security qualification.

Preserve user work, keep commits atomic and evidence accurate. Do not introduce production credentials, private third-party code, fake success or unapproved releases.
