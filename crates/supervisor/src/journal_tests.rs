use crate::journal::{IntentDisposition, JournalError, LaunchState, SessionJournal};
use rusqlite::Connection;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

struct TemporaryDatabase {
    directory: PathBuf,
}

impl TemporaryDatabase {
    fn new() -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after the Unix epoch")
            .as_nanos();
        let directory = std::env::temp_dir().join(format!(
            "conductor-supervisor-journal-{}-{timestamp}",
            std::process::id()
        ));
        std::fs::create_dir(&directory).expect("create isolated test database directory");
        Self { directory }
    }

    fn path(&self) -> PathBuf {
        self.directory.join("journal.sqlite")
    }
}

impl Drop for TemporaryDatabase {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.directory);
    }
}

#[test]
fn uncertain_start_after_restart_is_not_reauthorized_by_duplicate() {
    let database = TemporaryDatabase::new();
    {
        let mut journal = SessionJournal::open(database.path()).expect("open journal");
        let intent = journal
            .ensure_start_intent("session-1", "digest-1")
            .expect("persist start intent");
        assert_eq!(intent.disposition, IntentDisposition::Created);
        assert_eq!(intent.intent.state, LaunchState::LaunchPending);

        let duplicate = journal
            .ensure_start_intent("session-1", "digest-1")
            .expect("find existing start intent");
        assert_eq!(duplicate.disposition, IntentDisposition::Existing);
        assert_eq!(duplicate.intent.state, LaunchState::LaunchPending);
    }

    let mut journal = SessionJournal::open(database.path()).expect("reopen journal");
    assert_eq!(
        journal
            .reconcile_pending_after_restart()
            .expect("reconcile incomplete start"),
        1
    );
    assert_eq!(
        journal
            .reconcile_pending_after_restart()
            .expect("reconciliation is idempotent"),
        0
    );

    let recovered = journal
        .find_launch_intent("session-1")
        .expect("query recovered intent")
        .expect("intent still exists");
    assert_eq!(recovered.state, LaunchState::InterruptedOutcomeUnknown);

    let duplicate = journal
        .ensure_start_intent("session-1", "digest-1")
        .expect("same payload returns prior attempt");
    assert_eq!(duplicate.disposition, IntentDisposition::Existing);
    assert_eq!(
        duplicate.intent.state,
        LaunchState::InterruptedOutcomeUnknown
    );
    assert!(matches!(
        journal.ensure_start_intent("session-1", "different-digest"),
        Err(JournalError::LaunchDigestConflict)
    ));
}

#[test]
fn rejects_empty_identifiers_and_requires_a_file_backed_database() {
    let database = TemporaryDatabase::new();
    let mut journal = SessionJournal::open(database.path()).expect("open journal");

    assert!(matches!(
        journal.ensure_start_intent("  ", "digest-1"),
        Err(JournalError::EmptySessionId)
    ));
    assert!(matches!(
        journal.ensure_start_intent("session-1", "\t"),
        Err(JournalError::EmptyLaunchDigest)
    ));
    assert_eq!(
        journal
            .find_launch_intent("session-1")
            .expect("query journal"),
        None
    );
    assert!(matches!(
        SessionJournal::open(":memory:"),
        Err(JournalError::NonPersistentPath)
    ));
    assert!(matches!(
        SessionJournal::open(""),
        Err(JournalError::NonPersistentPath)
    ));
}

#[test]
fn session_identifiers_are_bound_as_data() {
    let database = TemporaryDatabase::new();
    let mut journal = SessionJournal::open(database.path()).expect("open journal");
    let hostile_identifier = "session'; DROP TABLE supervisor_launch_intents; --";

    let result = journal
        .ensure_start_intent(hostile_identifier, "digest-1")
        .expect("store arbitrary session identifier as data");

    assert_eq!(result.disposition, IntentDisposition::Created);
    assert_eq!(result.intent.session_id, hostile_identifier);
    assert_eq!(
        journal
            .find_launch_intent(hostile_identifier)
            .expect("query arbitrary session identifier")
            .expect("intent exists"),
        result.intent
    );
    assert!(
        journal
            .find_launch_intent("another-session")
            .expect("journal table remains available")
            .is_none()
    );
}

#[test]
fn refuses_unknown_schema_versions() {
    let database = TemporaryDatabase::new();
    let connection = Connection::open(database.path()).expect("create future-version database");
    connection
        .pragma_update(None, "user_version", 42)
        .expect("mark unsupported schema version");
    drop(connection);

    assert!(matches!(
        SessionJournal::open(database.path()),
        Err(JournalError::UnsupportedSchemaVersion(42))
    ));
}
