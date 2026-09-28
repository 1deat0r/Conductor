# Execution substrate

`conductor-host doctor` and `conductor-supervisor doctor` print a truthful versioned scaffold envelope. They create no sockets, processes, repositories, databases, credentials or host enrollment. Unsupported invocations exit 2. The shared protocol crate validates cross-language fixtures.

The initial S1 foundation in `conductor-supervisor` is a file-backed SQLite launch-intent journal with WAL/FULL durability and restart reconciliation. It stops before process spawn: no real agent launch or start/exit receipt is implemented. Remaining S1 work includes the durable inbox, complete Start/Query/Cancel receipts and fault injection across acknowledgement and spawn crash windows. Controller/supervisor process lifetime, upgrades and OS-specific PTY implementations remain future work. Do not infer process persistence from these entry points.
