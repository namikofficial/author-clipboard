//! Database operations using `SQLite`

use crate::types::ClipboardItem;
use rusqlite::{Connection, Result as SqlResult};
use std::path::Path;

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn open(path: &Path) -> SqlResult<Self> {
        let conn = Connection::open(path)?;
        let db = Self { conn };
        db.init_schema()?;
        Ok(db)
    }

    /// Create an in-memory database (useful for testing)
    pub fn open_in_memory() -> SqlResult<Self> {
        let conn = Connection::open_in_memory()?;
        let db = Self { conn };
        db.init_schema()?;
        Ok(db)
    }

    fn init_schema(&self) -> SqlResult<()> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS clipboard_items (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                content TEXT NOT NULL,
                mime_type TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                pinned INTEGER NOT NULL DEFAULT 0,
                source_app TEXT
            );
            CREATE INDEX IF NOT EXISTS idx_timestamp ON clipboard_items(timestamp DESC);",
        )?;
        Ok(())
    }

    pub fn insert_item(&self, item: &ClipboardItem) -> SqlResult<i64> {
        self.conn.execute(
            "INSERT INTO clipboard_items (content, mime_type, timestamp, pinned, source_app)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            (
                &item.content,
                &item.mime_type,
                item.timestamp.to_rfc3339(),
                i32::from(item.pinned),
                &item.source_app,
            ),
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn get_recent(&self, limit: usize) -> SqlResult<Vec<ClipboardItem>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, content, mime_type, timestamp, pinned, source_app
             FROM clipboard_items
             ORDER BY pinned DESC, timestamp DESC
             LIMIT ?1",
        )?;

        let items = stmt.query_map([limit], |row| {
            Ok(ClipboardItem {
                id: row.get(0)?,
                content: row.get(1)?,
                mime_type: row.get(2)?,
                timestamp: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(3)?)
                    .map_or_else(|_| chrono::Utc::now(), |dt| dt.with_timezone(&chrono::Utc)),
                pinned: row.get::<_, i32>(4)? != 0,
                source_app: row.get(5)?,
            })
        })?;

        items.collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_query() {
        let db = Database::open_in_memory().unwrap();
        let item = ClipboardItem::new_text("hello world".to_string());
        let id = db.insert_item(&item).unwrap();
        assert!(id > 0);

        let items = db.get_recent(10).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].content, "hello world");
    }

    #[test]
    fn test_multiple_items() {
        let db = Database::open_in_memory().unwrap();
        db.insert_item(&ClipboardItem::new_text("first".to_string()))
            .unwrap();
        db.insert_item(&ClipboardItem::new_text("second".to_string()))
            .unwrap();
        db.insert_item(&ClipboardItem::new_text("third".to_string()))
            .unwrap();

        let items = db.get_recent(10).unwrap();
        assert_eq!(items.len(), 3);
        // Most recent first
        assert_eq!(items[0].content, "third");
    }
}
