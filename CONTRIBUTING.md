# Contributing to Conductor

Conductor is developed autonomously and locally first. For ordinary changes, work in the current checkout, run the canonical verification command, review the diff, make a focused commit and synchronize it without routine user approval. The independent expert board makes material product and engineering decisions; do not ask the user to choose between options. Do not create an Issue, branch, worktree or PR just to satisfy process. The complete policy is in [docs/development-workflow.md](docs/development-workflow.md) and [docs/agents/decision-board.md](docs/agents/decision-board.md).

## Verify before committing

From the repository root, run:

    pnpm verify

This is the canonical pre-commit check. It validates the spec structure, TypeScript workspace, tests and build, then Rust formatting, Clippy and tests. Use targeted commands while iterating; run relevant platform checks for changes that need them. If verification cannot run, report the exact command and reason, and do not describe the change as verified.

## GitHub and review

Issues and PRs are optional unless durable tracking, coordination, material review or remote verification makes them useful. The active main ruleset requires successful Ubuntu, Windows and macOS checks, linear history, no deletion and no force push; it does not require a PR or review. A direct fast-forward push is allowed only for the exact commit with successful required checks. Check the live ruleset if its requirements may have changed, and never bypass it.

Use independent security-informed review in proportion to risk. Changes to authority, trust boundaries, privacy, durability, credentials, platform lifecycle, release controls or execution isolation need focused specialist review and relevant adversarial/platform evidence before the affected capability is enabled or released. AI reviews can find issues but do not establish human approval or security qualification. If a genuine human-only or release gate blocks one capability, record it and continue other safe work.

Preserve user work, keep commits atomic and evidence accurate. Do not introduce production credentials, private third-party code, fake success or unapproved releases.
