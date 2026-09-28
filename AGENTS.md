# Conductor contributor contract

Read SPEC.md, STATE.md, docs/implementation-plan.md and docs/development-workflow.md before changes.

STRICT RULE: Conductor development must happen through GitHub issues, isolated branches/worktrees and pull requests. After the single approved bootstrap, never push directly to main, bypass protection or self-approve. Every PR requires independent parallel systems, security and platform expert approvals on the exact current head, required CI and an eligible non-author GitHub approval. Agents using one account are not distinct GitHub reviewers. Follow CONTRIBUTING.md and docs/development-workflow.md; never silently substitute chat-only completion for the GitHub record.

SPEC.md and contracts/ are normative; docs/reference/ is historical rationale only. This is a new independent implementation. Do not copy Superset implementation code or brand assets.

Keep all five targets in scope. Source typechecking is not native-platform qualification. The S0 scaffold intentionally cannot execute agents, authorize tools, enroll devices, store credentials or mutate repositories. Unsupported execution must remain unavailable until the relevant SPEC requirement and release gates are implemented.

Use pnpm with its committed lockfile and Cargo with Cargo.lock. From the root run `pnpm verify`; review changes also run `pnpm check:approvals`. No install-script blanket approval, production credentials, real cloud provisioning, publishing or app-store submission is part of ordinary scaffolding.

Authority, privacy, durable command acknowledgement and execution isolation are protocol concerns, never renderer conveniences. Keep model/provider adapters and untrusted work outside privileged processes. Do not make fake success or insecure fallbacks to pass tests.

Review scope is docs/reviews/scope.json. Approval binds to the exact scope hash from `pnpm spec:hash`. Spec or contract changes invalidate approval. Independent reviewers write only their assigned files in docs/reviews; the author resolves findings and requests a recheck. APPROVE means ready to implement the specification, not tested production security. Never change a reviewer's verdict yourself.

Mounted disk UUID: 6b9f257c-a863-4dab-94d6-70be1e10b526. Canonical project path: /mnt/data/Desktop Apps/Conductor. The user-selected /run/media/its1deat0r/Projects alias is the same SSD. Parent catalog tooling was absent on 2026-09-28; STATE.md records the pending registration without modifying the global generated ledger.
