use std::{error::Error, fmt, path::Path, sync::Mutex, time::Duration};

use rusqlite::{
    Connection, OpenFlags, OptionalExtension, Transaction, TransactionBehavior, params,
};
use sha2::{Digest, Sha256};

const DATABASE_VERSION: i64 = 2;
const MAX_IDENTIFIER_BYTES: usize = 256;
const MAX_COMMAND_PAYLOAD_BYTES: usize = 1_048_576;
const MAX_OUTBOX_BATCH_SIZE: usize = 256;

/// Immutable command data accepted by the local host journal.
///
/// `payload_json` is stored byte-for-byte. Its digest also covers every field
/// below, so reusing a command ID with a changed target, actor, revision,
/// deadline, schema version, or payload is rejected.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommandEnvelope {
    pub schema_version: u8,
    pub command_id: String,
    pub tenant_id: String,
    pub task_id: String,
    pub attempt_id: String,
    pub expected_revision: i64,
    pub actor_id: String,
    pub device_id: String,
    pub deadline_unix_ms: i64,
    pub payload_json: Vec<u8>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CommandState {
    PendingHostAdmission,
    HostAccepted,
    HostRejected,
    EffectResolved,
}

impl CommandState {
    fn as_str(self) -> &'static str {
        match self {
            Self::PendingHostAdmission => "pending_host_admission",
            Self::HostAccepted => "host_accepted",
            Self::HostRejected => "host_rejected",
            Self::EffectResolved => "effect_resolved",
        }
    }

    fn parse(value: &str) -> Result<Self, JournalError> {
        match value {
            "pending_host_admission" => Ok(Self::PendingHostAdmission),
            "host_accepted" => Ok(Self::HostAccepted),
            "host_rejected" => Ok(Self::HostRejected),
            "effect_resolved" => Ok(Self::EffectResolved),
            _ => Err(JournalError::CorruptState(value.to_owned())),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HostRejection {
    Expired,
    StaleRevision,
}

impl HostRejection {
    fn as_str(self) -> &'static str {
        match self {
            Self::Expired => "expired",
            Self::StaleRevision => "stale_revision",
        }
    }

    fn parse(value: &str) -> Result<Self, JournalError> {
        match value {
            "expired" => Ok(Self::Expired),
            "stale_revision" => Ok(Self::StaleRevision),
            _ => Err(JournalError::CorruptState(value.to_owned())),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommandReceipt {
    pub command: CommandEnvelope,
    pub payload_digest: String,
    pub state: CommandState,
    pub rejection: Option<HostRejection>,
    pub effect_digest: Option<String>,
    pub state_updated_at_ms: i64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommandOutboxEvent {
    pub event_id: i64,
    pub command_id: String,
    pub aggregate_id: String,
    pub schema_version: u8,
    pub event_type: String,
    pub occurred_at_ms: i64,
    pub delivered_at_ms: Option<i64>,
}

#[derive(Debug)]
pub enum JournalError {
    Storage(rusqlite::Error),
    InvalidInput(&'static str),
    CommandNotFound,
    PayloadMismatch,
    InvalidTransition {
        state: CommandState,
        requested: &'static str,
    },
    CorruptState(String),
    UnsupportedDatabaseVersion(i64),
    UnsafeDatabaseConfiguration(&'static str),
    LockPoisoned,
}

impl fmt::Display for JournalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Storage(error) => write!(f, "local command journal storage error: {error}"),
            Self::InvalidInput(message) => write!(f, "invalid command journal input: {message}"),
            Self::CommandNotFound => f.write_str("command not found"),
            Self::PayloadMismatch => {
                f.write_str("command ID was already used with different immutable content")
            }
            Self::InvalidTransition { state, requested } => {
                write!(f, "cannot {requested} a command in state {state:?}")
            }
            Self::CorruptState(state) => write!(f, "unknown persisted command state: {state}"),
            Self::UnsupportedDatabaseVersion(version) => {
                write!(f, "unsupported command journal schema version: {version}")
            }
            Self::UnsafeDatabaseConfiguration(setting) => {
                write!(f, "SQLite did not enable required setting {setting}")
            }
            Self::LockPoisoned => f.write_str("local command journal lock was poisoned"),
        }
    }
}

impl Error for JournalError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Storage(error) => Some(error),
            _ => None,
        }
    }
}

impl From<rusqlite::Error> for JournalError {
    fn from(error: rusqlite::Error) -> Self {
        Self::Storage(error)
    }
}

/// One writer connection owned by the local host controller.
///
/// Host inbox state is committed before a receipt is returned. Each transition
/// that emits an outbox event commits that event with the state change. The
/// caller may send an outbox event and mark it delivered only after a
/// successful send; a crash between send and mark can produce a duplicate,
/// consistent with at-least-once delivery. This module does not implement
/// authorization or execute effects.
pub struct CommandJournal {
    connection: Mutex<Connection>,
}

impl CommandJournal {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, JournalError> {
        let flags = OpenFlags::SQLITE_OPEN_READ_WRITE
            | OpenFlags::SQLITE_OPEN_CREATE
            | OpenFlags::SQLITE_OPEN_FULL_MUTEX;
        let connection = Connection::open_with_flags(path, flags)?;
        Self::from_connection(connection)
    }

    fn from_connection(mut connection: Connection) -> Result<Self, JournalError> {
        configure_connection(&connection)?;
        migrate(&mut connection)?;
        Ok(Self {
            connection: Mutex::new(connection),
        })
    }

    /// Commit a command to the local host inbox before returning a receipt.
    /// This is not a `coordinator_stored` acknowledgement: only the
    /// coordinator can emit that receipt. The caller must deliver the
    /// coordinator's stored command to this inbox, then call
    /// `record_host_admission` before emitting `host_accepted`.
    ///
    /// A replay with the same immutable command returns its prior state, even
    /// when the original deadline has since passed. Expired commands are
    /// stored so the host can durably reject them.
    pub fn store_host_command(
        &self,
        command: &CommandEnvelope,
        now_unix_ms: i64,
    ) -> Result<CommandReceipt, JournalError> {
        validate_command(command)?;
        if now_unix_ms < 0 {
            return Err(JournalError::InvalidInput("negative current time"));
        }
        let digest = command_digest(command);
        let mut connection = self
            .connection
            .lock()
            .map_err(|_| JournalError::LockPoisoned)?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;

        if let Some(existing) = read_receipt(&transaction, &command.command_id)? {
            if existing.payload_digest != hex(&digest) {
                return Err(JournalError::PayloadMismatch);
            }
            transaction.commit()?;
            return Ok(existing);
        }
        transaction.execute(
            "INSERT INTO local_commands (
                command_id, payload_digest, schema_version, tenant_id, task_id,
                attempt_id, expected_revision, actor_id, device_id, deadline_unix_ms,
                payload_json, state, rejection, effect_digest, state_updated_at_ms
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11,
                      'pending_host_admission', NULL, NULL, ?12)",
            params![
                command.command_id,
                digest.as_slice(),
                i64::from(command.schema_version),
                command.tenant_id,
                command.task_id,
                command.attempt_id,
                command.expected_revision,
                command.actor_id,
                command.device_id,
                command.deadline_unix_ms,
                command.payload_json,
                now_unix_ms,
            ],
        )?;
        let receipt = read_receipt(&transaction, &command.command_id)?
            .ok_or(JournalError::CommandNotFound)?;
        transaction.commit()?;
        Ok(receipt)
    }

    /// Durably record the host's admission decision. `current_task_revision`
    /// must come from the sole authoritative task writer, and the caller must
    /// serialize this call with that writer's task-state updates. No public S0
    /// entrypoint invokes this method yet.
    pub fn record_host_admission(
        &self,
        command_id: &str,
        current_task_revision: i64,
        now_unix_ms: i64,
    ) -> Result<CommandReceipt, JournalError> {
        if command_id.is_empty() || current_task_revision < 0 || now_unix_ms < 0 {
            return Err(JournalError::InvalidInput("invalid host admission context"));
        }
        let mut connection = self
            .connection
            .lock()
            .map_err(|_| JournalError::LockPoisoned)?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let existing =
            read_receipt(&transaction, command_id)?.ok_or(JournalError::CommandNotFound)?;
        if existing.state != CommandState::PendingHostAdmission {
            transaction.commit()?;
            return Ok(existing);
        }

        let (state, rejection) = if existing.command.deadline_unix_ms <= now_unix_ms {
            (CommandState::HostRejected, Some(HostRejection::Expired))
        } else if existing.command.expected_revision != current_task_revision {
            (
                CommandState::HostRejected,
                Some(HostRejection::StaleRevision),
            )
        } else {
            (CommandState::HostAccepted, None)
        };
        transaction.execute(
            "UPDATE local_commands SET state = ?2, rejection = ?3,
                    state_updated_at_ms = ?4
             WHERE command_id = ?1 AND state = 'pending_host_admission'",
            params![
                command_id,
                state.as_str(),
                rejection.map(HostRejection::as_str),
                now_unix_ms
            ],
        )?;
        insert_outbox_event(&transaction, &existing.command, state.as_str(), now_unix_ms)?;
        let receipt =
            read_receipt(&transaction, command_id)?.ok_or(JournalError::CommandNotFound)?;
        transaction.commit()?;
        Ok(receipt)
    }

    /// Persist effect resolution separately from host admission. This method
    /// records evidence supplied by the actual executor; it never runs the
    /// operation itself.
    // Reserved for the effect executor; no S0 entrypoint can resolve effects.
    #[allow(dead_code)]
    pub(crate) fn record_effect_resolved(
        &self,
        command_id: &str,
        resolution_evidence: &[u8],
        now_unix_ms: i64,
    ) -> Result<CommandReceipt, JournalError> {
        if command_id.is_empty() || resolution_evidence.is_empty() || now_unix_ms < 0 {
            return Err(JournalError::InvalidInput("invalid effect resolution"));
        }
        let evidence_digest = Sha256::digest(resolution_evidence);
        let mut connection = self
            .connection
            .lock()
            .map_err(|_| JournalError::LockPoisoned)?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let existing =
            read_receipt(&transaction, command_id)?.ok_or(JournalError::CommandNotFound)?;
        if existing.state == CommandState::EffectResolved {
            if existing.effect_digest.as_deref() != Some(hex(&evidence_digest).as_str()) {
                return Err(JournalError::PayloadMismatch);
            }
            transaction.commit()?;
            return Ok(existing);
        }
        if existing.state != CommandState::HostAccepted {
            return Err(JournalError::InvalidTransition {
                state: existing.state,
                requested: "resolve effect for",
            });
        }
        transaction.execute(
            "UPDATE local_commands SET state = 'effect_resolved',
                    effect_digest = ?2, state_updated_at_ms = ?3
             WHERE command_id = ?1 AND state = 'host_accepted'",
            params![command_id, evidence_digest.as_slice(), now_unix_ms],
        )?;
        insert_outbox_event(
            &transaction,
            &existing.command,
            "effect_resolved",
            now_unix_ms,
        )?;
        let receipt =
            read_receipt(&transaction, command_id)?.ok_or(JournalError::CommandNotFound)?;
        transaction.commit()?;
        Ok(receipt)
    }

    pub fn query(&self, command_id: &str) -> Result<Option<CommandReceipt>, JournalError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| JournalError::LockPoisoned)?;
        read_receipt(&connection, command_id)
    }

    pub fn pending_outbox_events(
        &self,
        limit: usize,
    ) -> Result<Vec<CommandOutboxEvent>, JournalError> {
        if limit == 0 || limit > MAX_OUTBOX_BATCH_SIZE {
            return Err(JournalError::InvalidInput(
                "outbox page size is outside the supported range",
            ));
        }
        let connection = self
            .connection
            .lock()
            .map_err(|_| JournalError::LockPoisoned)?;
        let mut statement = connection.prepare(
            "SELECT event_id, command_id, aggregate_id, schema_version,
                    event_type, occurred_at_ms, delivered_at_ms
             FROM local_command_outbox WHERE delivered_at_ms IS NULL ORDER BY event_id LIMIT ?1",
        )?;
        let rows = statement.query_map([limit as i64], |row| {
            let schema_version: i64 = row.get(3)?;
            let schema_version = u8::try_from(schema_version)
                .map_err(|_| rusqlite::Error::IntegralValueOutOfRange(3, schema_version))?;
            Ok(CommandOutboxEvent {
                event_id: row.get(0)?,
                command_id: row.get(1)?,
                aggregate_id: row.get(2)?,
                schema_version,
                event_type: row.get(4)?,
                occurred_at_ms: row.get(5)?,
                delivered_at_ms: row.get(6)?,
            })
        })?;
        rows.map(|row| row.map_err(JournalError::Storage)).collect()
    }

    /// Mark only after the consumer confirms receipt. A crash after send and
    /// before this write intentionally permits redelivery.
    pub fn mark_outbox_delivered(
        &self,
        event_id: i64,
        delivered_at_ms: i64,
    ) -> Result<(), JournalError> {
        if event_id <= 0 || delivered_at_ms < 0 {
            return Err(JournalError::InvalidInput("invalid outbox receipt"));
        }
        let connection = self
            .connection
            .lock()
            .map_err(|_| JournalError::LockPoisoned)?;
        let changed = connection.execute(
            "UPDATE local_command_outbox SET delivered_at_ms = ?2
             WHERE event_id = ?1 AND delivered_at_ms IS NULL",
            params![event_id, delivered_at_ms],
        )?;
        if changed == 0 {
            let exists: bool = connection.query_row(
                "SELECT EXISTS(SELECT 1 FROM local_command_outbox WHERE event_id = ?1)",
                [event_id],
                |row| row.get(0),
            )?;
            if !exists {
                return Err(JournalError::InvalidInput("unknown outbox event"));
            }
        }
        Ok(())
    }
}

fn configure_connection(connection: &Connection) -> Result<(), JournalError> {
    connection.busy_timeout(Duration::from_secs(5))?;
    let journal_mode: String =
        connection.query_row("PRAGMA journal_mode = WAL", [], |row| row.get(0))?;
    if !journal_mode.eq_ignore_ascii_case("wal") {
        return Err(JournalError::UnsafeDatabaseConfiguration(
            "journal_mode=WAL",
        ));
    }
    connection.pragma_update(None, "synchronous", "FULL")?;
    let synchronous: i64 = connection.query_row("PRAGMA synchronous", [], |row| row.get(0))?;
    if synchronous != 2 {
        return Err(JournalError::UnsafeDatabaseConfiguration(
            "synchronous=FULL",
        ));
    }
    connection.pragma_update(None, "foreign_keys", "ON")?;
    let foreign_keys: i64 = connection.query_row("PRAGMA foreign_keys", [], |row| row.get(0))?;
    if foreign_keys != 1 {
        return Err(JournalError::UnsafeDatabaseConfiguration("foreign_keys=ON"));
    }
    Ok(())
}

fn migrate(connection: &mut Connection) -> Result<(), JournalError> {
    let version: i64 = connection.pragma_query_value(None, "user_version", |row| row.get(0))?;
    match version {
        0 => {
            let transaction =
                connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
            transaction.execute_batch(
                "CREATE TABLE local_commands (
                    command_id TEXT PRIMARY KEY NOT NULL,
                    payload_digest BLOB NOT NULL CHECK(length(payload_digest) = 32),
                    schema_version INTEGER NOT NULL CHECK(schema_version BETWEEN 1 AND 255),
                    tenant_id TEXT NOT NULL,
                    task_id TEXT NOT NULL,
                    attempt_id TEXT NOT NULL,
                    expected_revision INTEGER NOT NULL CHECK(expected_revision >= 0),
                    actor_id TEXT NOT NULL,
                    device_id TEXT NOT NULL,
                    deadline_unix_ms INTEGER NOT NULL CHECK(deadline_unix_ms >= 0),
                    payload_json BLOB NOT NULL,
                    state TEXT NOT NULL CHECK(state IN (
                        'pending_host_admission', 'host_accepted', 'host_rejected', 'effect_resolved'
                    )),
                    rejection TEXT CHECK(rejection IN ('expired', 'stale_revision') OR rejection IS NULL),
                    effect_digest BLOB CHECK(effect_digest IS NULL OR length(effect_digest) = 32),
                    state_updated_at_ms INTEGER NOT NULL CHECK(state_updated_at_ms >= 0)
                );
                CREATE INDEX local_commands_by_task ON local_commands(task_id, state);
                CREATE TABLE local_command_outbox (
                    event_id INTEGER PRIMARY KEY AUTOINCREMENT,
                    command_id TEXT NOT NULL REFERENCES local_commands(command_id),
                    aggregate_id TEXT NOT NULL,
                    schema_version INTEGER NOT NULL CHECK(schema_version BETWEEN 1 AND 255),
                    event_type TEXT NOT NULL,
                    occurred_at_ms INTEGER NOT NULL CHECK(occurred_at_ms >= 0),
                    delivered_at_ms INTEGER CHECK(delivered_at_ms IS NULL OR delivered_at_ms >= 0),
                    UNIQUE(command_id, event_type)
                );
                PRAGMA user_version = 2;",
            )?;
            transaction.commit()?;
            Ok(())
        }
        1 => {
            let transaction =
                connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
            transaction.execute_batch(
                "ALTER TABLE local_command_outbox RENAME TO local_command_outbox_v1;
                ALTER TABLE local_commands RENAME TO local_commands_v1;
                DROP INDEX local_commands_by_task;
                CREATE TABLE local_commands (
                    command_id TEXT PRIMARY KEY NOT NULL,
                    payload_digest BLOB NOT NULL CHECK(length(payload_digest) = 32),
                    schema_version INTEGER NOT NULL CHECK(schema_version BETWEEN 1 AND 255),
                    tenant_id TEXT NOT NULL,
                    task_id TEXT NOT NULL,
                    attempt_id TEXT NOT NULL,
                    expected_revision INTEGER NOT NULL CHECK(expected_revision >= 0),
                    actor_id TEXT NOT NULL,
                    device_id TEXT NOT NULL,
                    deadline_unix_ms INTEGER NOT NULL CHECK(deadline_unix_ms >= 0),
                    payload_json BLOB NOT NULL,
                    state TEXT NOT NULL CHECK(state IN (
                        'pending_host_admission', 'host_accepted', 'host_rejected', 'effect_resolved'
                    )),
                    rejection TEXT CHECK(rejection IN ('expired', 'stale_revision') OR rejection IS NULL),
                    effect_digest BLOB CHECK(effect_digest IS NULL OR length(effect_digest) = 32),
                    state_updated_at_ms INTEGER NOT NULL CHECK(state_updated_at_ms >= 0)
                );
                CREATE INDEX local_commands_by_task ON local_commands(task_id, state);
                CREATE TABLE local_command_outbox (
                    event_id INTEGER PRIMARY KEY AUTOINCREMENT,
                    command_id TEXT NOT NULL REFERENCES local_commands(command_id),
                    aggregate_id TEXT NOT NULL,
                    schema_version INTEGER NOT NULL CHECK(schema_version BETWEEN 1 AND 255),
                    event_type TEXT NOT NULL,
                    occurred_at_ms INTEGER NOT NULL CHECK(occurred_at_ms >= 0),
                    delivered_at_ms INTEGER CHECK(delivered_at_ms IS NULL OR delivered_at_ms >= 0),
                    UNIQUE(command_id, event_type)
                );
                INSERT INTO local_commands (
                    command_id, payload_digest, schema_version, tenant_id, task_id,
                    attempt_id, expected_revision, actor_id, device_id, deadline_unix_ms,
                    payload_json, state, rejection, effect_digest, state_updated_at_ms
                ) SELECT command_id, payload_digest, schema_version, tenant_id, task_id,
                    attempt_id, expected_revision, actor_id, device_id, deadline_unix_ms,
                    payload_json,
                    CASE state WHEN 'coordinator_stored' THEN 'pending_host_admission' ELSE state END,
                    rejection, effect_digest, state_updated_at_ms
                FROM local_commands_v1;
                INSERT INTO local_command_outbox (
                    event_id, command_id, aggregate_id, schema_version, event_type,
                    occurred_at_ms, delivered_at_ms
                ) SELECT event_id, command_id, aggregate_id, schema_version, event_type,
                    occurred_at_ms, delivered_at_ms
                FROM local_command_outbox_v1 WHERE event_type <> 'coordinator_stored';
                DROP TABLE local_command_outbox_v1;
                DROP TABLE local_commands_v1;
                PRAGMA user_version = 2;",
            )?;
            transaction.commit()?;
            Ok(())
        }
        DATABASE_VERSION => Ok(()),
        other => Err(JournalError::UnsupportedDatabaseVersion(other)),
    }
}

fn validate_command(command: &CommandEnvelope) -> Result<(), JournalError> {
    if command.schema_version == 0 {
        return Err(JournalError::InvalidInput("unsupported schema version"));
    }
    for (name, value) in [
        ("command ID", command.command_id.as_str()),
        ("tenant ID", command.tenant_id.as_str()),
        ("task ID", command.task_id.as_str()),
        ("attempt ID", command.attempt_id.as_str()),
        ("actor ID", command.actor_id.as_str()),
        ("device ID", command.device_id.as_str()),
    ] {
        if value.is_empty() || value.contains('\0') || value.len() > MAX_IDENTIFIER_BYTES {
            return Err(JournalError::InvalidInput(name));
        }
    }
    if command.expected_revision < 0 {
        return Err(JournalError::InvalidInput("negative expected revision"));
    }
    if command.deadline_unix_ms < 0 {
        return Err(JournalError::InvalidInput("negative deadline"));
    }
    if command.payload_json.len() > MAX_COMMAND_PAYLOAD_BYTES {
        return Err(JournalError::InvalidInput(
            "command payload exceeds the journal limit",
        ));
    }
    serde_json::from_slice::<serde_json::Value>(&command.payload_json)
        .map_err(|_| JournalError::InvalidInput("command payload is not valid JSON"))?;
    Ok(())
}

fn command_digest(command: &CommandEnvelope) -> [u8; 32] {
    let mut digest = Sha256::new();
    digest.update(b"conductor-local-command-v1\0");
    hash_field(&mut digest, &[command.schema_version]);
    hash_field(&mut digest, command.command_id.as_bytes());
    hash_field(&mut digest, command.tenant_id.as_bytes());
    hash_field(&mut digest, command.task_id.as_bytes());
    hash_field(&mut digest, command.attempt_id.as_bytes());
    hash_field(&mut digest, &command.expected_revision.to_be_bytes());
    hash_field(&mut digest, command.actor_id.as_bytes());
    hash_field(&mut digest, command.device_id.as_bytes());
    hash_field(&mut digest, &command.deadline_unix_ms.to_be_bytes());
    hash_field(&mut digest, &command.payload_json);
    digest.finalize().into()
}

fn hash_field(digest: &mut Sha256, value: &[u8]) {
    digest.update((value.len() as u64).to_be_bytes());
    digest.update(value);
}

fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut result = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        result.push(char::from(DIGITS[usize::from(byte >> 4)]));
        result.push(char::from(DIGITS[usize::from(byte & 0x0f)]));
    }
    result
}

fn read_receipt(
    connection: &Connection,
    command_id: &str,
) -> Result<Option<CommandReceipt>, JournalError> {
    connection
        .query_row(
            "SELECT schema_version, command_id, tenant_id, task_id, attempt_id,
                    expected_revision, actor_id, device_id, deadline_unix_ms,
                    payload_json, payload_digest, state, rejection, effect_digest,
                    state_updated_at_ms
             FROM local_commands WHERE command_id = ?1",
            [command_id],
            |row| {
                let schema_version: i64 = row.get(0)?;
                let schema_version = u8::try_from(schema_version)
                    .map_err(|_| rusqlite::Error::IntegralValueOutOfRange(0, schema_version))?;
                let digest: Vec<u8> = row.get(10)?;
                let effect_digest: Option<Vec<u8>> = row.get(13)?;
                let state: String = row.get(11)?;
                let rejection: Option<String> = row.get(12)?;
                Ok((
                    CommandEnvelope {
                        schema_version,
                        command_id: row.get(1)?,
                        tenant_id: row.get(2)?,
                        task_id: row.get(3)?,
                        attempt_id: row.get(4)?,
                        expected_revision: row.get(5)?,
                        actor_id: row.get(6)?,
                        device_id: row.get(7)?,
                        deadline_unix_ms: row.get(8)?,
                        payload_json: row.get(9)?,
                    },
                    digest,
                    state,
                    rejection,
                    effect_digest,
                    row.get(14)?,
                ))
            },
        )
        .optional()?
        .map(
            |(command, digest, state, rejection, effect_digest, state_updated_at_ms)| {
                Ok(CommandReceipt {
                    command,
                    payload_digest: hex(&digest),
                    state: CommandState::parse(&state)?,
                    rejection: rejection.as_deref().map(HostRejection::parse).transpose()?,
                    effect_digest: effect_digest.as_deref().map(hex),
                    state_updated_at_ms,
                })
            },
        )
        .transpose()
}

fn insert_outbox_event(
    transaction: &Transaction<'_>,
    command: &CommandEnvelope,
    event_type: &str,
    occurred_at_ms: i64,
) -> Result<(), JournalError> {
    transaction.execute(
        "INSERT INTO local_command_outbox (
            command_id, aggregate_id, schema_version, event_type, occurred_at_ms
         ) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            command.command_id,
            command.task_id,
            i64::from(command.schema_version),
            event_type,
            occurred_at_ms,
        ],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
    };

    use super::*;

    static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(0);

    struct TempDatabase(PathBuf);

    impl TempDatabase {
        fn new() -> Self {
            let id = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);
            let directory = std::env::temp_dir().join(format!(
                "conductor-host-journal-{}-{id}",
                std::process::id()
            ));
            fs::create_dir_all(&directory).expect("create test database directory");
            Self(directory.join("journal.sqlite3"))
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TempDatabase {
        fn drop(&mut self) {
            if let Some(parent) = self.0.parent() {
                let _ = fs::remove_dir_all(parent);
            }
        }
    }

    fn command(id: &str, expected_revision: i64, deadline: i64) -> CommandEnvelope {
        CommandEnvelope {
            schema_version: 1,
            command_id: id.into(),
            tenant_id: "local-tenant".into(),
            task_id: "task-1".into(),
            attempt_id: "attempt-1".into(),
            expected_revision,
            actor_id: "local-user".into(),
            device_id: "device-1".into(),
            deadline_unix_ms: deadline,
            payload_json: br#"{"operation":"run"}"#.to_vec(),
        }
    }

    #[test]
    fn enables_wal_full_synchronous_and_foreign_keys() {
        let database = TempDatabase::new();
        let journal = CommandJournal::open(database.path()).expect("open journal");
        let connection = journal.connection.lock().expect("journal lock");
        let mode: String = connection
            .query_row("PRAGMA journal_mode", [], |row| row.get(0))
            .expect("read journal mode");
        let synchronous: i64 = connection
            .query_row("PRAGMA synchronous", [], |row| row.get(0))
            .expect("read synchronous mode");
        let foreign_keys: i64 = connection
            .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
            .expect("read foreign key setting");
        assert_eq!(mode.to_lowercase(), "wal");
        assert_eq!(synchronous, 2);
        assert_eq!(foreign_keys, 1);
    }

    #[test]
    fn migration_removes_host_emitted_coordinator_acknowledgements() {
        let database = TempDatabase::new();
        let old_command = command("legacy-command", 4, 1_000);
        let digest = command_digest(&old_command);
        let legacy = Connection::open(database.path()).expect("create v1 journal");
        legacy
            .execute_batch(
                "CREATE TABLE local_commands (
                    command_id TEXT PRIMARY KEY NOT NULL,
                    payload_digest BLOB NOT NULL CHECK(length(payload_digest) = 32),
                    schema_version INTEGER NOT NULL CHECK(schema_version BETWEEN 1 AND 255),
                    tenant_id TEXT NOT NULL,
                    task_id TEXT NOT NULL,
                    attempt_id TEXT NOT NULL,
                    expected_revision INTEGER NOT NULL CHECK(expected_revision >= 0),
                    actor_id TEXT NOT NULL,
                    device_id TEXT NOT NULL,
                    deadline_unix_ms INTEGER NOT NULL CHECK(deadline_unix_ms >= 0),
                    payload_json BLOB NOT NULL,
                    state TEXT NOT NULL CHECK(state IN (
                        'coordinator_stored', 'host_accepted', 'host_rejected', 'effect_resolved'
                    )),
                    rejection TEXT CHECK(rejection IN ('expired', 'stale_revision') OR rejection IS NULL),
                    effect_digest BLOB CHECK(effect_digest IS NULL OR length(effect_digest) = 32),
                    state_updated_at_ms INTEGER NOT NULL CHECK(state_updated_at_ms >= 0)
                );
                CREATE INDEX local_commands_by_task ON local_commands(task_id, state);
                CREATE TABLE local_command_outbox (
                    event_id INTEGER PRIMARY KEY AUTOINCREMENT,
                    command_id TEXT NOT NULL REFERENCES local_commands(command_id),
                    aggregate_id TEXT NOT NULL,
                    schema_version INTEGER NOT NULL CHECK(schema_version BETWEEN 1 AND 255),
                    event_type TEXT NOT NULL,
                    occurred_at_ms INTEGER NOT NULL CHECK(occurred_at_ms >= 0),
                    delivered_at_ms INTEGER CHECK(delivered_at_ms IS NULL OR delivered_at_ms >= 0),
                    UNIQUE(command_id, event_type)
                );
                PRAGMA user_version = 1;",
            )
            .expect("create v1 schema");
        legacy
            .execute(
                "INSERT INTO local_commands (
                    command_id, payload_digest, schema_version, tenant_id, task_id,
                    attempt_id, expected_revision, actor_id, device_id, deadline_unix_ms,
                    payload_json, state, rejection, effect_digest, state_updated_at_ms
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11,
                    'coordinator_stored', NULL, NULL, ?12)",
                rusqlite::params![
                    old_command.command_id,
                    digest.as_slice(),
                    i64::from(old_command.schema_version),
                    old_command.tenant_id,
                    old_command.task_id,
                    old_command.attempt_id,
                    old_command.expected_revision,
                    old_command.actor_id,
                    old_command.device_id,
                    old_command.deadline_unix_ms,
                    old_command.payload_json,
                    500_i64,
                ],
            )
            .expect("insert old command");
        legacy
            .execute(
                "INSERT INTO local_command_outbox (
                    command_id, aggregate_id, schema_version, event_type, occurred_at_ms
                 ) VALUES (?1, ?2, ?3, 'coordinator_stored', ?4)",
                rusqlite::params![old_command.command_id, old_command.task_id, 1_i64, 500_i64],
            )
            .expect("insert false coordinator receipt");
        drop(legacy);

        let journal = CommandJournal::open(database.path()).expect("migrate v1 journal");
        let migrated = journal
            .query("legacy-command")
            .expect("query migrated row")
            .expect("migrated command");
        assert_eq!(migrated.state, CommandState::PendingHostAdmission);
        assert!(
            journal
                .pending_outbox_events(32)
                .expect("outbox")
                .is_empty()
        );
    }

    #[test]
    fn persists_command_and_returns_prior_receipt_for_identical_replay() {
        let database = TempDatabase::new();
        let first = CommandJournal::open(database.path()).expect("open journal");
        let accepted = command("command-1", 4, 1_000);
        let initial = first
            .store_host_command(&accepted, 500)
            .expect("store command");
        assert_eq!(initial.state, CommandState::PendingHostAdmission);
        assert!(first.pending_outbox_events(32).expect("outbox").is_empty());
        drop(first);

        let reopened = CommandJournal::open(database.path()).expect("reopen journal");
        let replay = reopened
            .store_host_command(&accepted, 1_500)
            .expect("replay stored command after deadline");
        assert_eq!(replay, initial);
        assert!(
            reopened
                .pending_outbox_events(32)
                .expect("outbox")
                .is_empty()
        );
    }

    #[test]
    fn rejects_reused_id_when_any_immutable_field_changes() {
        let database = TempDatabase::new();
        let journal = CommandJournal::open(database.path()).expect("open journal");
        let original = command("command-1", 4, 1_000);
        journal
            .store_host_command(&original, 500)
            .expect("store command");
        let mut changed = original;
        changed.expected_revision = 5;
        assert!(matches!(
            journal.store_host_command(&changed, 500),
            Err(JournalError::PayloadMismatch)
        ));
    }

    #[test]
    fn expired_command_is_stored_before_durable_host_rejection() {
        let database = TempDatabase::new();
        let journal = CommandJournal::open(database.path()).expect("open journal");
        let pending = journal
            .store_host_command(&command("late", 0, 50), 50)
            .expect("persist received command");
        assert_eq!(pending.state, CommandState::PendingHostAdmission);
        assert!(
            journal
                .pending_outbox_events(32)
                .expect("outbox")
                .is_empty()
        );

        let rejected = journal
            .record_host_admission("late", 0, 50)
            .expect("persist expiry rejection");
        assert_eq!(rejected.state, CommandState::HostRejected);
        assert_eq!(rejected.rejection, Some(HostRejection::Expired));
        assert_eq!(
            journal.pending_outbox_events(32).expect("outbox")[0].event_type,
            "host_rejected"
        );
    }

    #[test]
    fn stale_or_expired_host_admission_is_a_durable_rejection() {
        let database = TempDatabase::new();
        let journal = CommandJournal::open(database.path()).expect("open journal");
        let stale = command("stale", 4, 1_000);
        journal
            .store_host_command(&stale, 500)
            .expect("store command");
        let rejected = journal
            .record_host_admission("stale", 5, 600)
            .expect("record stale rejection");
        assert_eq!(rejected.state, CommandState::HostRejected);
        assert_eq!(rejected.rejection, Some(HostRejection::StaleRevision));

        let expired = command("expired", 4, 700);
        journal
            .store_host_command(&expired, 600)
            .expect("store command before deadline");
        let rejected = journal
            .record_host_admission("expired", 4, 700)
            .expect("record expired rejection");
        assert_eq!(rejected.state, CommandState::HostRejected);
        assert_eq!(rejected.rejection, Some(HostRejection::Expired));
    }

    #[test]
    fn admission_and_effect_resolution_survive_reopen_and_are_idempotent() {
        let database = TempDatabase::new();
        let stored_journal = CommandJournal::open(database.path()).expect("open journal");
        stored_journal
            .store_host_command(&command("command-1", 4, 1_000), 500)
            .expect("store command");
        drop(stored_journal);

        let accepted_journal =
            CommandJournal::open(database.path()).expect("reopen stored command");
        let stored = accepted_journal
            .query("command-1")
            .expect("query")
            .expect("stored receipt");
        assert_eq!(stored.state, CommandState::PendingHostAdmission);
        let admitted = accepted_journal
            .record_host_admission("command-1", 4, 600)
            .expect("accept at current revision");
        assert_eq!(admitted.state, CommandState::HostAccepted);
        drop(accepted_journal);

        let resolved_journal =
            CommandJournal::open(database.path()).expect("reopen accepted command");
        let accepted = resolved_journal
            .query("command-1")
            .expect("query")
            .expect("accepted receipt");
        assert_eq!(accepted.state, CommandState::HostAccepted);
        let resolved = resolved_journal
            .record_effect_resolved("command-1", b"executor-receipt", 700)
            .expect("resolve effect");
        assert_eq!(resolved.state, CommandState::EffectResolved);
        drop(resolved_journal);

        let final_journal = CommandJournal::open(database.path()).expect("reopen resolved command");
        let replay = final_journal
            .record_effect_resolved("command-1", b"executor-receipt", 800)
            .expect("replay resolution");
        assert_eq!(replay, resolved);
        assert_eq!(
            final_journal
                .pending_outbox_events(32)
                .expect("outbox")
                .len(),
            2
        );
    }

    #[test]
    fn concurrent_duplicate_command_delivery_creates_one_durable_inbox_row() {
        use std::sync::{Arc, Barrier};

        let database = TempDatabase::new();
        let first = Arc::new(CommandJournal::open(database.path()).expect("open first journal"));
        let second = Arc::new(CommandJournal::open(database.path()).expect("open second journal"));
        let barrier = Arc::new(Barrier::new(2));
        let payload = command("racing-command", 4, 1_000);
        let first_thread = {
            let journal = Arc::clone(&first);
            let barrier = Arc::clone(&barrier);
            let payload = payload.clone();
            std::thread::spawn(move || {
                barrier.wait();
                journal.store_host_command(&payload, 500)
            })
        };
        let second_thread = {
            let journal = Arc::clone(&second);
            let barrier = Arc::clone(&barrier);
            std::thread::spawn(move || {
                barrier.wait();
                journal.store_host_command(&payload, 500)
            })
        };
        let first_receipt = first_thread
            .join()
            .expect("first worker")
            .expect("first receipt");
        let second_receipt = second_thread
            .join()
            .expect("second worker")
            .expect("second receipt");
        assert_eq!(first_receipt, second_receipt);
        assert!(first.pending_outbox_events(32).expect("outbox").is_empty());
    }

    #[test]
    fn outbox_failure_rolls_back_host_admission_and_produces_no_receipt() {
        let database = TempDatabase::new();
        let journal = CommandJournal::open(database.path()).expect("open journal");
        journal
            .store_host_command(&command("command-1", 4, 1_000), 500)
            .expect("store command");
        journal
            .connection
            .lock()
            .expect("journal lock")
            .execute_batch(
                "CREATE TRIGGER reject_command_outbox
                 BEFORE INSERT ON local_command_outbox
                 BEGIN SELECT RAISE(ABORT, 'injected outbox failure'); END;",
            )
            .expect("install failure injection trigger");
        assert!(matches!(
            journal.record_host_admission("command-1", 4, 600),
            Err(JournalError::Storage(_))
        ));
        assert_eq!(
            journal
                .query("command-1")
                .expect("query")
                .expect("receipt")
                .state,
            CommandState::PendingHostAdmission
        );
    }

    #[test]
    fn effect_resolution_event_failure_keeps_host_acceptance_unresolved() {
        let database = TempDatabase::new();
        let journal = CommandJournal::open(database.path()).expect("open journal");
        journal
            .store_host_command(&command("command-1", 4, 1_000), 500)
            .expect("store command");
        journal
            .record_host_admission("command-1", 4, 600)
            .expect("accept command");
        journal
            .connection
            .lock()
            .expect("journal lock")
            .execute_batch(
                "CREATE TRIGGER reject_effect_resolution
                 BEFORE INSERT ON local_command_outbox
                 WHEN NEW.event_type = 'effect_resolved'
                 BEGIN SELECT RAISE(ABORT, 'injected resolution failure'); END;",
            )
            .expect("install failure injection trigger");
        assert!(matches!(
            journal.record_effect_resolved("command-1", b"executor-receipt", 700),
            Err(JournalError::Storage(_))
        ));
        let receipt = journal.query("command-1").expect("query").expect("receipt");
        assert_eq!(receipt.state, CommandState::HostAccepted);
        assert_eq!(journal.pending_outbox_events(32).expect("outbox").len(), 1);
    }

    #[test]
    fn outbox_reads_are_bounded() {
        let database = TempDatabase::new();
        let journal = CommandJournal::open(database.path()).expect("open journal");
        assert!(matches!(
            journal.pending_outbox_events(0),
            Err(JournalError::InvalidInput(_))
        ));
        assert!(matches!(
            journal.pending_outbox_events(MAX_OUTBOX_BATCH_SIZE + 1),
            Err(JournalError::InvalidInput(_))
        ));
    }

    #[test]
    fn outbox_delivery_mark_is_idempotent_and_keeps_unknown_ids_unacknowledged() {
        let database = TempDatabase::new();
        let journal = CommandJournal::open(database.path()).expect("open journal");
        journal
            .store_host_command(&command("command-1", 4, 1_000), 500)
            .expect("store command");
        journal
            .record_host_admission("command-1", 4, 550)
            .expect("accept command");
        let event = journal.pending_outbox_events(32).expect("outbox").remove(0);
        journal
            .mark_outbox_delivered(event.event_id, 600)
            .expect("mark delivered");
        journal
            .mark_outbox_delivered(event.event_id, 700)
            .expect("mark delivered replay");
        assert!(
            journal
                .pending_outbox_events(32)
                .expect("pending")
                .is_empty()
        );
        assert!(journal.mark_outbox_delivered(i64::MAX, 700).is_err());
    }
}
