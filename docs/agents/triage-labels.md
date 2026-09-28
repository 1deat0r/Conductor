# Triage labels

Use the installed Matt Pocock triage skill only when an Issue needs triage. Issues and triage are optional; do not route routine local tasks through GitHub.

| Canonical role | GitHub label | Meaning |
| --- | --- | --- |
| needs-triage | needs-triage | Maintainer evaluation is needed |
| needs-info | needs-info | Waiting for reporter information |
| ready-for-agent | ready-for-agent | Scoped and ready for an agent |
| ready-for-human | ready-for-human | Requires a genuinely human-only action, missing owner authority or independent release gate |
| wontfix | wontfix | Will not be actioned |
| bug | bug | Issue category: broken behavior |
| enhancement | enhancement | Issue category: requested improvement |

When an Issue is triaged, use one category and one state when those labels are available. The five state labels are the default Matt Pocock vocabulary. Do not create Issues or labels just to satisfy the triage state machine. Resolve ordinary state conflicts from evidence and advance agent-ready work without asking the maintainer; use `ready-for-human` only for a real human-only dependency, not for routine review or approval.
