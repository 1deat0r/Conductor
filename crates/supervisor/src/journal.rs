use rusqlite::{Connection, OpenFlags, OptionalExtension, TransactionBehavior, params};
use std::error::Error;
use std::fmt;
use std::path::Path;

const SCHEMA_VERSION: i64 = 1;

#[derive(Debug)]
pub enum JournalError {
    Database(rusqlite::Error),
    EmptySessionId,
    EmptyLaunchDigest,
    LaunchDigestConflict,
    NonPersistentPath,
    UnsupportedJournalMode(String),
    UnsupportedSynchronousMode(i64),
    UnsupportedSchemaVersion(i64),
    UnknownLaunchState(String),
}

impl fmt::Display for JournalError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Database(error) => write!(formatter, "SQLite error: {error}"),
            Self::EmptySessionId => formatter.write_str("session ID must not be empty"),
            Self::EmptyLaunchDigest => formatter.write_str("launch digest must not be empty"),
            Self::LaunchDigestConflict => {
                formatter.write_str("session ID is already bound to a different launch digest")
            }
            Self::NonPersistentPath => {
                formatter.write_str("journal requires a persistent file-backed database path")
            }
            Self::UnsupportedJournalMode(mode) => {
                write!(
                    formatter,
                    "SQLite did not enable WAL mode (reported {mode})"
                )
            }
            Self::UnsupportedSynchronousMode(mode) => write!(
                formatter,
                "SQLite did not enable FULL synchronous mode (reported {mode})"
            ),
            Self::UnsupportedSchemaVersion(version) => {
                write!(
                    formatter,
                    "unsupported supervisor journal schema version {version}"
                )
            }
            Self::UnknownLaunchState(state) => {
                write!(formatter, "unknown supervisor launch state {state}")
            }
        }
    }
}

impl Error for JournalError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Database(error) => Some(error),
            _ => None,
        }
    }
}

impl From<rusqlite::Error> for JournalError {
    fn from(error: rusqlite::Error) -> Self {
        Self::Database(error)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LaunchState {
    LaunchPending,
    InterruptedOutcomeUnknown,
}

impl LaunchState {
    fn as_database_value(self) -> &'static str {
        match self {
            Self::LaunchPending => "launch-pending",
            Self::InterruptedOutcomeUnknown => "interrupted-outcome-unknown",
        }
    }

    fn from_database_value(value: String) -> Result<Self, JournalError> {
        match value.as_str() {
            "launch-pending" => Ok(Self::LaunchPending),
            "interrupted-outcome-unknown" => Ok(Self::InterruptedOutcomeUnknown),
            _ => Err(JournalError::UnknownLaunchState(value)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaunchIntent {
    pub session_id: String,
    pub launch_digest: String,
    pub state: LaunchState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntentDisposition {
    /// This call inserted and durably committed a new intent.
    Created,
    /// A matching intent already existed; this call must not start another process.
    Existing,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[must_use = "inspect the disposition before deciding whether to start a process"]
pub struct StartIntentResult {
    pub intent: LaunchIntent,
    pub disposition: IntentDisposition,
}

pub struct SessionJournal {
    connection: Connection,
}

impl SessionJournal {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, JournalError> {
        let path = path.as_ref();
        if path.as_os_str().is_empty() || path == Path::new(":memory:") {
            return Err(JournalError::NonPersistentPath);
        }

        let flags = OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE;
        let mut connection = Connection::open_with_flags(path, flags)?;
        let journal_mode: String =
            connection.query_row("PRAGMA journal_mode=WAL", [], |row| row.get(0))?;
        if !journal_mode.eq_ignore_ascii_case("wal") {
            return Err(JournalError::UnsupportedJournalMode(journal_mode));
        }

        connection.pragma_update(None, "synchronous", "FULL")?;
        let synchronous: i64 =
            connection.pragma_query_value(None, "synchronous", |row| row.get(0))?;
        if synchronous != 2 {
            return Err(JournalError::UnsupportedSynchronousMode(synchronous));
        }

        Self::migrate(&mut connection)?;
        Ok(Self { connection })
    }

    pub fn ensure_start_intent(
        &mut self,
        session_id: &str,
        launch_digest: &str,
    ) -> Result<StartIntentResult, JournalError> {
        if session_id.trim().is_empty() {
            return Err(JournalError::EmptySessionId);
        }
        if launch_digest.trim().is_empty() {
            return Err(JournalError::EmptyLaunchDigest);
        }

        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let existing: Option<(String, String)> = transaction
            .query_row(
                "SELECT launch_digest, state FROM supervisor_launch_intents WHERE session_id = ?1",
                [session_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?;

        if let Some((stored_digest, stored_state)) = existing {
            if stored_digest != launch_digest {
                return Err(JournalError::LaunchDigestConflict);
            }
            let state = LaunchState::from_database_value(stored_state)?;
            transaction.commit()?;
            return Ok(StartIntentResult {
                intent: LaunchIntent {
                    session_id: session_id.to_owned(),
                    launch_digest: stored_digest,
                    state,
                },
                disposition: IntentDisposition::Existing,
            });
        }

        transaction.execute(
            "INSERT INTO supervisor_launch_intents (session_id, launch_digest, state) VALUES (?1, ?2, ?3)",
            params![session_id, launch_digest, LaunchState::LaunchPending.as_database_value()],
        )?;
        transaction.commit()?;

        Ok(StartIntentResult {
            intent: LaunchIntent {
                session_id: session_id.to_owned(),
                launch_digest: launch_digest.to_owned(),
                state: LaunchState::LaunchPending,
            },
            disposition: IntentDisposition::Created,
        })
    }

    pub fn find_launch_intent(
        &self,
        session_id: &str,
    ) -> Result<Option<LaunchIntent>, JournalError> {
        let intent = self
            .connection
            .query_row(
                "SELECT session_id, launch_digest, state FROM supervisor_launch_intents WHERE session_id = ?1",
                [session_id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                    ))
                },
            )
            .optional()?;

        intent
            .map(|(session_id, launch_digest, state)| {
                Ok(LaunchIntent {
                    session_id,
                    launch_digest,
                    state: LaunchState::from_database_value(state)?,
                })
            })
            .transpose()
    }

    /// Marks unfinished intents as uncertain during restart recovery.
    ///
    /// Call this after establishing exclusive supervisor ownership and before
    /// accepting new start requests. It is not safe to call during normal
    /// operation while another supervisor may own a pending intent.
    pub fn reconcile_pending_after_restart(&mut self) -> Result<usize, JournalError> {
        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let reconciled = transaction.execute(
            "UPDATE supervisor_launch_intents SET state = ?1 WHERE state = ?2",
            params![
                LaunchState::InterruptedOutcomeUnknown.as_database_value(),
                LaunchState::LaunchPending.as_database_value()
            ],
        )?;
        transaction.commit()?;
        Ok(reconciled)
    }

    fn migrate(connection: &mut Connection) -> Result<(), JournalError> {
        let schema_version: i64 =
            connection.pragma_query_value(None, "user_version", |row| row.get(0))?;
        match schema_version {
            0 => {
                let transaction =
                    connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
                transaction.execute_batch(
                    "CREATE TABLE supervisor_launch_intents (
                        session_id TEXT PRIMARY KEY NOT NULL,
                        launch_digest TEXT NOT NULL,
                        state TEXT NOT NULL CHECK (state IN ('launch-pending', 'interrupted-outcome-unknown'))
                    )",
                )?;
                transaction.pragma_update(None, "user_version", SCHEMA_VERSION)?;
                transaction.commit()?;
            }
            SCHEMA_VERSION => {}
            other => return Err(JournalError::UnsupportedSchemaVersion(other)),
        }
        Ok(())
    }
}
