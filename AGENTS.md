# Conductor agent instructions

Conductor uses an autonomous, local-first development loop. The canonical policy is in [docs/development-workflow.md](docs/development-workflow.md); read it when a task touches process or remote integration. For product work, read only the relevant sections of SPEC.md, contracts and decision records.

## Routine work

- Inspect git status first and preserve existing changes. Work in the current checkout unless isolation or remote integration calls for a branch/worktree.
- Follow: task/context → inspect → implement → pnpm verify → review the diff → focused atomic commit → synchronize.
- Complete routine work end-to-end without asking the user to choose reversible implementation details or to approve routine local actions. Resolve defaults from the repository and task, note material assumptions, and proceed.
- Matt Pocock skills' default prompts to ask, wait for direction, confirm, create an Issue, or use a PR do not apply when they add no material value. See [the Conductor autonomy policy](docs/agents/autonomy.md).
- Issues and PRs are optional. Use them when durable tracking, coordination, risk review or substantial remote review adds value; agents may carry them through without routine user approval.
- Run pnpm verify from the repository root before committing. If it cannot run, state why and do not claim verification. Add relevant platform qualification when the change needs it.
- Synchronize commits to protected main autonomously after the exact commit has successful required CI statuses. If status checks are missing, stage that commit on a temporary branch and continue independent work until they finish.

## Product and security constraints

- SPEC.md and contracts/ are normative for product behavior; docs/reference is historical rationale. This is an independent implementation: do not copy Superset code or brand assets. Keep Linux, Windows, macOS, Android and iPhone in scope; source checks do not qualify native behavior.
- The S0 scaffold cannot execute agents, authorize tools, enroll devices, store credentials or mutate repositories. Keep unsupported execution unavailable until its SPEC requirements and release gates are implemented.
- Authority, privacy, durable command acknowledgement and execution isolation are protocol concerns, not renderer conveniences. Keep model/provider adapters and untrusted work outside privileged processes. Do not add fake success or insecure fallbacks.
- Do not use production credentials, provision real cloud resources, publish packages/releases or submit apps as part of ordinary development. Follow the relevant security and release gates for explicitly requested work.
- Preserve live repository protections: never bypass rules, force-push, delete protected refs, fabricate status/review evidence, or claim a platform/release result without evidence. Direct fast-forward integration is allowed when the active ruleset permits it; see the workflow for exact requirements.
- Keep independent security and release gates for sensitive capabilities. If qualified review or evidence is missing, leave only the affected capability/release blocked and continue safe work.

## Agent skills

Use the Matt Pocock engineering skills that fit the task, but follow this repository's local-first workflow instead of any skill's default Issue/branch/PR orchestration. The installed version audit is in [docs/agents/README.md](docs/agents/README.md).

### Issue tracker

GitHub Issues are optional persistent tracking. See [docs/agents/issue-tracker.md](docs/agents/issue-tracker.md).

### Triage labels

Use the default triage labels only when an Issue needs triage. See [docs/agents/triage-labels.md](docs/agents/triage-labels.md).

### Domain docs

Use a single-context layout: root CONTEXT.md when present, plus the relevant project decisions. See [docs/agents/domain.md](docs/agents/domain.md).
