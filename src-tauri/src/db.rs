use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transcription {
    pub id: i64,
    pub text: String,
    pub provider: String,
    pub model: String,
    pub language: Option<String>,
    pub duration_ms: Option<i64>,
    pub created_at: String,
}

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    /// Opens the SQLite database at `{config_dir}/tinywhispr/history.db`
    /// and creates the transcriptions table if it does not exist.
    pub fn new() -> Result<Self, String> {
        let config = dirs::config_dir().expect("Could not determine config directory");
        let dir = config.join("tinywhispr");
        std::fs::create_dir_all(&dir)
            .map_err(|e| format!("Failed to create database directory: {e}"))?;
        let db_path = dir.join("history.db");

        let conn = Connection::open(&db_path)
            .map_err(|e| format!("Failed to open database: {e}"))?;

        Self::init_tables(&conn)?;

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// Creates a Database backed by an in-memory SQLite connection (for testing).
    #[cfg(test)]
    pub fn new_in_memory() -> Result<Self, String> {
        let conn =
            Connection::open_in_memory().map_err(|e| format!("Failed to open in-memory db: {e}"))?;
        Self::init_tables(&conn)?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    fn init_tables(conn: &Connection) -> Result<(), String> {
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS transcriptions (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                text        TEXT NOT NULL,
                provider    TEXT NOT NULL,
                model       TEXT NOT NULL,
                language    TEXT,
                duration_ms INTEGER,
                created_at  TEXT NOT NULL DEFAULT (datetime('now'))
            );",
        )
        .map_err(|e| format!("Failed to create table: {e}"))?;
        Ok(())
    }

    /// Inserts a new transcription and returns the inserted row.
    pub fn insert(
        &self,
        text: &str,
        provider: &str,
        model: &str,
        language: Option<&str>,
        duration_ms: Option<i64>,
    ) -> Result<Transcription, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {e}"))?;
        conn.execute(
            "INSERT INTO transcriptions (text, provider, model, language, duration_ms) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![text, provider, model, language, duration_ms],
        )
        .map_err(|e| format!("Failed to insert transcription: {e}"))?;

        let id = conn.last_insert_rowid();
        let t = conn
            .query_row(
                "SELECT id, text, provider, model, language, duration_ms, created_at FROM transcriptions WHERE id = ?1",
                params![id],
                |row| {
                    Ok(Transcription {
                        id: row.get(0)?,
                        text: row.get(1)?,
                        provider: row.get(2)?,
                        model: row.get(3)?,
                        language: row.get(4)?,
                        duration_ms: row.get(5)?,
                        created_at: row.get(6)?,
                    })
                },
            )
            .map_err(|e| format!("Failed to retrieve inserted transcription: {e}"))?;

        Ok(t)
    }

    /// Returns all transcriptions ordered by `created_at` descending.
    pub fn get_all(&self) -> Result<Vec<Transcription>, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {e}"))?;
        let mut stmt = conn
            .prepare("SELECT id, text, provider, model, language, duration_ms, created_at FROM transcriptions ORDER BY created_at DESC")
            .map_err(|e| format!("Failed to prepare query: {e}"))?;

        let rows = stmt
            .query_map([], |row| {
                Ok(Transcription {
                    id: row.get(0)?,
                    text: row.get(1)?,
                    provider: row.get(2)?,
                    model: row.get(3)?,
                    language: row.get(4)?,
                    duration_ms: row.get(5)?,
                    created_at: row.get(6)?,
                })
            })
            .map_err(|e| format!("Failed to query transcriptions: {e}"))?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row.map_err(|e| format!("Row error: {e}"))?);
        }
        Ok(results)
    }

    /// Deletes a transcription by ID. Returns an error if the row was not found.
    pub fn delete(&self, id: i64) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {e}"))?;
        let affected = conn
            .execute("DELETE FROM transcriptions WHERE id = ?1", params![id])
            .map_err(|e| format!("Failed to delete transcription: {e}"))?;

        if affected == 0 {
            return Err(format!("Transcription with id {id} not found"));
        }
        Ok(())
    }

    /// Deletes all transcriptions.
    pub fn clear_all(&self) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {e}"))?;
        conn.execute("DELETE FROM transcriptions", [])
            .map_err(|e| format!("Failed to clear transcriptions: {e}"))?;
        Ok(())
    }

    /// Searches transcriptions whose text matches the query (case-insensitive LIKE).
    pub fn search(&self, query: &str) -> Result<Vec<Transcription>, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {e}"))?;
        let pattern = format!("%{query}%");
        let mut stmt = conn
            .prepare("SELECT id, text, provider, model, language, duration_ms, created_at FROM transcriptions WHERE text LIKE ?1 ORDER BY created_at DESC")
            .map_err(|e| format!("Failed to prepare search query: {e}"))?;

        let rows = stmt
            .query_map(params![pattern], |row| {
                Ok(Transcription {
                    id: row.get(0)?,
                    text: row.get(1)?,
                    provider: row.get(2)?,
                    model: row.get(3)?,
                    language: row.get(4)?,
                    duration_ms: row.get(5)?,
                    created_at: row.get(6)?,
                })
            })
            .map_err(|e| format!("Failed to search transcriptions: {e}"))?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row.map_err(|e| format!("Row error: {e}"))?);
        }
        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_get() {
        let db = Database::new_in_memory().unwrap();
        let t = db
            .insert("Hello world", "openai", "whisper-1", Some("en"), Some(1500))
            .unwrap();
        assert_eq!(t.text, "Hello world");
        assert_eq!(t.provider, "openai");
        assert_eq!(t.model, "whisper-1");
        assert_eq!(t.language, Some("en".to_string()));
        assert_eq!(t.duration_ms, Some(1500));

        let all = db.get_all().unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].id, t.id);
    }

    #[test]
    fn test_delete() {
        let db = Database::new_in_memory().unwrap();
        let t = db
            .insert("To be deleted", "openai", "whisper-1", None, None)
            .unwrap();

        db.delete(t.id).unwrap();
        let all = db.get_all().unwrap();
        assert!(all.is_empty());

        // Deleting non-existent should error
        assert!(db.delete(9999).is_err());
    }

    #[test]
    fn test_clear_all() {
        let db = Database::new_in_memory().unwrap();
        db.insert("One", "openai", "whisper-1", None, None).unwrap();
        db.insert("Two", "openai", "whisper-1", None, None).unwrap();
        db.insert("Three", "openai", "whisper-1", None, None).unwrap();

        assert_eq!(db.get_all().unwrap().len(), 3);

        db.clear_all().unwrap();
        assert!(db.get_all().unwrap().is_empty());
    }

    #[test]
    fn test_search() {
        let db = Database::new_in_memory().unwrap();
        db.insert("Hello world", "openai", "whisper-1", None, None).unwrap();
        db.insert("Goodbye world", "openai", "whisper-1", None, None).unwrap();
        db.insert("Something else", "openai", "whisper-1", None, None).unwrap();

        let results = db.search("world").unwrap();
        assert_eq!(results.len(), 2);

        let results = db.search("else").unwrap();
        assert_eq!(results.len(), 1);

        let results = db.search("nonexistent").unwrap();
        assert!(results.is_empty());
    }
}
