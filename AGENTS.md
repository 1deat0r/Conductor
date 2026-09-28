# Conductor agent instructions

Conductor uses a local-first development loop. The canonical policy is in [docs/development-workflow.md](docs/development-workflow.md); read it when a task touches process or remote integration. For product work, read only the relevant sections of SPEC.md, contracts and decision records.

## Routine work

- Inspect git status first and preserve existing changes. Work in the current checkout unless isolation or remote integration calls for a branch/worktree.
- Follow: task/context → inspect → implement → pnpm verify → review the diff → make a focused, atomic commit.
- Issues and PRs are optional for local work. Use them when they add durable tracking, coordination, risk review or are required to integrate with protected main.
- Run pnpm verify from the repository root before committing. If it cannot run, state why and do not claim verification. Add relevant platform qualification when the change needs it.

## Product and security constraints

- SPEC.md and contracts/ are normative for product behavior; docs/reference is historical rationale. This is an independent implementation: do not copy Superset code or brand assets. Keep Linux, Windows, macOS, Android and iPhone in scope; source checks do not qualify native behavior.
- The S0 scaffold cannot execute agents, authorize tools, enroll devices, store credentials or mutate repositories. Keep unsupported execution unavailable until its SPEC requirements and release gates are implemented.
- Authority, privacy, durable command acknowledgement and execution isolation are protocol concerns, not renderer conveniences. Keep model/provider adapters and untrusted work outside privileged processes. Do not add fake success or insecure fallbacks.
- Do not use production credentials, provision real cloud resources, publish packages/releases or submit apps as part of ordinary development. Follow the relevant security and release gates for explicitly requested work.
- Never bypass active repository protections, push directly to protected main, self-approve, or claim a platform/release result without evidence. See the workflow for current remote integration requirements.

## Agent skills

Use the Matt Pocock engineering skills that fit the task, but follow this repository's local-first workflow instead of any skill's default Issue/branch/PR orchestration. The installed version audit is in [docs/agents/README.md](docs/agents/README.md).

### Issue tracker

GitHub Issues are optional persistent tracking. See [docs/agents/issue-tracker.md](docs/agents/issue-tracker.md).

### Triage labels

Use the default triage labels only when an Issue needs triage. See [docs/agents/triage-labels.md](docs/agents/triage-labels.md).

### Domain docs

Use a single-context layout: root CONTEXT.md when present, plus the relevant project decisions. See [docs/agents/domain.md](docs/agents/domain.md).
