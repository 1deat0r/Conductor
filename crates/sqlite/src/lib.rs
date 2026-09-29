use std::{error::Error, fmt, thread, time::Duration, time::Instant};

use rusqlite::Connection;

const BUSY_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Debug)]
pub enum ConfigurationError {
    Storage(rusqlite::Error),
    RequiredSetting(&'static str),
}

impl fmt::Display for ConfigurationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Storage(error) => write!(formatter, "SQLite configuration error: {error}"),
            Self::RequiredSetting(setting) => {
                write!(
                    formatter,
                    "SQLite did not enable required setting {setting}"
                )
            }
        }
    }
}

impl Error for ConfigurationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Storage(error) => Some(error),
            Self::RequiredSetting(_) => None,
        }
    }
}

impl From<rusqlite::Error> for ConfigurationError {
    fn from(error: rusqlite::Error) -> Self {
        Self::Storage(error)
    }
}

/// Applies the durability settings required by Conductor's file-backed stores.
pub fn configure_durable_connection(connection: &Connection) -> Result<(), ConfigurationError> {
    connection.busy_timeout(BUSY_TIMEOUT)?;
    enable_wal(connection)?;

    connection.pragma_update(None, "synchronous", "FULL")?;
    let synchronous: i64 = connection.query_row("PRAGMA synchronous", [], |row| row.get(0))?;
    if synchronous != 2 {
        return Err(ConfigurationError::RequiredSetting("synchronous=FULL"));
    }

    connection.pragma_update(None, "foreign_keys", "ON")?;
    let foreign_keys: i64 = connection.query_row("PRAGMA foreign_keys", [], |row| row.get(0))?;
    if foreign_keys != 1 {
        return Err(ConfigurationError::RequiredSetting("foreign_keys=ON"));
    }
    Ok(())
}

fn enable_wal(connection: &Connection) -> Result<(), ConfigurationError> {
    let started = Instant::now();
    loop {
        let journal_mode = connection.query_row("PRAGMA journal_mode = WAL", [], |row| {
            row.get::<_, String>(0)
        });
        match journal_mode {
            Ok(mode) if mode.eq_ignore_ascii_case("wal") => return Ok(()),
            Ok(_) => return Err(ConfigurationError::RequiredSetting("journal_mode=WAL")),
            Err(error) if is_database_busy(&error) && started.elapsed() < BUSY_TIMEOUT => {
                thread::sleep(Duration::from_millis(10));
            }
            Err(error) => return Err(ConfigurationError::Storage(error)),
        }
    }
}

fn is_database_busy(error: &rusqlite::Error) -> bool {
    matches!(
        error,
        rusqlite::Error::SqliteFailure(code, _)
            if matches!(
                code.code,
                rusqlite::ErrorCode::DatabaseBusy | rusqlite::ErrorCode::DatabaseLocked
            )
    )
}
