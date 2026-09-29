# Protocol source of truth

health-v1.schema.json is the sole executable S0 wire contract. Both TypeScript validation and Rust fixture tests cover it. schema_version is a protocol version, not a package version. S0 strictly rejects unknown properties and unknown versions. This envelope advertises no execution capability.

Future command, action, artifact and event envelopes are normative requirements in SPEC.md. They are not implemented schemas yet. Add schemas and cross-language valid/invalid conformance vectors before exposing those endpoints. Runtime input validation and semantic authorization remain separate. No TypeScript type alone authorizes an operation.

`command-json-inputs.v1.json` is a pre-schema parser conformance fixture for the S1 command-contract work; it is not a command schema or admission path. TypeScript `parseJsonInput` and Rust `parse_json_input` reject duplicate decoded property names, invalid JSON/Unicode, non-finite or non-safe-integer numbers, and every raw number token that rounds to IEEE-754 negative zero. These checks run before schema validation or durable identity calculation and preserve string contents without Unicode normalization.

Callers must supply a byte limit from their trusted transport or operation policy. Both parsers enforce the same nesting limit supplied by the caller, capped at 128. The byte bound also bounds number-token and exponent processing. Error categories are shared across languages; parser-specific error text is not part of the contract. RFC 8785 digest generation, command schemas, authority checks, HTTP/IPC wiring, and command admission remain separate work; S0 still exposes only `health-v1` and keeps execution unavailable.
