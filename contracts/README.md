# Protocol source of truth

health-v1.schema.json is the sole executable S0 wire contract. Both TypeScript validation and Rust fixture tests cover it. schema_version is a protocol version, not a package version. S0 strictly rejects unknown properties and unknown versions. This envelope advertises no execution capability.

Future command, action, artifact and event envelopes are normative requirements in SPEC.md. They are not implemented schemas yet. Add schemas and cross-language valid/invalid conformance vectors before exposing those endpoints. Runtime input validation and semantic authorization remain separate. No TypeScript type alone authorizes an operation.
