# S0 validation — 2026-09-28

This is the local validation record before the initial GitHub bootstrap; later hosted CI results are recorded by GitHub Actions.

Environment: Linux 7.0.0-34-generic, x86_64; Node v26.8.1; pnpm 11.18.0; Rust/Cargo 1.98.1. CI is configured for Node 24 on Linux, Windows and macOS; those remote jobs have not run. Local use of Node 26 is allowed by the scaffold's engine range but does not establish Node 24 qualification.

All results below are local development checks. The source manifest in this directory identifies the final scaffold bytes; the separate review manifest identifies the normative specification bytes. Neither is an independent attestation or release signature.

| Command | Observed result | Limit |
|---|---|---|
| `pnpm install --frozen-lockfile` | Passed; lockfile unchanged | Registry/platform availability is still required on a fresh machine |
| `pnpm verify` | Passed: spec structure, all five TS packages, five TS/API tests, desktop renderer build, Rust format/lint and three Rust tests | Spec structure is not semantic review; no executing host exists |
| `xvfb-run -a pnpm smoke:desktop` | Passed: real Electron launch, expected renderer content, no renderer Node global; sandbox enabled | Development Linux shell, not installed/signed package or all desktop targets |
| `pnpm check:mobile` | Passed; Expo reported dependencies up to date | Dependency alignment is not native qualification |
| `pnpm --filter @conductor/mobile export` | Passed; Android and iOS Hermes bundles generated | Not APK/AAB/IPA builds, simulator execution or physical-device tests |
| `cargo run -q -p conductor-host -- doctor` | Valid scaffold health, execution unavailable | Diagnostic only |
| `cargo run -q -p conductor-supervisor -- doctor` | Valid scaffold health, execution unavailable | Diagnostic only |

The initial TS run failed because Node test types were not explicitly selected. Package tsconfig files were corrected, and subsequent checks passed. A later desktop smoke emitted a Wayland/Vulkan compatibility warning before passing; no sandbox-disable flag was used. Mobile export emitted only color-environment warnings. Install reported a deprecated transitive uuid@7.0.3 dependency; this is recorded for dependency maintenance, not represented as a security assessment.

`verify.txt` and `desktop-smoke.txt` retain the final local command output. Turbo may replay unchanged task results from the earlier successful executions; Rust tests executed in the final verify command. The mobile export and doctor results were observed in the task tool output, summarized here rather than represented as native test logs.

Not performed: Windows/macOS execution, native Android/iOS builds or device tests, Node 24 CI, installer/notarization/store/signing checks, hostile-workload isolation, execution/crash recovery, authenticated remote control, mobile key/cache enforcement, cloud deployment or production security audit. These remain explicit specification gates.

At bootstrap, pnpm check:approvals verified the initial specification verdicts. That one-time gate was retired from routine verification on 2026-09-29; the original review records remain historical and do not qualify product security.

Workflow YAML parsed successfully and the Linux sandbox setup passed `bash -n` syntax validation. Its privileged helper setup is scoped to an ephemeral GitHub-hosted Linux runner and was not executed against the developer host. A limited scan of new nonignored scaffold files found no matches for known GitHub/AWS token and private-key patterns; that scan is not a comprehensive secret audit.
