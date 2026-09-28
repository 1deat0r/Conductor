# Domain documentation

Conductor uses a single-context layout for the Matt Pocock engineering skills:

- CONTEXT.md at the repository root for shared domain vocabulary.
- docs/decisions.md is the existing concise architecture-decision register; use docs/adr/ for full standalone ADRs if they become useful.
- Material unsettled choices go to the independent expert board; preserve its decision record under docs/decisions/board/ and link durable decisions from docs/decisions.md.

Read only the glossary and decisions relevant to the task. If those files do not exist, proceed without calling out their absence. Create them lazily when domain modeling or a real decision makes durable documentation useful; do not add empty context files up front. Do not create a CONTEXT-MAP.md or split docs by package unless the owner changes this choice.

Use agreed domain terms in code, tests and task descriptions. Surface an ADR conflict rather than silently overriding it. Product requirements and contracts remain normative over context notes.
