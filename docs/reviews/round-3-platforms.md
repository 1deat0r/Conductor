# Round 3 — platforms

- Reviewer: independent platform agent `/root/platform_review`
- Role: `platforms`
- Verdict: **APPROVE**
- Reviewed SHA-256: `46bd998630729de759eecb2e7b10ceca6ab482220c52b5122d4ce98b43ae1b01`
- Scope: all 15 normative files listed in `docs/reviews/scope.json`, plus the scope file included by the prescribed hash calculation.

Read `AGENTS.md` first, reread the complete final normative bundle, and independently calculated the scope hash at the beginning and after the checks. Both matched the value above. Prior review rounds remain unchanged and apply only to their recorded hashes.

**No blocking findings remain. This approval means readiness to implement the specification, not tested production or native-platform readiness.**

## Closed findings and final changes

- **PLAT-001 remains resolved — R09, R15–R16.** Trusted native execution explicitly retains account authority and cannot claim the resource-denial guarantees of qualified restricted/managed profiles. Strong publication requires an independently enforced credential/policy boundary. The implementation plan and threat model align, without adding an unplanned S1 sandbox.
- **PLAT-002 is resolved — R25 and CI.** `pnpm/action-setup` now references `b906affcce14559ad1aafd4ab0e942779e9f58b1`. A fresh independent `git ls-remote` check returned that value as the peeled commit for `v4.3.0`; the workflow no longer uses the annotated-tag object. Its comment also identifies the correct version.
- **PLAT-N01 remains resolved — S0 checks.** The local aggregate includes Rust format, all-target Clippy and locked tests; it passed in this round.
- **PLAT-N02 is resolved at the CI-design level — R03/R25.** The Linux-only preparation installs the pinned Electron distribution, derives the adjacent helper from its executable, requires a regular nonsymlink file, and sets only that helper to root ownership and mode 4755. Electron itself runs as the ordinary runner user. The workflow uses hosted VM labels, an X11 hint and Xvfb, without disabling sandboxing or changing global AppArmor/sysctl policy. The embedded Bash passed syntax checking. I did not run these privileged preparation commands on the local machine; the actual hosted-runner result remains a bootstrap check.
- **Windows hash reproducibility is addressed.** `.gitattributes` is now in review scope and specifies LF checkout for text. I checked all 16 hashed inputs: their current bytes contain no CR, and `git check-attr eol` reports `lf` for each. This removes the ordinary automatic CRLF conversion risk for the reviewed text bundle. A real Windows CI result is still required, not inferred from this static check.

## Final platform and governance assessment

The five primary client OS families remain in scope with explicit native qualification boundaries and staged execution capabilities. Electron/Rust lifecycle separation, native process semantics, mobile foreground catch-up, approval presentation, cache/key lifecycle and preview isolation remain implementable requirements. S0 still claims no execution, enrollment, credentials or enforcement. Source builds and mobile exports do not count as native device evidence.

The 12-person plan remains a gated planning model rather than a GA commitment. Linux, Windows and macOS source CI is required; installed desktop and physical mobile tests remain additional product gates. Later managed execution, self-hosting and shared native pools do not displace the five-platform beta.

R25 and the contributor rules continue to require GitHub issues, isolated work, exact-head independent expert records, required CI and an eligible non-author/latest-push GitHub approval. Shared-account agents cannot manufacture that native approval. A PR without an eligible reviewer stays unmerged. Normative approval files cannot substitute for review of later implementation code. The single initial seed is conditioned on unanimous approval of this exact final scope and immediately followed by live protection read-back and CI inspection; no public repository or applied protection is implied by this review.

## Evidence and limits

- `node scripts/spec-hash.mjs`: matched the recorded final hash before and after review/checks.
- `git ls-remote` for `pnpm/action-setup` `v4.3.0` and its peeled commit: confirmed the corrected pin.
- All reviewed file bytes and Git EOL attributes: LF checks passed.
- Extracted sandbox preparation with `bash -n`: passed; no privileged commands executed.
- `pnpm verify`: passed, including specification structure, cached unchanged TypeScript/tests/build tasks, Rust formatting, Clippy and executed Cargo tests.
- `pnpm check:approvals`: exited 1 because the integrator's aggregate `docs/reviews/approvals.json` does not yet exist. This individual approval is recorded; unanimous aggregate approval is not claimed by this reviewer.

The local checks do not qualify GitHub-hosted Node 24 runners, Windows/macOS native behavior, native Android/iOS builds or devices, signed installers, store acceptance, process survival, runtime isolation or live repository rules. Those remain explicit implementation/bootstrap/release gates. Only this round's assigned review records were authored.
