# Issue tracker

GitHub Issues are available for Conductor's persistent backlog: 1deat0r/Conductor. Use the gh CLI when an Issue materially helps with multi-session work, deferred bugs, dependencies, external reports, major features or coordination.

Issues are optional. For routine work that can be implemented and verified in the current session, use the task/context directly; do not create an Issue merely because a skill asks for one. Local notes under .scratch/<topic>/ are suitable when lightweight persistence is useful. Do not put credentials, provider transcripts or private third-party source into an Issue.

Pull requests are not a request-triage surface. A PR may be used when it provides meaningful review or when the protected main branch requires it for integration. See [the local-first development workflow](../development-workflow.md) for the current remote rules.

Useful commands when an Issue is warranted:

- gh issue create --title "..." --body-file <path>
- gh issue view <number> --comments
- gh issue list --state open
- gh issue edit <number> --add-label <label>
