use std::{error::Error, fmt, path::Path, sync::Mutex, time::Duration};

use rusqlite::{
    Connection, OpenFlags, OptionalExtension, Transaction, TransactionBehavior, params,
};
use sha2::{Digest, Sha256};

const DATABASE_VERSION: i64 = 1;
const MAX_IDENTIFIER_BYTES: usize = 256;
const MAX_LAUNCH_SPEC_BYTES: usize = 1_048_576;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SessionState {
    Starting,
    Running,
    CancelRequested,
    Exited,
    Failed,
    Cancelled,
    Interrupted,
    OutcomeUnknown,
}

impl SessionState {
    fn parse(value: &str) -> Result<Self, SessionError> {
        match value {
            "starting" => Ok(Self::Starting),
            "running" => Ok(Self::Running),
            "cancel_requested" => Ok(Self::CancelRequested),
            "exited" => Ok(Self::Exited),
            "failed" => Ok(Self::Failed),
            "cancelled" => Ok(Self::Cancelled),
            "interrupted" => Ok(Self::Interrupted),
            "outcome_unknown" => Ok(Self::OutcomeUnknown),
            _ => Err(SessionError::CorruptState(value.to_owned())),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SessionReceipt {
    pub session_id: String,
    pub launch_digest: String,
    pub controller_identity: String,
    pub supervisor_epoch: String,
    pub state: SessionState,
    /// Diagnostic only. A PID never proves process identity or ownership.
    pub process_id: Option<u32>,
    pub exit_code: Option<i32>,
    pub failure_code: Option<String>,
    pub cancel_requested_at_ms: Option<i64>,
    pub cancel_confirmed_at_ms: Option<i64>,
    pub state_updated_at_ms: i64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StartReceipt {
    pub receipt: SessionReceipt,
    /// True only for the transaction that first persisted this session ID.
    /// Retrying a stored or ambiguous start never grants a second launch.
    pub launch_is_new: bool,
}

#[derive(Debug)]
pub enum SessionError {
    Storage(rusqlite::Error),
    InvalidInput(&'static str),
    SessionNotFound,
    SessionIdReused,
    ControllerIdentityMismatch,
    InvalidTransition {
        state: SessionState,
        requested: &'static str,
    },
    CorruptState(String),
    UnsupportedDatabaseVersion(i64),
    UnsafeDatabaseConfiguration(&'static str),
    LockPoisoned,
}

impl fmt::Display for SessionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Storage(error) => write!(f, "supervisor session storage error: {error}"),
            Self::InvalidInput(message) => write!(f, "invalid supervisor session input: {message}"),
            Self::SessionNotFound => f.write_str("supervisor session not found"),
            Self::SessionIdReused => {
                f.write_str("session ID was already used with a different launch digest")
            }
            Self::ControllerIdentityMismatch => {
                f.write_str("controller identity does not own this session")
            }
            Self::InvalidTransition { state, requested } => {
                write!(f, "cannot {requested} a session in state {state:?}")
            }
            Self::CorruptState(state) => write!(f, "unknown persisted session state: {state}"),
            Self::UnsupportedDatabaseVersion(version) => {
                write!(
                    f,
                    "unsupported supervisor session schema version: {version}"
                )
            }
            Self::UnsafeDatabaseConfiguration(setting) => {
                write!(f, "SQLite did not enable required setting {setting}")
            }
            Self::LockPoisoned => f.write_str("supervisor session store lock was poisoned"),
        }
    }
}

impl Error for SessionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Storage(error) => Some(error),
            _ => None,
        }
    }
}

impl From<rusqlite::Error> for SessionError {
    fn from(error: rusqlite::Error) -> Self {
        Self::Storage(error)
    }
}

/// The supervisor owns this database and its schema migration lifecycle.
///
/// `open` creates a new supervisor epoch. Active receipts from another epoch
/// become `outcome_unknown` in one transaction and are never relaunched. The
/// controller identity must already have been authenticated by the caller;
/// this storage layer is not an IPC or authorization boundary.
pub struct SessionStore {
    connection: Mutex<Connection>,
    supervisor_epoch: String,
}

impl SessionStore {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, SessionError> {
        let flags = OpenFlags::SQLITE_OPEN_READ_WRITE
            | OpenFlags::SQLITE_OPEN_CREATE
            | OpenFlags::SQLITE_OPEN_FULL_MUTEX;
        let connection = Connection::open_with_flags(path, flags)?;
        configure_connection(&connection)?;
        migrate(&connection)?;
        let supervisor_epoch: String =
            connection.query_row("SELECT lower(hex(randomblob(16)))", [], |row| row.get(0))?;
        let now_unix_ms = current_time_unix_ms()?;
        Self::from_connection(connection, supervisor_epoch, now_unix_ms)
    }

    fn from_connection(
        mut connection: Connection,
        supervisor_epoch: String,
        now_unix_ms: i64,
    ) -> Result<Self, SessionError> {
        if supervisor_epoch.is_empty() || now_unix_ms < 0 {
            return Err(SessionError::InvalidInput("invalid supervisor epoch"));
        }
        configure_connection(&connection)?;
        migrate(&connection)?;
        reconcile_previous_epoch(&mut connection, &supervisor_epoch, now_unix_ms)?;
        Ok(Self {
            connection: Mutex::new(connection),
            supervisor_epoch,
        })
    }

    /// Persist Start intent before a process is spawned. The first transaction
    /// returns `launch_is_new = true`; duplicate or uncertain retries return
    /// the saved receipt with `false` and must not launch again.
    pub fn start(
        &self,
        session_id: &str,
        launch_spec_json: &[u8],
        controller_identity: &str,
        now_unix_ms: i64,
    ) -> Result<StartReceipt, SessionError> {
        validate_identifier("session ID", session_id)?;
        validate_identifier("controller identity", controller_identity)?;
        if now_unix_ms < 0 {
            return Err(SessionError::InvalidInput("negative current time"));
        }
        if launch_spec_json.len() > MAX_LAUNCH_SPEC_BYTES {
            return Err(SessionError::InvalidInput(
                "launch specification exceeds the store limit",
            ));
        }
        serde_json::from_slice::<serde_json::Value>(launch_spec_json)
            .map_err(|_| SessionError::InvalidInput("launch specification is not valid JSON"))?;
        let launch_digest: [u8; 32] = Sha256::digest(launch_spec_json).into();
        let mut connection = self
            .connection
            .lock()
            .map_err(|_| SessionError::LockPoisoned)?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;

        if let Some(existing) = read_receipt(&transaction, session_id)? {
            if existing.launch_digest != hex(&launch_digest) {
                return Err(SessionError::SessionIdReused);
            }
            if existing.controller_identity != controller_identity {
                return Err(SessionError::ControllerIdentityMismatch);
            }
            transaction.commit()?;
            return Ok(StartReceipt {
                receipt: existing,
                launch_is_new: false,
            });
        }

        transaction.execute(
            "INSERT INTO supervisor_sessions (
                session_id, launch_digest, controller_identity, supervisor_epoch,
                state, process_id, exit_code, failure_code, cancel_requested_at_ms,
                cancel_confirmed_at_ms, state_updated_at_ms
             ) VALUES (?1, ?2, ?3, ?4, 'starting', NULL, NULL, NULL, NULL, NULL, ?5)",
            params![
                session_id,
                launch_digest.as_slice(),
                controller_identity,
                self.supervisor_epoch,
                now_unix_ms,
            ],
        )?;
        insert_event(
            &transaction,
            session_id,
            &self.supervisor_epoch,
            "start_intent_persisted",
            "starting",
            now_unix_ms,
        )?;
        let receipt =
            read_receipt(&transaction, session_id)?.ok_or(SessionError::SessionNotFound)?;
        transaction.commit()?;
        Ok(StartReceipt {
            receipt,
            launch_is_new: true,
        })
    }

    /// Record a successful OS spawn while the supervisor still owns the real
    /// process handle. This receipt does not make the persisted PID proof of
    /// ownership after a supervisor restart.
    // Reserved for the future process owner after it has retained the OS handle.
    #[allow(dead_code)]
    pub(crate) fn record_running(
        &self,
        session_id: &str,
        controller_identity: &str,
        process_id: u32,
        now_unix_ms: i64,
    ) -> Result<SessionReceipt, SessionError> {
        if process_id == 0 || now_unix_ms < 0 {
            return Err(SessionError::InvalidInput("invalid process receipt"));
        }
        let mut connection = self
            .connection
            .lock()
            .map_err(|_| SessionError::LockPoisoned)?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let existing = owned_receipt(&transaction, session_id, controller_identity)?;
        require_state(
            existing.state,
            SessionState::Starting,
            "record process start",
        )?;
        let changed = transaction.execute(
            "UPDATE supervisor_sessions SET state = 'running', process_id = ?2,
                    state_updated_at_ms = ?3
             WHERE session_id = ?1 AND supervisor_epoch = ?4 AND state = 'starting'",
            params![
                session_id,
                i64::from(process_id),
                now_unix_ms,
                self.supervisor_epoch,
            ],
        )?;
        if changed != 1 {
            return Err(SessionError::InvalidTransition {
                state: existing.state,
                requested: "record process start",
            });
        }
        insert_event(
            &transaction,
            session_id,
            &self.supervisor_epoch,
            "process_started",
            "running",
            now_unix_ms,
        )?;
        let receipt =
            read_receipt(&transaction, session_id)?.ok_or(SessionError::SessionNotFound)?;
        transaction.commit()?;
        Ok(receipt)
    }

    // Reserved for the future process owner after a real spawn failure.
    #[allow(dead_code)]
    pub(crate) fn record_start_failure(
        &self,
        session_id: &str,
        controller_identity: &str,
        failure_code: &str,
        now_unix_ms: i64,
    ) -> Result<SessionReceipt, SessionError> {
        validate_identifier("failure code", failure_code)?;
        if now_unix_ms < 0 {
            return Err(SessionError::InvalidInput("negative current time"));
        }
        let mut connection = self
            .connection
            .lock()
            .map_err(|_| SessionError::LockPoisoned)?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let existing = owned_receipt(&transaction, session_id, controller_identity)?;
        require_state(
            existing.state,
            SessionState::Starting,
            "record start failure",
        )?;
        transaction.execute(
            "UPDATE supervisor_sessions SET state = 'failed', failure_code = ?2,
                    state_updated_at_ms = ?3
             WHERE session_id = ?1 AND supervisor_epoch = ?4 AND state = 'starting'",
            params![session_id, failure_code, now_unix_ms, self.supervisor_epoch],
        )?;
        insert_event(
            &transaction,
            session_id,
            &self.supervisor_epoch,
            "start_failed",
            "failed",
            now_unix_ms,
        )?;
        let receipt =
            read_receipt(&transaction, session_id)?.ok_or(SessionError::SessionNotFound)?;
        transaction.commit()?;
        Ok(receipt)
    }

    /// Persist a child exit observed through the live supervisor-owned handle.
    // Reserved for the future process owner after observing the OS child handle.
    #[allow(dead_code)]
    pub(crate) fn record_exit(
        &self,
        session_id: &str,
        controller_identity: &str,
        exit_code: i32,
        now_unix_ms: i64,
    ) -> Result<SessionReceipt, SessionError> {
        if now_unix_ms < 0 {
            return Err(SessionError::InvalidInput("negative current time"));
        }
        let mut connection = self
            .connection
            .lock()
            .map_err(|_| SessionError::LockPoisoned)?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let existing = owned_receipt(&transaction, session_id, controller_identity)?;
        if !matches!(
            existing.state,
            SessionState::Starting | SessionState::Running | SessionState::CancelRequested
        ) {
            return Err(SessionError::InvalidTransition {
                state: existing.state,
                requested: "record process exit",
            });
        }
        transaction.execute(
            "UPDATE supervisor_sessions SET state = 'exited', exit_code = ?2,
                    state_updated_at_ms = ?3
             WHERE session_id = ?1 AND supervisor_epoch = ?4
               AND state IN ('starting', 'running', 'cancel_requested')",
            params![session_id, exit_code, now_unix_ms, self.supervisor_epoch,],
        )?;
        insert_event(
            &transaction,
            session_id,
            &self.supervisor_epoch,
            "process_exited",
            "exited",
            now_unix_ms,
        )?;
        let receipt =
            read_receipt(&transaction, session_id)?.ok_or(SessionError::SessionNotFound)?;
        transaction.commit()?;
        Ok(receipt)
    }

    /// Persist cancellation intent. The process owner must kill/wait for the
    /// complete owned process tree before calling `confirm_cancel`.
    pub fn request_cancel(
        &self,
        session_id: &str,
        controller_identity: &str,
        now_unix_ms: i64,
    ) -> Result<SessionReceipt, SessionError> {
        if now_unix_ms < 0 {
            return Err(SessionError::InvalidInput("negative current time"));
        }
        let mut connection = self
            .connection
            .lock()
            .map_err(|_| SessionError::LockPoisoned)?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let existing = owned_receipt(&transaction, session_id, controller_identity)?;
        if existing.state == SessionState::CancelRequested {
            transaction.commit()?;
            return Ok(existing);
        }
        if !matches!(
            existing.state,
            SessionState::Starting | SessionState::Running
        ) {
            return Err(SessionError::InvalidTransition {
                state: existing.state,
                requested: "request cancellation for",
            });
        }
        transaction.execute(
            "UPDATE supervisor_sessions SET state = 'cancel_requested',
                    cancel_requested_at_ms = COALESCE(cancel_requested_at_ms, ?2),
                    state_updated_at_ms = ?2
             WHERE session_id = ?1 AND supervisor_epoch = ?3
               AND state IN ('starting', 'running')",
            params![session_id, now_unix_ms, self.supervisor_epoch],
        )?;
        insert_event(
            &transaction,
            session_id,
            &self.supervisor_epoch,
            "cancel_requested",
            "cancel_requested",
            now_unix_ms,
        )?;
        let receipt =
            read_receipt(&transaction, session_id)?.ok_or(SessionError::SessionNotFound)?;
        transaction.commit()?;
        Ok(receipt)
    }

    /// Confirm cancellation only after the process owner proves the whole
    /// owned process tree is no longer running.
    // Reserved for the future process owner after proving the owned tree stopped.
    #[allow(dead_code)]
    pub(crate) fn confirm_cancel(
        &self,
        session_id: &str,
        controller_identity: &str,
        now_unix_ms: i64,
    ) -> Result<SessionReceipt, SessionError> {
        if now_unix_ms < 0 {
            return Err(SessionError::InvalidInput("negative current time"));
        }
        let mut connection = self
            .connection
            .lock()
            .map_err(|_| SessionError::LockPoisoned)?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let existing = owned_receipt(&transaction, session_id, controller_identity)?;
        if existing.state == SessionState::Cancelled {
            transaction.commit()?;
            return Ok(existing);
        }
        require_state(
            existing.state,
            SessionState::CancelRequested,
            "confirm cancellation for",
        )?;
        transaction.execute(
            "UPDATE supervisor_sessions SET state = 'cancelled',
                    cancel_confirmed_at_ms = ?2, state_updated_at_ms = ?2
             WHERE session_id = ?1 AND supervisor_epoch = ?3
               AND state = 'cancel_requested'",
            params![session_id, now_unix_ms, self.supervisor_epoch],
        )?;
        insert_event(
            &transaction,
            session_id,
            &self.supervisor_epoch,
            "cancel_confirmed",
            "cancelled",
            now_unix_ms,
        )?;
        let receipt =
            read_receipt(&transaction, session_id)?.ok_or(SessionError::SessionNotFound)?;
        transaction.commit()?;
        Ok(receipt)
    }

    pub fn query(
        &self,
        session_id: &str,
        controller_identity: &str,
    ) -> Result<Option<SessionReceipt>, SessionError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| SessionError::LockPoisoned)?;
        let receipt = read_receipt(&connection, session_id)?;
        match receipt {
            Some(receipt) if receipt.controller_identity == controller_identity => {
                Ok(Some(receipt))
            }
            Some(_) => Err(SessionError::ControllerIdentityMismatch),
            None => Ok(None),
        }
    }

    pub fn supervisor_epoch(&self) -> &str {
        &self.supervisor_epoch
    }

    #[cfg(test)]
    fn open_with_epoch(
        path: impl AsRef<Path>,
        epoch: &str,
        now_unix_ms: i64,
    ) -> Result<Self, SessionError> {
        let flags = OpenFlags::SQLITE_OPEN_READ_WRITE
            | OpenFlags::SQLITE_OPEN_CREATE
            | OpenFlags::SQLITE_OPEN_FULL_MUTEX;
        let connection = Connection::open_with_flags(path, flags)?;
        Self::from_connection(connection, epoch.to_owned(), now_unix_ms)
    }
}

fn configure_connection(connection: &Connection) -> Result<(), SessionError> {
    connection.busy_timeout(Duration::from_secs(5))?;
    let journal_mode: String =
        connection.query_row("PRAGMA journal_mode = WAL", [], |row| row.get(0))?;
    if !journal_mode.eq_ignore_ascii_case("wal") {
        return Err(SessionError::UnsafeDatabaseConfiguration(
            "journal_mode=WAL",
        ));
    }
    connection.pragma_update(None, "synchronous", "FULL")?;
    let synchronous: i64 = connection.query_row("PRAGMA synchronous", [], |row| row.get(0))?;
    if synchronous != 2 {
        return Err(SessionError::UnsafeDatabaseConfiguration(
            "synchronous=FULL",
        ));
    }
    connection.pragma_update(None, "foreign_keys", "ON")?;
    let foreign_keys: i64 = connection.query_row("PRAGMA foreign_keys", [], |row| row.get(0))?;
    if foreign_keys != 1 {
        return Err(SessionError::UnsafeDatabaseConfiguration("foreign_keys=ON"));
    }
    Ok(())
}

fn migrate(connection: &Connection) -> Result<(), SessionError> {
    let version: i64 = connection.pragma_query_value(None, "user_version", |row| row.get(0))?;
    match version {
        0 => {
            connection.execute_batch(
                "BEGIN IMMEDIATE;
                CREATE TABLE supervisor_sessions (
                    session_id TEXT PRIMARY KEY NOT NULL,
                    launch_digest BLOB NOT NULL CHECK(length(launch_digest) = 32),
                    controller_identity TEXT NOT NULL,
                    supervisor_epoch TEXT NOT NULL,
                    state TEXT NOT NULL CHECK(state IN (
                        'starting', 'running', 'cancel_requested', 'exited', 'failed',
                        'cancelled', 'interrupted', 'outcome_unknown'
                    )),
                    process_id INTEGER CHECK(process_id IS NULL OR process_id > 0),
                    exit_code INTEGER,
                    failure_code TEXT,
                    cancel_requested_at_ms INTEGER CHECK(cancel_requested_at_ms IS NULL OR cancel_requested_at_ms >= 0),
                    cancel_confirmed_at_ms INTEGER CHECK(cancel_confirmed_at_ms IS NULL OR cancel_confirmed_at_ms >= 0),
                    state_updated_at_ms INTEGER NOT NULL CHECK(state_updated_at_ms >= 0)
                );
                CREATE INDEX supervisor_sessions_by_state ON supervisor_sessions(state);
                CREATE TABLE supervisor_session_events (
                    event_id INTEGER PRIMARY KEY AUTOINCREMENT,
                    session_id TEXT NOT NULL REFERENCES supervisor_sessions(session_id),
                    supervisor_epoch TEXT NOT NULL,
                    event_type TEXT NOT NULL,
                    state TEXT NOT NULL,
                    occurred_at_ms INTEGER NOT NULL CHECK(occurred_at_ms >= 0)
                );
                PRAGMA user_version = 1;
                COMMIT;",
            )?;
            Ok(())
        }
        DATABASE_VERSION => Ok(()),
        other => Err(SessionError::UnsupportedDatabaseVersion(other)),
    }
}

fn reconcile_previous_epoch(
    connection: &mut Connection,
    current_epoch: &str,
    now_unix_ms: i64,
) -> Result<(), SessionError> {
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let stale_sessions = {
        let mut statement = transaction.prepare(
            "SELECT session_id FROM supervisor_sessions
             WHERE supervisor_epoch <> ?1
               AND state IN ('starting', 'running', 'cancel_requested')
             ORDER BY session_id",
        )?;
        let rows = statement.query_map([current_epoch], |row| row.get::<_, String>(0))?;
        rows.collect::<Result<Vec<_>, _>>()?
    };
    for session_id in stale_sessions {
        transaction.execute(
            "UPDATE supervisor_sessions SET state = 'outcome_unknown',
                    failure_code = 'supervisor_epoch_changed', state_updated_at_ms = ?2
             WHERE session_id = ?1 AND supervisor_epoch <> ?3
               AND state IN ('starting', 'running', 'cancel_requested')",
            params![session_id, now_unix_ms, current_epoch],
        )?;
        insert_event(
            &transaction,
            &session_id,
            current_epoch,
            "prior_epoch_reconciled_unknown",
            "outcome_unknown",
            now_unix_ms,
        )?;
    }
    transaction.commit()?;
    Ok(())
}

fn validate_identifier(name: &'static str, value: &str) -> Result<(), SessionError> {
    if value.is_empty() || value.contains('\0') || value.len() > MAX_IDENTIFIER_BYTES {
        return Err(SessionError::InvalidInput(name));
    }
    Ok(())
}

fn current_time_unix_ms() -> Result<i64, SessionError> {
    let duration = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|_| SessionError::InvalidInput("system clock precedes Unix epoch"))?;
    i64::try_from(duration.as_millis())
        .map_err(|_| SessionError::InvalidInput("system time exceeds supported range"))
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
    session_id: &str,
) -> Result<Option<SessionReceipt>, SessionError> {
    connection
        .query_row(
            "SELECT session_id, launch_digest, controller_identity, supervisor_epoch,
                    state, process_id, exit_code, failure_code, cancel_requested_at_ms,
                    cancel_confirmed_at_ms, state_updated_at_ms
             FROM supervisor_sessions WHERE session_id = ?1",
            [session_id],
            |row| {
                let digest: Vec<u8> = row.get(1)?;
                let process_id: Option<i64> = row.get(5)?;
                let process_id = process_id
                    .map(u32::try_from)
                    .transpose()
                    .map_err(|_| rusqlite::Error::IntegralValueOutOfRange(5, i64::MAX))?;
                Ok((
                    row.get::<_, String>(0)?,
                    digest,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    process_id,
                    row.get::<_, Option<i32>>(6)?,
                    row.get::<_, Option<String>>(7)?,
                    row.get::<_, Option<i64>>(8)?,
                    row.get::<_, Option<i64>>(9)?,
                    row.get::<_, i64>(10)?,
                ))
            },
        )
        .optional()?
        .map(
            |(
                session_id,
                digest,
                controller_identity,
                supervisor_epoch,
                state,
                process_id,
                exit_code,
                failure_code,
                cancel_requested_at_ms,
                cancel_confirmed_at_ms,
                state_updated_at_ms,
            )| {
                Ok(SessionReceipt {
                    session_id,
                    launch_digest: hex(&digest),
                    controller_identity,
                    supervisor_epoch,
                    state: SessionState::parse(&state)?,
                    process_id,
                    exit_code,
                    failure_code,
                    cancel_requested_at_ms,
                    cancel_confirmed_at_ms,
                    state_updated_at_ms,
                })
            },
        )
        .transpose()
}

fn owned_receipt(
    connection: &Connection,
    session_id: &str,
    controller_identity: &str,
) -> Result<SessionReceipt, SessionError> {
    let receipt = read_receipt(connection, session_id)?.ok_or(SessionError::SessionNotFound)?;
    if receipt.controller_identity != controller_identity {
        return Err(SessionError::ControllerIdentityMismatch);
    }
    Ok(receipt)
}

// Used by process-owner transitions that are intentionally not exposed in S0.
#[allow(dead_code)]
fn require_state(
    state: SessionState,
    expected: SessionState,
    requested: &'static str,
) -> Result<(), SessionError> {
    if state == expected {
        Ok(())
    } else {
        Err(SessionError::InvalidTransition { state, requested })
    }
}

fn insert_event(
    transaction: &Transaction<'_>,
    session_id: &str,
    epoch: &str,
    event_type: &str,
    state: &str,
    occurred_at_ms: i64,
) -> Result<(), SessionError> {
    transaction.execute(
        "INSERT INTO supervisor_session_events (
            session_id, supervisor_epoch, event_type, state, occurred_at_ms
         ) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![session_id, epoch, event_type, state, occurred_at_ms],
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
                "conductor-supervisor-sessions-{}-{id}",
                std::process::id()
            ));
            fs::create_dir_all(&directory).expect("create test database directory");
            Self(directory.join("sessions.sqlite3"))
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

    #[test]
    fn persists_start_intent_and_never_grants_duplicate_launch() {
        let database = TempDatabase::new();
        let store = SessionStore::open_with_epoch(database.path(), "epoch-1", 100)
            .expect("open session store");
        let first = store
            .start(
                "session-1",
                br#"{"program":"tool"}"#,
                "host-controller",
                200,
            )
            .expect("persist start");
        assert!(first.launch_is_new);
        assert_eq!(first.receipt.state, SessionState::Starting);

        let replay = store
            .start(
                "session-1",
                br#"{"program":"tool"}"#,
                "host-controller",
                300,
            )
            .expect("replay start");
        assert!(!replay.launch_is_new);
        assert_eq!(replay.receipt, first.receipt);
        assert!(matches!(
            store.start(
                "session-1",
                br#"{"program":"other"}"#,
                "host-controller",
                400
            ),
            Err(SessionError::SessionIdReused)
        ));
    }

    #[test]
    fn supervisor_restart_reconciles_ambiguous_session_without_relaunch() {
        let database = TempDatabase::new();
        let first = SessionStore::open_with_epoch(database.path(), "epoch-1", 100)
            .expect("open session store");
        let start = first
            .start(
                "session-1",
                br#"{"program":"tool"}"#,
                "host-controller",
                200,
            )
            .expect("persist start");
        first
            .record_running("session-1", "host-controller", 42, 250)
            .expect("record start receipt");
        assert_eq!(start.receipt.state, SessionState::Starting);
        drop(first);

        let restarted = SessionStore::open_with_epoch(database.path(), "epoch-2", 500)
            .expect("reopen after supervisor restart");
        let receipt = restarted
            .query("session-1", "host-controller")
            .expect("query session")
            .expect("stored receipt");
        assert_eq!(receipt.state, SessionState::OutcomeUnknown);
        assert_eq!(receipt.process_id, Some(42));
        assert_eq!(
            receipt.failure_code.as_deref(),
            Some("supervisor_epoch_changed")
        );
        let replay = restarted
            .start(
                "session-1",
                br#"{"program":"tool"}"#,
                "host-controller",
                600,
            )
            .expect("retry uncertain start");
        assert!(!replay.launch_is_new);
        assert_eq!(replay.receipt.state, SessionState::OutcomeUnknown);
    }

    #[test]
    fn cancellation_receipts_survive_reopen_and_remain_distinct() {
        let database = TempDatabase::new();
        let store = SessionStore::open_with_epoch(database.path(), "epoch-1", 100)
            .expect("open session store");
        store
            .start(
                "session-1",
                br#"{"program":"tool"}"#,
                "host-controller",
                200,
            )
            .expect("persist start");
        store
            .record_running("session-1", "host-controller", 42, 250)
            .expect("record start receipt");
        let requested = store
            .request_cancel("session-1", "host-controller", 300)
            .expect("request cancel");
        assert_eq!(requested.state, SessionState::CancelRequested);
        assert_eq!(requested.cancel_requested_at_ms, Some(300));
        assert_eq!(requested.cancel_confirmed_at_ms, None);
        drop(store);

        let reopened = SessionStore::open_with_epoch(database.path(), "epoch-1", 350)
            .expect("reopen during same supervisor epoch");
        let persisted = reopened
            .query("session-1", "host-controller")
            .expect("query")
            .expect("receipt");
        assert_eq!(persisted, requested);
        let confirmed = reopened
            .confirm_cancel("session-1", "host-controller", 400)
            .expect("confirm after process tree is stopped");
        assert_eq!(confirmed.state, SessionState::Cancelled);
        assert_eq!(confirmed.cancel_requested_at_ms, Some(300));
        assert_eq!(confirmed.cancel_confirmed_at_ms, Some(400));
        drop(reopened);

        let final_store = SessionStore::open_with_epoch(database.path(), "epoch-1", 450)
            .expect("reopen cancelled session");
        assert_eq!(
            final_store
                .query("session-1", "host-controller")
                .expect("query")
                .expect("receipt"),
            confirmed
        );
        assert_eq!(
            final_store
                .confirm_cancel("session-1", "host-controller", 500)
                .expect("repeat confirmation"),
            confirmed
        );
    }

    #[test]
    fn observed_exit_after_cancel_request_is_not_reported_as_cancel_confirmed() {
        let database = TempDatabase::new();
        let store = SessionStore::open_with_epoch(database.path(), "epoch-1", 100)
            .expect("open session store");
        store
            .start(
                "session-1",
                br#"{"program":"tool"}"#,
                "host-controller",
                200,
            )
            .expect("persist start");
        store
            .record_running("session-1", "host-controller", 42, 250)
            .expect("record start receipt");
        store
            .request_cancel("session-1", "host-controller", 300)
            .expect("request cancellation");
        let exited = store
            .record_exit("session-1", "host-controller", 143, 350)
            .expect("record observed exit");
        assert_eq!(exited.state, SessionState::Exited);
        assert_eq!(exited.cancel_requested_at_ms, Some(300));
        assert_eq!(exited.cancel_confirmed_at_ms, None);
        assert!(matches!(
            store.confirm_cancel("session-1", "host-controller", 400),
            Err(SessionError::InvalidTransition { .. })
        ));
    }

    #[test]
    fn start_event_failure_rolls_back_intent_and_grants_no_launch_receipt() {
        let database = TempDatabase::new();
        let store = SessionStore::open_with_epoch(database.path(), "epoch-1", 100)
            .expect("open session store");
        store
            .connection
            .lock()
            .expect("session store lock")
            .execute_batch(
                "CREATE TRIGGER reject_session_event
                 BEFORE INSERT ON supervisor_session_events
                 WHEN NEW.event_type = 'start_intent_persisted'
                 BEGIN SELECT RAISE(ABORT, 'injected session event failure'); END;",
            )
            .expect("install failure injection trigger");
        assert!(matches!(
            store.start(
                "session-1",
                br#"{"program":"tool"}"#,
                "host-controller",
                200
            ),
            Err(SessionError::Storage(_))
        ));
        assert!(
            store
                .query("session-1", "host-controller")
                .expect("query")
                .is_none()
        );
    }

    #[test]
    fn controller_identity_is_required_for_query_and_cancel() {
        let database = TempDatabase::new();
        let store = SessionStore::open_with_epoch(database.path(), "epoch-1", 100)
            .expect("open session store");
        store
            .start("session-1", br#"{"program":"tool"}"#, "controller-A", 200)
            .expect("persist start");
        assert!(matches!(
            store.query("session-1", "controller-B"),
            Err(SessionError::ControllerIdentityMismatch)
        ));
        assert!(matches!(
            store.request_cancel("session-1", "controller-B", 300),
            Err(SessionError::ControllerIdentityMismatch)
        ));
    }

    #[test]
    fn unknown_spawn_window_is_not_automatically_retried_after_reopen() {
        let database = TempDatabase::new();
        let first = SessionStore::open_with_epoch(database.path(), "epoch-1", 100)
            .expect("open session store");
        first
            .start(
                "session-1",
                br#"{"program":"tool"}"#,
                "host-controller",
                200,
            )
            .expect("persist start intent");
        drop(first);

        let restarted = SessionStore::open_with_epoch(database.path(), "epoch-2", 500)
            .expect("restart after uncertain spawn boundary");
        let receipt = restarted
            .query("session-1", "host-controller")
            .expect("query")
            .expect("session receipt");
        assert_eq!(receipt.state, SessionState::OutcomeUnknown);
        let replay = restarted
            .start(
                "session-1",
                br#"{"program":"tool"}"#,
                "host-controller",
                600,
            )
            .expect("same command retry");
        assert!(!replay.launch_is_new);
        assert_eq!(replay.receipt.state, SessionState::OutcomeUnknown);
    }
}
