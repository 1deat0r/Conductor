# Autonomous operation in Conductor

This is the project-specific adapter for the installed Matt Pocock skills. [The development workflow](../development-workflow.md) is the canonical policy; this file tells agents how to apply it when a skill's default is interactive.

## Skill behavior

- **Grilling / grill-me / grill-with-docs:** inspect the repository and answer factual questions yourself. For choices with a safe, reversible default, select it, record the assumption if it affects the result, and proceed. Ask only when the unresolved choice materially changes product behavior and no safe default exists.
- **To-spec / implement / implement-spec / TDD:** derive scope and test seams from the user task, SPEC, contracts, existing architecture and implementation gates. Do not stop for routine plan, seam, or test approval. Preserve every explicit security/release gate and verify the resulting change.
- **To-tickets / Wayfinder:** work directly from the current task for small, in-session work. Create a GitHub Issue/map only for durable backlog, multi-session work, dependencies, major features or coordination. When a durable plan helps, create and maintain it without asking for routine confirmation.
- **Triage:** inspect evidence, choose the correct state/category labels, and advance agent-ready work without asking a maintainer to repeat the instruction. Leave work awaiting an actual external fact or human-only action only when that dependency is real; record the exact blocker.
- **Code review / diagnosing bugs:** inspect available evidence, relevant source and local checks first. If a missing production artifact, credential, physical-device action or independent reviewer blocks only one conclusion, report that limitation and continue all other safe work. Never invent a reproduction, review identity, CI status or security qualification.
- **PR / GitHub skills:** use GitHub only when it adds review, tracking, coordination or required remote verification. Keep a PR moving through checks and merge it autonomously when all actual rules are satisfied. Do not ask the user to approve a PR when no such rule exists.
- **Wizard / human-in-the-loop skills:** use only for steps that truly require the account owner or a human physically present. Do not create a human wizard to obtain approval for routine code changes.

## Stop only for a real boundary

The agent may inspect, edit, verify, commit and synchronize repository changes; manage task branches; and create/update Issues or PRs when justified by the workflow. Continue safe, reversible work without prompting. Do not use production credentials, spend money, provision live infrastructure, publish, deploy, submit to app stores, announce externally, send third-party messages, or perform irreversible data/permission changes unless the task explicitly authorizes the specific action.

Independent security-informed review and evidence required by SPEC before a sensitive capability is enabled or released are genuine product gates, not routine maintainer approvals. If they cannot be satisfied, keep that capability or release disabled and proceed with unrelated work. The S0 scaffold remains non-executing.

The upstream skill package stays unmodified so its version can be compared byte-for-byte with the latest upstream release. These repository rules take precedence when using that package inside Conductor.
