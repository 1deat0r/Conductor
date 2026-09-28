fn main() {}

#[cfg(test)]
mod tests {
    use rusqlite::Connection;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn bundled_sqlite_wal_full_recovers_committed_intent() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("conductor-msrv-{}-{nonce}.db", std::process::id()));
        let connection = Connection::open(&path).expect("open sqlite database");
        let journal_mode: String = connection
            .query_row("PRAGMA journal_mode=WAL", [], |row| row.get(0))
            .expect("enable WAL");
        assert_eq!(journal_mode.to_ascii_lowercase(), "wal");
        connection
            .execute_batch("PRAGMA synchronous=FULL; CREATE TABLE launch_intents (session_id TEXT PRIMARY KEY, launch_digest TEXT NOT NULL, state TEXT NOT NULL);")
            .expect("create journal table");
        let synchronous: i64 = connection
            .query_row("PRAGMA synchronous", [], |row| row.get(0))
            .expect("read synchronous mode");
        assert_eq!(synchronous, 2);
        let transaction = connection
            .unchecked_transaction()
            .expect("begin transaction");
        transaction
            .execute(
                "INSERT INTO launch_intents VALUES (?1, ?2, ?3)",
                ("session-1", "digest-1", "launch-pending"),
            )
            .expect("write durable intent");
        transaction.commit().expect("commit durable intent");
        drop(connection);

        let reopened = Connection::open(&path).expect("reopen sqlite database");
        reopened
            .execute_batch("PRAGMA synchronous=FULL;")
            .expect("restore full synchronous mode");
        let state: String = reopened
            .query_row(
                "SELECT state FROM launch_intents WHERE session_id = ?1 AND launch_digest = ?2",
                ("session-1", "digest-1"),
                |row| row.get(0),
            )
            .expect("read committed intent after reopen");
        assert_eq!(state, "launch-pending");
        drop(reopened);
        let _ = std::fs::remove_file(path);
    }
}
