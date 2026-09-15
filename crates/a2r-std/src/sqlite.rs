//! SQLite backend for the a2r stdlib `sqlite` module (Plan 415-B1).
//!
//! Wraps `rusqlite` (bundled feature) behind the Auto-facing sentinel-error
//! conventions used across a2r-std: never panic, return sentinels and expose
//! the last error via [`last_error`] (see `fs::read_text` and
//! `http::last_status` for the precedent).
//!
//! Hand-copied from `stdlib/auto/sqlite.at` + `sqlite.rs.at` — keep the
//! signatures aligned (checked by the KNOWN-DEBT 396 signature-parity test).

use std::sync::Mutex;

static LAST_ERROR: Mutex<String> = Mutex::new(String::new());

fn record_error(err: &rusqlite::Error) {
    *LAST_ERROR.lock().unwrap() = err.to_string();
}

/// Opaque handle to an open SQLite connection. Closed automatically on
/// drop (RAII) — the Auto surface has no explicit `close`.
pub struct SqliteDb {
    conn: rusqlite::Connection,
}

/// Open (or create) a SQLite database file. On failure, falls back to an
/// in-memory database and records the error for [`last_error`], so later
/// calls remain usable instead of panicking.
pub fn open(path: &str) -> SqliteDb {
    match rusqlite::Connection::open(path) {
        Ok(conn) => SqliteDb { conn },
        Err(e) => {
            *LAST_ERROR.lock().unwrap() = format!("sqlite open '{path}' failed: {e}");
            SqliteDb {
                conn: rusqlite::Connection::open_in_memory()
                    .expect("in-memory sqlite cannot fail"),
            }
        }
    }
}

/// The most recent error message recorded by this module ("" if none).
pub fn last_error() -> String {
    LAST_ERROR.lock().unwrap().clone()
}

fn cell_to_string(value: rusqlite::types::ValueRef<'_>) -> String {
    use rusqlite::types::ValueRef::*;
    match value {
        Null => String::new(),
        Integer(i) => i.to_string(),
        Real(f) => f.to_string(),
        Text(t) => String::from_utf8_lossy(t).into_owned(),
        Blob(b) => format!("<{} bytes>", b.len()),
    }
}

impl SqliteDb {
    /// Execute a SQL statement (DDL or DML). Returns the number of rows
    /// affected, or -1 on failure. Multi-statement scripts are executed as
    /// a batch and report 0 on success.
    pub fn exec(&self, sql: &str) -> i64 {
        match self.conn.execute(sql, []) {
            Ok(n) => n as i64,
            // Batch scripts, and row-returning statements run through exec
            // (e.g. `SELECT 1`), both go through execute_batch: no affected-
            // rows count is available, so success reports 0.
            Err(rusqlite::Error::MultipleStatement)
            | Err(rusqlite::Error::ExecuteReturnedResults) => {
                match self.conn.execute_batch(sql) {
                    Ok(()) => 0,
                    Err(e) => {
                        record_error(&e);
                        -1
                    }
                }
            }
            Err(e) => {
                record_error(&e);
                -1
            }
        }
    }

    /// Run a SELECT query; every cell of every row is returned as a string
    /// (NULL → "", BLOB → "<N bytes>"). Returns an empty list on failure.
    pub fn query(&self, sql: &str) -> Vec<Vec<String>> {
        let mut stmt = match self.conn.prepare(sql) {
            Ok(stmt) => stmt,
            Err(e) => {
                record_error(&e);
                return Vec::new();
            }
        };
        let ncols = stmt.column_count();
        let mut rows = match stmt.query([]) {
            Ok(rows) => rows,
            Err(e) => {
                record_error(&e);
                return Vec::new();
            }
        };
        let mut out = Vec::new();
        loop {
            match rows.next() {
                Ok(Some(row)) => {
                    let mut cells = Vec::with_capacity(ncols);
                    for i in 0..ncols {
                        let cell = row
                            .get_ref(i)
                            .map(cell_to_string)
                            .unwrap_or_default();
                        cells.push(cell);
                    }
                    out.push(cells);
                }
                Ok(None) => break,
                Err(e) => {
                    record_error(&e);
                    return Vec::new();
                }
            }
        }
        out
    }

    /// Rowid of the most recent successful INSERT on this connection
    /// (0 if none).
    pub fn last_insert_rowid(&self) -> i64 {
        self.conn.last_insert_rowid()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sqlite_roundtrip_in_memory() {
        let db = open(":memory:");
        let n = db.exec("CREATE TABLE cats (id INTEGER PRIMARY KEY, name TEXT)");
        assert_eq!(n, 0);
        let n = db.exec("INSERT INTO cats (name) VALUES ('Michi'), ('Nori')");
        assert_eq!(n, 2);
        assert_eq!(db.last_insert_rowid(), 2);
        let rows = db.query("SELECT id, name FROM cats ORDER BY id");
        assert_eq!(
            rows,
            vec![
                vec!["1".to_string(), "Michi".to_string()],
                vec!["2".to_string(), "Nori".to_string()],
            ]
        );
        // NULL cells come back as "".
        db.exec("INSERT INTO cats (id, name) VALUES (3, NULL)");
        let rows = db.query("SELECT name FROM cats WHERE id = 3");
        assert_eq!(rows, vec![vec!["".to_string()]]);
    }

    #[test]
    fn sqlite_failure_sentinels() {
        // Bad SQL records the error and returns the exec sentinel.
        let db = open(":memory:");
        assert_eq!(db.exec("NOT VALID SQL"), -1);
        assert!(!last_error().is_empty());
        assert!(db.query("ALSO NOT VALID").is_empty());
        // A failed file open falls back to in-memory (still usable).
        let db = open("Z:/no/such/dir/x_415b.db");
        assert!(!last_error().is_empty());
        assert_eq!(db.exec("SELECT 1"), 0);
    }
}
