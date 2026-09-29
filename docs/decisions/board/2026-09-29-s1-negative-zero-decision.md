# S1 negative-zero input rule — Critical board decision

Date: 2026-09-29
Status: Adopted
Decision ID: `S1_JCS_NEGATIVE_ZERO`
Tier: Critical

## Decision

Select `REJECT_NEGATIVE_ZERO`. Before schema validation, canonical digest calculation, replay-fingerprint comparison, or command admission, reject any raw JSON number token whose correctly rounded IEEE-754 binary64 value has a negative sign and zero magnitude. Apply the rule recursively to all JSON values, including number tokens in nested arrays and objects. This covers `-0`, `-0.0`, `-0e0`, equivalent exponent spellings, and negative nonzero decimal spellings that underflow to negative zero. Positive zero remains governed by the Issue #3 safe-integer rule. A JSON string containing `"-0"` is not a number and is unaffected.

The RFC Editor's verified technical erratum 7920 recommends a parser error for `-0` because JCS serializes signed zero as `0`. The board extends that defensive rule to every spelling that decodes to IEEE-754 negative zero. This is a Conductor protocol requirement, not a claim that RFC 8785 explicitly lists the other spellings. The command digest and submission replay fingerprint remain integrity and deduplication inputs; neither is authorization evidence.

This decision does not expose a route, admit a command, start a process, authorize tools, store credentials, or qualify security/platform/release behavior. Those gates remain in SPEC and the S1 acceptance evidence.

## Reviewed artifact and adoption event

The five seats reviewed the immutable proposal [`2026-09-29-s1-negative-zero-proposal-v1.md`](2026-09-29-s1-negative-zero-proposal-v1.md), Version 1, SHA-256 `719385f4f82d87c6f41f8365775b30aa3304cf94e3a94fddc2b3bc2d498a4fb8`. Its bytes were committed unchanged in adoption commit `84fdbbfea1e7584c37a411afd1074c15ac4ebe26`. The local diff passed `git diff --cached --check`; `pnpm verify` passed (exit 0) before that commit. This record carries the outcome separately and does not alter the reviewed proposal.

Exact isolated prompts are preserved in [`2026-09-29-s1-negative-zero-prompts.md`](2026-09-29-s1-negative-zero-prompts.md). The reviewers received public repository/specification and public standards material only, on the local Codex execution host; no credentials, tenant data, private user content, production access, or new provider routing was involved. Seats ran in concurrency-limited waves against the same artifact hash. Later seats did not receive earlier votes. Codex subagents are separate agent contexts, not independent human reviewers or GitHub identities.

## Critical board round 1

| Mandatory seat | Model/tool | `REJECT_NEGATIVE_ZERO` | `CANONICALIZE_AS_ZERO` | `DEFER_JCS_INPUTS` | Concise rationale |
|---|---|---|---|---|---|
| Domain/product | gpt-6-astra, Codex subagent | BUILD (1) | REJECT (2) | REJECT (3) | Malformed negative-zero submissions are rejected before acceptance, preserving R07 replay semantics; accepted command IDs stay unambiguous. |
| Systems architecture | gpt-5.6-sol, Codex subagent | BUILD (1) | REJECT (2) | REJECT (3) | A token-aware stage before ordinary parsing lets TS and Rust share the same negative-zero rule without losing lexical sign information. |
| Security/privacy | gpt-6-sol, Codex subagent | BUILD (1) | REJECT (3) | CONDITIONAL (2) | Rejection avoids merging distinct signed-zero source inputs in durable fingerprints; digest identity still does not grant authority. |
| Platform/reliability | gpt-5.6-terra, Codex subagent | BUILD (1) | REJECT (3) | CONDITIONAL (2) | The rule is implementable consistently before schema/JCS processing; bound token/exponent work and assert stable rejection across platforms. |
| Adversarial | gpt-6-luna, Codex subagent | BUILD (1) | REJECT (2) | REJECT (3) | No concrete blocker to early raw-token rejection; deferral leaves the selected durable identity contract unresolved. |

All five mandatory seats returned BUILD for the same option, satisfying critical unanimity. No material blocker remains against the selected option.

## Blocker adjudication

1. Security's blocker against `DEFER_JCS_INPUTS` is confirmed for that option: it defers the rule and shared vectors required by proposal acceptance criteria 1–2. The adopted option specifies the rule now. Issue #3 and its TS/Rust vectors remain required before digest-dependent command behavior is implemented; this is a downstream implementation gate, not an open blocker to the selected design.
2. Platform/reliability's blocker against `CANONICALIZE_AS_ZERO` is confirmed for that option: accepting negative-zero inputs into canonical identity would rely on later consumers preserving an additional normalization invariant, despite RFC 8785's signed-zero collapse and the erratum's parser-error recommendation. The adopted option closes it by rejecting before any durable identity calculation.
3. Domain/product, systems architecture, and adversarial reported no blockers against the selected option. All material conditions from the other ballots concern unselected options or implementation evidence and are closed for this design decision; implementation recommendations remain recorded below.

## Implementation notes

- Inspect the raw numeric token before a parser or serializer can erase its sign. Detection must cover nested values and negative nonzero lexemes that round to negative zero.
- Use byte-for-byte shared TS/Rust fixtures for negative-zero forms, underflow, positive-zero controls, string `"-0"`, duplicate names, and Unicode preservation. Assert stable error categories rather than parser-specific prose.
- Bound raw input, token length, exponent handling, and nesting so malformed or oversized input fails closed without platform-dependent resource behavior.
- Retain duplicate-property rejection, Unicode preservation without normalization, the safe-integer constraint, the two distinct SHA-256 domain separators, authenticated principal derivation, and same-ID/different-content rejection.

## Evidence checked by the facilitator

- [RFC 8785, JSON Canonicalization Scheme](https://www.rfc-editor.org/rfc/rfc8785.html): §3.1 requires I-JSON input; Appendix B serializes both IEEE signed zeros as `0`.
- [RFC Editor Erratum 7920](https://www.rfc-editor.org/errata/eid7920): Verified technical erratum; recommends (`SHOULD`) an error when a parser encounters `-0` to prevent attacks on not-yet-parsed data. The erratum itself does not enumerate other equivalent lexemes; their rejection is the board's selected conservative extension.
- `SPEC.md` R06–R07 and Issue #3: typed cross-language contracts, fail-closed handling, durable same-ID replay and changed-content rejection.
- `pnpm verify`: passed before adoption commit `84fdbbfea1e7584c37a411afd1074c15ac4ebe26`. This verifies the repository at adoption time, not the future S1 protocol implementation or its platform/security gates.
