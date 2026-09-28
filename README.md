# Conductor

[Public GitHub repository](https://github.com/1deat0r/Conductor) · [Mandatory development workflow](docs/development-workflow.md) · [Unanimous specification review](docs/reviews/SUMMARY.md)

A local-first agent development workbench for Linux, Windows, macOS, Android and iPhone. This is the **S0 scaffold**, not a working agent runner. No agent execution, remote access, credentials, PTY management or approvals are enabled.

Start with [SPEC.md](SPEC.md), [implementation gates](docs/implementation-plan.md), [project state](STATE.md) and the [review record](docs/reviews/README.md). Explore the [overall architecture](docs/diagrams/overall.html) and [execution architecture](docs/diagrams/execution.html). Historical rationale lives in docs/reference; it does not describe implemented features.

## Development

Use Node 24 LTS, pnpm 11.18.0 and Rust 1.98.1 (rust-toolchain.toml). Later supported Node versions may work; validation records the actual local version. Install dependencies with `pnpm install --frozen-lockfile` and retain pnpm-lock.yaml/Cargo.lock. Only named required dependency build scripts are allowed by pnpm-workspace.yaml.

```sh
pnpm verify
pnpm check:approvals
pnpm dev:desktop
pnpm dev:api
pnpm dev:mobile
cargo run -p conductor-host -- doctor
cargo run -p conductor-supervisor -- doctor
```

Desktop dev builds the renderer and opens Electron; there is no privileged remote-content development mode. `pnpm smoke:desktop` requires a working local display or Xvfb and Electron sandbox support. Do not disable the sandbox to make smoke tests pass. API binds only 127.0.0.1:4318; GET /healthz describes scaffold capabilities and POST /v1/commands returns 501. Rust doctor commands only print health JSON. See apps/mobile/README.md for Android SDK and macOS/Xcode prerequisites.

## Layout

```text
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
scripts/                Spec/approval integrity checks
SPEC.md                 Normative implementation specification
docs/                   Decisions, threats, gates, reviews and evidence
```

Public source hosting is authorized only after unanimous final spec approval. Packages remain private/unlicensed; no package publication, deployment, store submission or distribution identity is part of this scaffold. See LICENSE-NOTICE.md before publication. CI is configured for the three desktop OS families but only actual evidence constitutes a passing run.

See [local validation evidence](docs/evidence/scaffold-validation.md) for actual commands, results and untested platform gates. The initial approved seed is the sole direct-main bootstrap exception. All subsequent development follows [the mandatory GitHub workflow](docs/development-workflow.md).
