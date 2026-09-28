# Conductor

[Public GitHub repository](https://github.com/1deat0r/Conductor) · [Local-first development workflow](docs/development-workflow.md) · [Initial specification review archive](docs/reviews/SUMMARY.md)

A local-first agent development workbench for Linux, Windows, macOS, Android and iPhone. This is the **S0 scaffold**, not a working agent runner. No agent execution, remote access, credentials, PTY management or approvals are enabled.

Start with [the product specification](SPEC.md), [implementation gates](docs/implementation-plan.md), [project state](STATE.md) and [review archive](docs/reviews/README.md). Explore the [overall architecture](docs/diagrams/overall.html) and [execution architecture](docs/diagrams/execution.html). Historical rationale lives in docs/reference; it does not describe implemented features.

## Development

Use Node 24, pnpm 11.18.0 and Rust 1.98.1 (rust-toolchain.toml). Later supported Node versions may work; validation records the actual local version. Install dependencies with pnpm install --frozen-lockfile and retain pnpm-lock.yaml and Cargo.lock. Only named dependency build scripts are allowed by pnpm-workspace.yaml.

Run the canonical verification command before committing:

    pnpm verify

Use task-specific checks while iterating. pnpm smoke:desktop needs a working display or Xvfb and Electron sandbox support; do not disable the sandbox. pnpm check:mobile checks Expo dependency alignment. CI also exports the mobile bundle and runs desktop smoke on Linux. See [the workflow](docs/development-workflow.md) for when Issues, branches, PRs and CI add value and for current protected-main requirements.

    pnpm dev:desktop
    pnpm dev:api
    pnpm dev:mobile
    cargo run -p conductor-host -- doctor
    cargo run -p conductor-supervisor -- doctor

Desktop dev builds the renderer and opens Electron; there is no privileged remote-content development mode. API binds only 127.0.0.1:4318; GET /healthz describes scaffold capabilities and POST /v1/commands returns 501. Rust doctor commands only print health JSON. See apps/mobile/README.md for Android SDK and macOS/Xcode prerequisites.

## Layout

    apps/desktop/           Electron + React shell
    apps/mobile/            Expo/React Native Android and iPhone source
    services/control-api/   Fastify scaffold health / unsupported command route
    services/gateway/       Documented future authenticated relay boundary
    packages/domain/        Shared source domain vocabulary
    packages/protocol/      Runtime health validation and conformance tests
    crates/protocol/        Rust health type and shared fixture tests
    crates/host/            Nonexecuting host diagnostic CLI
    crates/supervisor/      Nonexecuting supervisor diagnostic CLI
    contracts/              Normative wire schemas and fixtures
    scripts/                Spec validation and hash utilities
    SPEC.md                 Normative implementation specification
    docs/                   Decisions, threats, gates, reviews and evidence
    docs/agents/            Matt Pocock skills configuration

The initial specification was unanimously reviewed before the public source repository was created. That review is preserved as historical bootstrap evidence; it is not a recurring gate for routine development. Packages remain private/unlicensed; no package publication, deployment, store submission or distribution identity is part of this scaffold. See LICENSE-NOTICE.md before publication. CI provides clean-checkout source checks on three desktop OS families, but only actual evidence constitutes a passing run.

See [local validation evidence](docs/evidence/scaffold-validation.md) for actual commands, results and untested platform gates. Remote integration with protected main must follow the active GitHub ruleset; routine local work follows the local-first workflow.
