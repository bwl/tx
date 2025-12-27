//! SQLite database for transcript history.

use anyhow::{Context, Result};
use chrono::{DateTime, Local};
use rusqlite::Connection;
use std::path::PathBuf;

/// A stored transcript record.
#[derive(Debug)]
pub struct Transcript {
    pub id: String,
    pub text: String,
    pub timestamp: DateTime<Local>,
    pub working_dir: String,
}

/// Returns the path to the database file.
fn db_path() -> Result<PathBuf> {
    let data_dir = dirs::data_local_dir()
        .context("Cannot determine local data directory")?
        .join("tx");
    std::fs::create_dir_all(&data_dir)?;
    Ok(data_dir.join("history.db"))
}

/// Opens a connection to the database, creating it if needed.
pub fn open() -> Result<Connection> {
    let path = db_path()?;
    let conn = Connection::open(&path)?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS transcripts (
            id TEXT PRIMARY KEY,
            text TEXT NOT NULL,
            timestamp TEXT NOT NULL,
            working_dir TEXT NOT NULL
        )",
        [],
    )?;

    Ok(conn)
}

/// Generates a short ID from the text and timestamp.
fn generate_id(text: &str, timestamp: &DateTime<Local>) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    text.hash(&mut hasher);
    timestamp.to_rfc3339().hash(&mut hasher);
    let hash = hasher.finish();

    format!("{:x}", hash)[..7].to_string()
}

/// Saves a transcript and returns its ID.
pub fn save(conn: &Connection, text: &str, working_dir: &str) -> Result<String> {
    let timestamp = Local::now();
    let id = generate_id(text, &timestamp);

    conn.execute(
        "INSERT OR REPLACE INTO transcripts (id, text, timestamp, working_dir) VALUES (?1, ?2, ?3, ?4)",
        (&id, text, timestamp.to_rfc3339(), working_dir),
    )?;

    Ok(id)
}

/// Lists recent transcripts.
pub fn list(conn: &Connection, limit: usize) -> Result<Vec<Transcript>> {
    let mut stmt = conn.prepare(
        "SELECT id, text, timestamp, working_dir FROM transcripts ORDER BY timestamp DESC LIMIT ?1",
    )?;

    let rows = stmt.query_map([limit], |row| {
        let timestamp_str: String = row.get(2)?;
        let timestamp = DateTime::parse_from_rfc3339(&timestamp_str)
            .map(|dt| dt.with_timezone(&Local))
            .unwrap_or_else(|_| Local::now());

        Ok(Transcript {
            id: row.get(0)?,
            text: row.get(1)?,
            timestamp,
            working_dir: row.get(3)?,
        })
    })?;

    let mut transcripts = Vec::new();
    for row in rows {
        transcripts.push(row?);
    }

    Ok(transcripts)
}

/// Finds a transcript by ID prefix.
pub fn find_by_prefix(conn: &Connection, prefix: &str) -> Result<Option<Transcript>> {
    let mut stmt = conn.prepare(
        "SELECT id, text, timestamp, working_dir FROM transcripts WHERE id LIKE ?1 || '%' LIMIT 1",
    )?;

    let mut rows = stmt.query_map([prefix], |row| {
        let timestamp_str: String = row.get(2)?;
        let timestamp = DateTime::parse_from_rfc3339(&timestamp_str)
            .map(|dt| dt.with_timezone(&Local))
            .unwrap_or_else(|_| Local::now());

        Ok(Transcript {
            id: row.get(0)?,
            text: row.get(1)?,
            timestamp,
            working_dir: row.get(3)?,
        })
    })?;

    match rows.next() {
        Some(Ok(t)) => Ok(Some(t)),
        Some(Err(e)) => Err(e.into()),
        None => Ok(None),
    }
}
