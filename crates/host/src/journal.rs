use std::{error::Error, fmt, path::Path, sync::Mutex};

use conductor_protocol::{
    CommandV1, JsonInputLimits, canonicalize_json_text, compute_command_v1_digest_json,
};
use conductor_sqlite::{ConfigurationError, configure_durable_connection};
use rusqlite::{Connection, OpenFlags, OptionalExtension, TransactionBehavior, params};

const DATABASE_VERSION: i64 = 4;
const MAX_JSON_DEPTH: usize = 128;

#[derive(Debug)]
pub(crate) enum JournalError {
    Storage(rusqlite::Error),
    InvalidInput(&'static str),
    PayloadMismatch,
    LegacyCommandQuarantined,
    UnsupportedDatabaseVersion(i64),
    UnsafeDatabaseConfiguration(&'static str),
    LockPoisoned,
}

impl fmt::Display for JournalError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Storage(error) => write!(formatter, "local command store error: {error}"),
            Self::InvalidInput(message) => write!(formatter, "invalid stored command: {message}"),
            Self::PayloadMismatch => {
                formatter.write_str("command ID was already used with different canonical content")
            }
            Self::LegacyCommandQuarantined => {
                formatter.write_str("command ID belongs to a quarantined legacy record")
            }
            Self::UnsupportedDatabaseVersion(version) => {
                write!(
                    formatter,
                    "unsupported local command store schema: {version}"
                )
            }
            Self::UnsafeDatabaseConfiguration(setting) => {
                write!(
                    formatter,
                    "SQLite did not enable required setting {setting}"
                )
            }
            Self::LockPoisoned => formatter.write_str("local command store lock was poisoned"),
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

impl From<ConfigurationError> for JournalError {
    fn from(error: ConfigurationError) -> Self {
        match error {
            ConfigurationError::Storage(error) => Self::Storage(error),
            ConfigurationError::RequiredSetting(setting) => {
                Self::UnsafeDatabaseConfiguration(setting)
            }
        }
    }
}

/// Internal persistence only. This crate-private store emits no host receipt,
/// admission state, or outbox event and is not wired to any S0 entry point.
pub(crate) struct CommandJournal {
    connection: Mutex<Connection>,
}

impl CommandJournal {
    pub(crate) fn open(path: impl AsRef<Path>) -> Result<Self, JournalError> {
        if path.as_ref().as_os_str().is_empty() || path.as_ref() == Path::new(":memory:") {
            return Err(JournalError::InvalidInput(
                "database path must be file-backed",
            ));
        }
        let flags = OpenFlags::SQLITE_OPEN_READ_WRITE
            | OpenFlags::SQLITE_OPEN_CREATE
            | OpenFlags::SQLITE_OPEN_FULL_MUTEX;
        let mut connection = Connection::open_with_flags(path, flags)?;
        configure_durable_connection(&connection)?;
        migrate(&mut connection)?;
        Ok(Self {
            connection: Mutex::new(connection),
        })
    }

    /// Persist only a protocol-validated command and its normative identity.
    /// Equal replays are a no-op; changed content and quarantined legacy IDs
    /// fail closed. No receipt or acceptance state is returned.
    pub(crate) fn store(&self, command: &CommandV1) -> Result<(), JournalError> {
        let canonical = canonical_command_json(command)?;
        let canonical_bytes = canonical.as_bytes();
        let mut connection = self
            .connection
            .lock()
            .map_err(|_| JournalError::LockPoisoned)?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;

        let is_quarantined: bool = transaction.query_row(
            "SELECT EXISTS(
                SELECT 1 FROM legacy_command_quarantine WHERE command_id = ?1
             )",
            [command.command_id()],
            |row| row.get(0),
        )?;
        if is_quarantined {
            return Err(JournalError::LegacyCommandQuarantined);
        }

        let existing = transaction
            .query_row(
                "SELECT payload_digest, submission_fingerprint, canonical_command_json
                 FROM canonical_commands WHERE command_id = ?1",
                [command.command_id()],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, Vec<u8>>(2)?,
                    ))
                },
            )
            .optional()?;

        if let Some((payload_digest, submission_fingerprint, prior_json)) = existing {
            if payload_digest != command.payload_digest()
                || submission_fingerprint != command.submission_fingerprint()
                || prior_json != canonical_bytes
            {
                return Err(JournalError::PayloadMismatch);
            }
            transaction.commit()?;
            return Ok(());
        }

        transaction.execute(
            "INSERT INTO canonical_commands (
                command_id, payload_digest, submission_fingerprint, canonical_command_json
             ) VALUES (?1, ?2, ?3, ?4)",
            params![
                command.command_id(),
                command.payload_digest(),
                command.submission_fingerprint(),
                canonical_bytes,
            ],
        )?;
        transaction.commit()?;
        Ok(())
    }
}

fn canonical_command_json(command: &CommandV1) -> Result<String, JournalError> {
    let serialized = serde_json::to_string(command)
        .map_err(|_| JournalError::InvalidInput("CommandV1 serialization failed"))?;
    let limits = JsonInputLimits {
        max_bytes: serialized.len(),
        max_depth: MAX_JSON_DEPTH,
    };
    let canonical = canonicalize_json_text(&serialized, limits)
        .map_err(|_| JournalError::InvalidInput("CommandV1 is not valid canonical input"))?;
    let normative_digest = compute_command_v1_digest_json(&canonical, limits)
        .map_err(|_| JournalError::InvalidInput("CommandV1 failed protocol validation"))?;
    if normative_digest != command.payload_digest() {
        return Err(JournalError::InvalidInput("CommandV1 digest mismatch"));
    }
    Ok(canonical)
}

fn create_current_schema(connection: &Connection) -> Result<(), JournalError> {
    connection.execute_batch(
        "CREATE TABLE canonical_commands (
            command_id TEXT PRIMARY KEY NOT NULL,
            payload_digest TEXT NOT NULL CHECK(length(payload_digest) = 64),
            submission_fingerprint TEXT NOT NULL CHECK(length(submission_fingerprint) = 64),
            canonical_command_json BLOB NOT NULL CHECK(length(canonical_command_json) > 0)
        ) WITHOUT ROWID;
        CREATE TABLE legacy_command_quarantine (
            command_id TEXT PRIMARY KEY NOT NULL,
            source_version INTEGER NOT NULL CHECK(source_version BETWEEN 1 AND 3)
        ) WITHOUT ROWID;",
    )?;
    Ok(())
}

fn migrate(connection: &mut Connection) -> Result<(), JournalError> {
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let version: i64 = transaction.pragma_query_value(None, "user_version", |row| row.get(0))?;
    match version {
        0 => {
            create_current_schema(&transaction)?;
            transaction.pragma_update(None, "user_version", DATABASE_VERSION)?;
            transaction.commit()?;
            Ok(())
        }
        1..=3 => {
            let command_table = format!("quarantined_local_commands_v{version}");
            let outbox_table = format!("quarantined_local_command_outbox_v{version}");
            transaction.execute_batch(&format!(
                "ALTER TABLE local_commands RENAME TO {command_table};
                 ALTER TABLE local_command_outbox RENAME TO {outbox_table};"
            ))?;
            create_current_schema(&transaction)?;
            transaction.execute(
                &format!(
                    "INSERT INTO legacy_command_quarantine(command_id, source_version)
                     SELECT command_id, ?1 FROM {command_table}"
                ),
                [version],
            )?;
            transaction.pragma_update(None, "user_version", DATABASE_VERSION)?;
            transaction.commit()?;
            Ok(())
        }
        DATABASE_VERSION => {
            transaction.commit()?;
            Ok(())
        }
        other => Err(JournalError::UnsupportedDatabaseVersion(other)),
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::{Path, PathBuf},
        sync::{
            Arc, Barrier,
            atomic::{AtomicU64, Ordering},
        },
    };

    use conductor_protocol::{
        CommandV1, JsonInputLimits, compute_command_v1_digest_json, parse_command_v1_json,
    };
    use rusqlite::{Connection, params};

    use super::*;

    const JSON_LIMITS: JsonInputLimits = JsonInputLimits {
        max_bytes: 1_048_576,
        max_depth: 128,
    };
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

    fn command_with(change: impl FnOnce(&mut serde_json::Value)) -> CommandV1 {
        let mut value: serde_json::Value = serde_json::from_str(include_str!(
            "../../../contracts/fixtures/commands/command.valid.json"
        ))
        .expect("valid command fixture");
        change(&mut value);
        let without_current_digest = serde_json::to_string(&value).expect("serialize command");
        let digest = compute_command_v1_digest_json(&without_current_digest, JSON_LIMITS)
            .expect("compute normative digest");
        value["payload_digest"] = serde_json::Value::String(digest);
        let input = serde_json::to_string(&value).expect("serialize command with digest");
        parse_command_v1_json(&input, JSON_LIMITS).expect("parse validated command")
    }

    fn command() -> CommandV1 {
        command_with(|_| {})
    }

    fn command_with_id(id: &str) -> CommandV1 {
        command_with(|value| value["command_id"] = id.into())
    }

    fn create_legacy_database(path: &Path, version: i64) {
        let connection = Connection::open(path).expect("open legacy database");
        connection
            .execute_batch(&format!(
                "CREATE TABLE local_commands (
                    command_id TEXT PRIMARY KEY NOT NULL,
                    state TEXT NOT NULL,
                    payload_json BLOB NOT NULL
                 );
                 CREATE TABLE local_command_outbox (
                    event_id INTEGER PRIMARY KEY AUTOINCREMENT,
                    command_id TEXT NOT NULL REFERENCES local_commands(command_id),
                    event_type TEXT NOT NULL
                 );
                 PRAGMA user_version = {version};"
            ))
            .expect("create legacy schema");
        connection
            .execute(
                "INSERT INTO local_commands(command_id, state, payload_json)
                 VALUES (?1, 'host_accepted', ?2)",
                params![
                    format!("legacy-{version}"),
                    br#"{"raw":"legacy"}"#.as_slice()
                ],
            )
            .expect("insert legacy command");
        connection
            .execute(
                "INSERT INTO local_command_outbox(command_id, event_type) VALUES (?1, 'host_accepted')",
                [format!("legacy-{version}")],
            )
            .expect("insert legacy outbox event");
    }

    #[test]
    fn persists_normative_command_identity_and_survives_reopen() {
        let database = TempDatabase::new();
        let validated = command();
        let first = CommandJournal::open(database.path()).expect("open store");
        first.store(&validated).expect("persist validated command");
        drop(first);

        let reopened = CommandJournal::open(database.path()).expect("reopen store");
        reopened
            .store(&validated)
            .expect("same command replay is idempotent");
        let row: (String, String, Vec<u8>) = reopened
            .connection
            .lock()
            .expect("store lock")
            .query_row(
                "SELECT payload_digest, submission_fingerprint, canonical_command_json
                 FROM canonical_commands WHERE command_id = ?1",
                [validated.command_id()],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .expect("read internal test evidence");
        assert_eq!(row.0, validated.payload_digest());
        assert_eq!(row.1, validated.submission_fingerprint());
        let stored = String::from_utf8(row.2).expect("canonical UTF-8 JSON");
        assert!(!stored.contains('\n'));
        assert_eq!(
            stored,
            canonical_command_json(&validated).expect("canonical command")
        );
    }

    #[test]
    fn rejects_same_id_with_changed_command_or_submission_identity() {
        let database = TempDatabase::new();
        let store = CommandJournal::open(database.path()).expect("open store");
        let original = command();
        store.store(&original).expect("persist command");

        let changed_action = command_with(|value| {
            value["action"]["launch_digest"] = "b".repeat(64).into();
        });
        assert_eq!(changed_action.command_id(), original.command_id());
        assert_ne!(changed_action.payload_digest(), original.payload_digest());
        assert!(matches!(
            store.store(&changed_action),
            Err(JournalError::PayloadMismatch)
        ));

        let changed_fingerprint = command_with(|value| {
            value["submission_fingerprint"] = "f".repeat(64).into();
        });
        assert_eq!(changed_fingerprint.command_id(), original.command_id());
        assert_ne!(
            changed_fingerprint.submission_fingerprint(),
            original.submission_fingerprint()
        );
        assert!(matches!(
            store.store(&changed_fingerprint),
            Err(JournalError::PayloadMismatch)
        ));
    }

    #[test]
    fn quarantines_legacy_rows_without_replaying_or_reusing_their_ids() {
        let database = TempDatabase::new();
        create_legacy_database(database.path(), 3);
        let store = CommandJournal::open(database.path()).expect("migrate legacy store");

        assert!(matches!(
            store.store(&command_with_id("legacy-3")),
            Err(JournalError::LegacyCommandQuarantined)
        ));
        let connection = store.connection.lock().expect("store lock");
        let legacy: (String, String, Vec<u8>) = connection
            .query_row(
                "SELECT command_id, state, payload_json
                 FROM quarantined_local_commands_v3 WHERE command_id = 'legacy-3'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .expect("legacy row retained in quarantine");
        assert_eq!(legacy.0, "legacy-3");
        assert_eq!(legacy.1, "host_accepted");
        assert_eq!(legacy.2, br#"{"raw":"legacy"}"#);
        let new_count: i64 = connection
            .query_row("SELECT count(*) FROM canonical_commands", [], |row| {
                row.get(0)
            })
            .expect("count canonical commands");
        assert_eq!(new_count, 0);
        let quarantined_outbox_count: i64 = connection
            .query_row(
                "SELECT count(*) FROM quarantined_local_command_outbox_v3",
                [],
                |row| row.get(0),
            )
            .expect("count quarantined outbox rows");
        assert_eq!(quarantined_outbox_count, 1);
    }

    #[test]
    fn migrates_every_supported_legacy_schema_version_by_quarantining() {
        for version in 1..=3 {
            let database = TempDatabase::new();
            create_legacy_database(database.path(), version);
            let store = CommandJournal::open(database.path()).expect("migrate legacy store");
            assert!(matches!(
                store.store(&command_with_id(&format!("legacy-{version}"))),
                Err(JournalError::LegacyCommandQuarantined)
            ));
            let connection = store.connection.lock().expect("store lock");
            let row_count: i64 = connection
                .query_row(
                    &format!("SELECT count(*) FROM quarantined_local_commands_v{version}"),
                    [],
                    |row| row.get(0),
                )
                .expect("query versioned quarantine table");
            assert_eq!(row_count, 1);
        }
    }

    #[test]
    fn refuses_to_open_a_future_database_version_without_rewriting_it() {
        let database = TempDatabase::new();
        let legacy = Connection::open(database.path()).expect("create future database");
        legacy
            .pragma_update(None, "user_version", 99_i64)
            .expect("set future schema version");
        drop(legacy);

        assert!(matches!(
            CommandJournal::open(database.path()),
            Err(JournalError::UnsupportedDatabaseVersion(99))
        ));
        let reopened = Connection::open(database.path()).expect("inspect rejected database");
        let version: i64 = reopened
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .expect("read preserved version");
        assert_eq!(version, 99);
    }

    #[test]
    fn failed_insert_leaves_no_partial_command_row() {
        let database = TempDatabase::new();
        let store = CommandJournal::open(database.path()).expect("open store");
        store
            .connection
            .lock()
            .expect("store lock")
            .execute_batch(
                "CREATE TRIGGER reject_command_insert
                 BEFORE INSERT ON canonical_commands
                 BEGIN SELECT RAISE(ABORT, 'injected write failure'); END;",
            )
            .expect("install write failure trigger");

        assert!(matches!(
            store.store(&command()),
            Err(JournalError::Storage(_))
        ));
        let row_count: i64 = store
            .connection
            .lock()
            .expect("store lock")
            .query_row("SELECT count(*) FROM canonical_commands", [], |row| {
                row.get(0)
            })
            .expect("count stored rows");
        assert_eq!(row_count, 0);
    }

    #[test]
    fn rejects_an_in_memory_database_path() {
        assert!(matches!(
            CommandJournal::open(":memory:"),
            Err(JournalError::InvalidInput(_))
        ));
    }

    #[test]
    fn uses_wal_full_synchronous_and_file_backed_storage() {
        let database = TempDatabase::new();
        let store = CommandJournal::open(database.path()).expect("open file-backed store");
        let connection = store.connection.lock().expect("store lock");
        let journal_mode: String = connection
            .query_row("PRAGMA journal_mode", [], |row| row.get(0))
            .expect("read journal mode");
        let synchronous: i64 = connection
            .query_row("PRAGMA synchronous", [], |row| row.get(0))
            .expect("read sync mode");
        let foreign_keys: i64 = connection
            .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
            .expect("read foreign key setting");
        assert_eq!(journal_mode.to_lowercase(), "wal");
        assert_eq!(synchronous, 2);
        assert_eq!(foreign_keys, 1);
    }

    #[test]
    fn concurrent_first_open_and_same_command_writes_are_serialized() {
        let database = TempDatabase::new();
        let open_barrier = Arc::new(Barrier::new(2));
        let first_open = {
            let open_barrier = Arc::clone(&open_barrier);
            let path = database.path().to_path_buf();
            std::thread::spawn(move || {
                open_barrier.wait();
                CommandJournal::open(path).expect("open first store")
            })
        };
        let second_open = {
            let open_barrier = Arc::clone(&open_barrier);
            let path = database.path().to_path_buf();
            std::thread::spawn(move || {
                open_barrier.wait();
                CommandJournal::open(path).expect("open second store")
            })
        };
        let first = Arc::new(first_open.join().expect("first open worker"));
        let second = Arc::new(second_open.join().expect("second open worker"));
        let barrier = Arc::new(Barrier::new(2));
        let command = command();
        let first_thread = {
            let store = Arc::clone(&first);
            let barrier = Arc::clone(&barrier);
            let command = command.clone();
            std::thread::spawn(move || {
                barrier.wait();
                store.store(&command)
            })
        };
        let second_thread = {
            let store = Arc::clone(&second);
            let barrier = Arc::clone(&barrier);
            std::thread::spawn(move || {
                barrier.wait();
                store.store(&command)
            })
        };
        first_thread
            .join()
            .expect("first worker")
            .expect("first store");
        second_thread
            .join()
            .expect("second worker")
            .expect("second store");
        let row_count: i64 = first
            .connection
            .lock()
            .expect("store lock")
            .query_row("SELECT count(*) FROM canonical_commands", [], |row| {
                row.get(0)
            })
            .expect("count stored rows");
        assert_eq!(row_count, 1);
    }
}
