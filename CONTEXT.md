# Conductor domain glossary

This file records shared product vocabulary only. Product requirements and versioned contracts remain normative.

- **Command submission**: A client's requested operation before trusted server context is attached. Client-supplied tenant, actor, or device values do not establish authority.
- **Command**: The immutable, versioned operation after the authority derives its tenant, actor, and device identity and assigns any new aggregate identifiers.
- **Command digest**: The integrity fingerprint of the server-canonical command. It is distinct from authorization and from the submission replay fingerprint.
- **Submission replay fingerprint**: The identity of one submitted request together with its authenticated principals. An identical retry returns the existing receipt; changed content or principal is rejected.
- **Coordinator receipt**: Durable evidence of the coordinator's state. It does not imply that a host accepted or executed the command.
- **Host receipt**: Durable evidence scoped to the host inbox and its execution lifecycle. It does not imply coordinator state beyond the evidence it carries.
- **Effect resolution (`effect_resolved`)**: A durable terminal outcome with stable resolution evidence. It can record failure or uncertainty as well as success.
- **Unknown outcome (`outcome_unknown`)**: An effect whose result cannot be established from durable evidence. It requires reconciliation and is not success, running, or authorization for automatic replay.
