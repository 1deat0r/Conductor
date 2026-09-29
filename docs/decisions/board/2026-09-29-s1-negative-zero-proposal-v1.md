# Critical board proposal: negative zero in S1 command fingerprints

Version: 1
Date: 2026-09-29
Tier: Critical
Decision ID: `S1_JCS_NEGATIVE_ZERO`
Status: PROPOSED
Data class: Public project-contract and public standards material; no credentials, tenant data, or private user content.

## Goal

Choose one cross-language parsing rule for negative-zero JSON numbers in the RFC 8785 canonical digest and replay-fingerprint inputs specified by [Issue #3](https://github.com/1deat0r/Conductor/issues/3). The decision changes durable command identity and duplicate/replay behavior, so it is critical under `docs/agents/decision-board.md`.

## Fixed context

- SPEC R06 requires versioned TS/Rust protocol contracts, cross-language valid/invalid vectors, and fail-closed unsupported inputs. R07 requires same-ID/same-payload replay to return prior state and changed-payload replay to fail (`SPEC.md` §§R06–R07, lines 43–49).
- Issue #3 defines `CommandV1` payload digest as SHA-256 over `Conductor.CommandV1\0 || JCS(command_without_payload_digest)`. It separately defines the submission replay fingerprint as SHA-256 over `Conductor.CommandSubmissionV1\0 || JCS({tenant_id, actor_id, device_id, submission})`. It requires duplicate-key rejection, preservation of Unicode without normalization, safe-integer-only JSON numbers, and byte-for-byte TS/Rust vectors.
- RFC 8785 says JCS input must have no duplicate property names, must use IEEE-754-compatible numbers, and must preserve Unicode as-is. JCS serializes negative zero as `0` (RFC 8785 §§3.1–3.2.2.3).
- RFC Editor verified technical erratum 7920 says a parser following RFC 8785 SHOULD report an error on `-0`, because serialization as `0` can create attacks on data not yet parsed. The erratum does not prescribe how equivalent lexical spellings such as `-0.0` or `-0e0` are handled; this proposal asks the board to make the cross-language rule explicit.
- RFC 8785 is an Informational RFC, not an Internet Standards Track specification. The project adopts it as the issue's explicitly chosen digest format; this decision does not make the digest an authorization proof.
- Current code has only the S0 health schema and fixture validators. Issue #3 explicitly forbids endpoint wiring, host admission, process launch, credentials, or execution enablement in that slice. Preserve `execution_available: false`.
- Current Conductor review policy is controlling: use the critical expert board for this design choice; retain the initial `docs/reviews/scope.json` archive and do not reinstate the retired `check:approvals` gate. Board approval is not security qualification.

## Mutually exclusive options

### `REJECT_NEGATIVE_ZERO`

Reject any raw JSON numeric token whose IEEE-754 value is negative zero before schema validation, digest calculation, or replay-fingerprint comparison. This includes equivalent spellings such as `-0`, `-0.0`, and `-0e0`; positive-zero spellings remain governed by the safe-integer rule. Add shared invalid vectors for the rejected forms and matching TS/Rust behavior.

### `CANONICALIZE_AS_ZERO`

Accept negative-zero tokens when they meet the issue's numeric constraints, and treat them as numeric zero before semantic handling. RFC 8785 canonicalizes them as `0`; add shared vectors proving the resulting canonical digest/fingerprint and application value are identical to positive zero.

### `DEFER_JCS_INPUTS`

Defer the digest and replay-fingerprint input implementation until a parser strategy can prove identical handling in TypeScript and Rust. Schemas may still be implemented without exposing a digest-dependent admission path; no command route or execution becomes available.

## Acceptance for the winning option

1. The selected rule is stated precisely in Issue #3 and its protocol contract.
2. TypeScript and Rust parse/validate the same positive and negative-zero cases with byte-for-byte matching golden results or identical rejection.
3. The decision does not weaken the issue's duplicate-key, Unicode-preservation, safe-integer, digest-domain-separation, authenticated-principal, or same-ID/different-content rules.
4. No endpoint, IPC path, command admission, process launch, tool authorization, credential storage, or S0 execution capability is enabled by this decision.
5. The implementation still needs its own tests, code review, `pnpm verify`, and exact-commit required CI before protected-main integration. Separate SPEC security/platform/release gates remain in force.

## Evidence

- [RFC 8785, JSON Canonicalization Scheme](https://www.rfc-editor.org/rfc/rfc8785.html), especially §§3.1 and 3.2.2.3.
- [RFC Editor verified erratum 7920](https://errata.rfc-editor.org/eid7920/).
- [Conductor Issue #3: typed command and durable receipt contracts](https://github.com/1deat0r/Conductor/issues/3).
- Repository requirements and gates: `SPEC.md` §§R06–R08, `contracts/README.md`, `docs/implementation-plan.md`, and `docs/agents/decision-board.md`.
