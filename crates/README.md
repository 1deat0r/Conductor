# Execution substrate

`conductor-host doctor` and `conductor-supervisor doctor` remain diagnostic commands. They create no sockets, processes, repositories, databases, credentials or host enrollment. Unsupported invocations exit 2. The shared protocol crate validates cross-language fixtures.

The first S1 storage slice adds opt-in local libraries: the host owns a SQLite command journal, and the supervisor owns a separate SQLite session-receipt database. Both require an explicit path and configure WAL with synchronous FULL. Command state and outbox events commit atomically; supervisor start intent is durable before any future spawn. Replays cannot relaunch a session whose result is uncertain, and cancellation request and confirmation remain separate receipts.

These libraries are not wired to a CLI, API or renderer. The supervisor does not spawn or own OS processes yet, and health still advertises execution as unavailable. Authenticated IPC, process-tree ownership, controller restart adoption, OS lifecycle qualification and PTY implementations remain later S1 work. Do not infer process persistence from these entry points.
