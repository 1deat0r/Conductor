# Round 2 — platforms

- Reviewer: independent platform agent `/root/platform_review`
- Role: `platforms`
- Verdict: **REQUEST_CHANGES**
- Reviewed SHA-256: `bc423cc2df0c3cf2ee1c663d8a47bbfd9ade3374630d16ef085648f5b4b8c598`
- Approval meaning: ready to implement this specification; not production, native-platform, store or live-GitHub qualification.

Read `AGENTS.md` first and independently reviewed every normative file in the expanded `docs/reviews/scope.json`, including the contributor documents, workflow, CODEOWNERS and main-ruleset configuration. Calculated the prescribed hash locally at the beginning and end of review; both matched the value above. The prior round remains unchanged.

## Findings and resolution

**The original platform blocker is resolved, but one new governance/configuration blocker remains.** Before finalizing this round, I independently checked the action reference raised during the review and confirmed the mismatch below.

### PLAT-002 — The pnpm action pin is a tag object, not the required commit

**Affected requirement:** R25 and `docs/development-workflow.md` requirement to pin Actions to commit SHAs; `.github/workflows/verify.yml`.

**Failure case:** The workflow uses `pnpm/action-setup@f40ffcd9367d9f12939873eb1018b921a783ffaa`. Running `git ls-remote https://github.com/pnpm/action-setup.git refs/tags/v4 'refs/tags/v4^{}'` independently returned that object for `refs/tags/v4` and `b906affcce14559ad1aafd4ab0e942779e9f58b1` for the peeled commit. The reviewed workflow therefore does not meet its own explicit commit-SHA requirement. This is not a claim that a tag-object hash is a movable branch or that an exploit was demonstrated.

**Required resolution:** Replace the action reference with the verified peeled commit SHA, update the specification hash and obtain fresh approvals for that final scope before bootstrap. No product scope expansion is needed.

- **PLAT-001 is resolved — R09, R15–R16.** Trusted-host execution now explicitly inherits account-accessible files, sockets, credentials and network access, with visible capability disclosure. Resource denial and external enforcement apply only to qualified restricted/managed profiles. Protected publication requires an actually enforced credential/policy boundary; voluntary broker use by a trusted CLI is insufficient. Shell approval no longer implies isolation the profile does not supply. Supporting threat and acceptance descriptions align. This preserves the intended S1 scope without misrepresenting native process management as a sandbox.
- **PLAT-N01 is resolved — S0 acceptance and local verification.** `pnpm verify` now invokes `check:rust`, including formatting, Clippy on all targets, and locked Cargo tests. That aggregate passed during this review.

## Platform readiness assessment

**R02–R03, R08–R10:** Electron and Rust remain credible choices for the specified boundaries. UI/controller/supervisor lifetimes are distinguished, ambiguous native starts are not replayed blindly, and POSIX versus ConPTY/Job behavior is delegated to explicit platform qualification. The primary OS matrix is bounded, with secondary architectures/distributions clearly deferred. Future installed/signed artifact evidence remains separate from this scaffold's development shell.

**R12–R14:** The Expo/React Native design supports the stated Android/iPhone operational-client scope without depending on continuous background execution. Native approval presentation, preview isolation, foreground catch-up, separate keys and bounded cache access remain required before those capabilities activate. Both mobile bundle exports passed, which demonstrates source bundling only; it establishes neither native builds nor device security/lifecycle behavior.

**R23–R24 and implementation plan:** Update/migration ownership, runtime security support and installation recovery are still release gates. The 12-engineer allocation and six-week proof are planning assumptions with scope reassessment; the document does not promise GA dates. Deferred managed execution and shared native pools do not remove any primary client OS family.

## New GitHub governance assessment

**R25, development workflow and contributor rules are coherent and implementable.** The initial public seed is conditioned on unanimous approval of the complete final spec. Its direct-main exception expires after the seed; subsequent corrections require ordinary issue/branch/PR integration. Applying and reading back protection, then inspecting actual CI, are required actions rather than claims that configuration files alone protect a nonexistent repository.

The ruleset requires a PR, three named platform checks from the selected integration, an up-to-date branch, stale-review dismissal, resolved conversations, one eligible approval and approval of the latest reviewable push. It blocks deletion/force pushes, requires linear history, restricts PR merges to squash and supplies no normal bypass actors. These mechanics correspond to GitHub's documented ruleset capabilities; GitHub also documents that authorized repository administrators can change rules. The workflow correctly treats that as an explicit trust/policy limit. [GitHub ruleset rules](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-rulesets/available-rules-for-rulesets), [rules REST API](https://docs.github.com/en/rest/repos/rules).

The three expert records are independent analysis with exact reviewed head SHAs, not three invented GitHub principals. New heads invalidate prior expert approvals; base/rebase changes require reassessment of the merge result and rerun CI. File-based spec approval cannot substitute for reviewing implementation changes. The integrator and eligible GitHub reviewer must inspect those records; a JSON verdict is not presented as an authenticated reviewer identity. Final exact-head PR records can be posted after committing reports, avoiding a requirement to embed a commit's own hash in its contents.

CODEOWNERS assigns accountability without requiring the PR author to approve their own work. The separate eligible non-author/latest-push approval requirement may leave a shared-account agent PR unmerged until a legitimate reviewer is available. The document expressly requires that outcome and forbids protection weakening or fabricated identities. This is an intentional integration prerequisite, not a hidden automated-team capability.

**CI is appropriately bounded for S0, subject to PLAT-002.** Checkout does not retain credentials, token permissions are read-only, and jobs do not combine privileged `pull_request_target` execution with PR code. Linux smoke and both mobile exports add useful scaffold evidence; neither replaces Windows/macOS installed-binary qualification or native mobile/device gates. Actual required-check names, integration binding and active main protections must still be verified after the authorized bootstrap, as specified.

### PLAT-N02 — Make the Linux CI sandbox prerequisite explicit

**Affected requirements:** R03 and R25's required Linux smoke.

**Failure case:** `xvfb-run` supplies a display but does not configure Chromium's OS sandbox. Ubuntu's user-namespace policy can prevent a downloaded Electron binary from using the namespace sandbox; a passing local run does not prove the hosted runner has identical policy. Chromium documents the setuid helper as an alternative that preserves sandboxing without changing Ubuntu's global policy. [Chromium sandbox guidance](https://chromium.googlesource.com/chromium/src/+/main/docs/security/apparmor-userns-restrictions.md).

**Suggested resolution:** In the disposable GitHub-hosted Linux VM, explicitly install the pinned Electron binary, resolve its adjacent bundled `chrome-sandbox`, verify the file, set only that helper to root ownership and mode 4755, then run the smoke as the unprivileged runner user. Quote paths and keep this preparation confined to hosted Linux CI; do not run Electron as root, use `--no-sandbox`, or weaken global AppArmor/sysctl policy. This is a practical CI prerequisite, not installed-package or AppImage qualification. The installed Electron 44 package exposes the `install-electron` CLI. No privilege changes were performed on the local machine for this review.

## Local evidence and limits

Executed for this review:

- `node scripts/spec-hash.mjs` — matching scope hash at start and end.
- `pnpm verify` — passed; unchanged TypeScript/build tasks used local Turbo cache, and the aggregate included Rust formatting, Clippy and tests.
- `pnpm --filter @conductor/mobile export` — passed; Android and iOS Hermes bundles generated.
- `xvfb-run -a pnpm smoke:desktop` — passed; Electron loaded expected content with no renderer Node global and sandbox configuration enabled. It emitted a Wayland/Vulkan compatibility warning. No sandbox-disable option was used.

`pnpm check:approvals` exited 1 because the aggregate `docs/reviews/approvals.json` does not yet exist. No aggregate approval is claimed.

These checks ran on the local Linux environment, not GitHub-hosted Node 24 runners. No Windows/macOS native execution, mobile native build/device test, signed installer, running supervisor, enforced workload isolation, remote control plane, store submission, public repository or live protection configuration was qualified by this review. Those remain disclosed gates. No normative or implementation source was edited by the reviewer.
