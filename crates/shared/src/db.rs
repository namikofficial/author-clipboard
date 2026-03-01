//! Database operations using `SQLite`

use crate::types::{ClipboardItem, DbStats};
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
                content_hash INTEGER NOT NULL,
                content TEXT NOT NULL,
                mime_type TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                pinned INTEGER NOT NULL DEFAULT 0,
                source_app TEXT
            );
            CREATE INDEX IF NOT EXISTS idx_timestamp ON clipboard_items(timestamp DESC);
            CREATE INDEX IF NOT EXISTS idx_content_hash ON clipboard_items(content_hash);
            CREATE INDEX IF NOT EXISTS idx_pinned ON clipboard_items(pinned);",
        )?;
        Ok(())
    }

    // ── Insert / Dedup ────────────────────────────────────────────────

    /// Insert a new item. Returns the row id.
    pub fn insert_item(&self, item: &ClipboardItem) -> SqlResult<i64> {
        self.conn.execute(
            "INSERT INTO clipboard_items
                (content_hash, content, mime_type, timestamp, pinned, source_app)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            (
                item.content_hash.cast_signed(),
                &item.content,
                &item.mime_type,
                item.timestamp.to_rfc3339(),
                i32::from(item.pinned),
                &item.source_app,
            ),
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// Insert only if content hash doesn't already exist.
    /// If duplicate, bumps the existing item's timestamp instead.
    /// Returns the id of the inserted or bumped row.
    pub fn insert_or_bump(&self, item: &ClipboardItem) -> SqlResult<i64> {
        if let Some(existing_id) = self.find_by_hash(item.content_hash)? {
            self.conn.execute(
                "UPDATE clipboard_items SET timestamp = ?1 WHERE id = ?2",
                (item.timestamp.to_rfc3339(), existing_id),
            )?;
            Ok(existing_id)
        } else {
            self.insert_item(item)
        }
    }

    /// Find an item by content hash (for deduplication).
    pub fn find_by_hash(&self, hash: u64) -> SqlResult<Option<i64>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id FROM clipboard_items WHERE content_hash = ?1 LIMIT 1")?;
        let mut rows = stmt.query_map([hash.cast_signed()], |row| row.get::<_, i64>(0))?;
        match rows.next() {
            Some(Ok(id)) => Ok(Some(id)),
            _ => Ok(None),
        }
    }

    // ── Query ─────────────────────────────────────────────────────────

    /// Get the most recent items, pinned first.
    pub fn get_recent(&self, limit: usize) -> SqlResult<Vec<ClipboardItem>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, content_hash, content, mime_type, timestamp, pinned, source_app
             FROM clipboard_items
             ORDER BY pinned DESC, timestamp DESC
             LIMIT ?1",
        )?;
        Self::collect_items(&mut stmt, [limit])
    }

    /// Search items by content substring (case-insensitive).
    pub fn search(&self, query: &str, limit: usize) -> SqlResult<Vec<ClipboardItem>> {
        let pattern = format!("%{query}%");
        let mut stmt = self.conn.prepare(
            "SELECT id, content_hash, content, mime_type, timestamp, pinned, source_app
             FROM clipboard_items
             WHERE content LIKE ?1
             ORDER BY pinned DESC, timestamp DESC
             LIMIT ?2",
        )?;
        Self::collect_items(&mut stmt, (&pattern as &dyn rusqlite::ToSql, &limit))
    }

    /// Get a single item by id.
    pub fn get_by_id(&self, id: i64) -> SqlResult<Option<ClipboardItem>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, content_hash, content, mime_type, timestamp, pinned, source_app
             FROM clipboard_items WHERE id = ?1",
        )?;
        let mut items = stmt.query_map([id], Self::row_to_item)?;
        match items.next() {
            Some(Ok(item)) => Ok(Some(item)),
            Some(Err(e)) => Err(e),
            None => Ok(None),
        }
    }

    // ── Pin / Unpin ───────────────────────────────────────────────────

    /// Toggle the pinned state of an item. Returns the new pinned value.
    pub fn toggle_pin(&self, id: i64) -> SqlResult<bool> {
        self.conn.execute(
            "UPDATE clipboard_items SET pinned = NOT pinned WHERE id = ?1",
            [id],
        )?;
        let pinned: bool = self.conn.query_row(
            "SELECT pinned FROM clipboard_items WHERE id = ?1",
            [id],
            |row| row.get(0),
        )?;
        Ok(pinned)
    }

    /// Set pinned state explicitly.
    pub fn set_pinned(&self, id: i64, pinned: bool) -> SqlResult<()> {
        self.conn.execute(
            "UPDATE clipboard_items SET pinned = ?1 WHERE id = ?2",
            (i32::from(pinned), id),
        )?;
        Ok(())
    }

    // ── Delete ────────────────────────────────────────────────────────

    /// Delete a single item by id.
    pub fn delete_item(&self, id: i64) -> SqlResult<bool> {
        let affected = self
            .conn
            .execute("DELETE FROM clipboard_items WHERE id = ?1", [id])?;
        Ok(affected > 0)
    }

    /// Delete all non-pinned items.
    pub fn clear_unpinned(&self) -> SqlResult<usize> {
        let affected = self
            .conn
            .execute("DELETE FROM clipboard_items WHERE pinned = 0", [])?;
        Ok(affected)
    }

    /// Delete all items (including pinned).
    pub fn clear_all(&self) -> SqlResult<usize> {
        let affected = self.conn.execute("DELETE FROM clipboard_items", [])?;
        Ok(affected)
    }

    // ── Cleanup / Limits ──────────────────────────────────────────────

    /// Enforce maximum item count. Deletes oldest non-pinned items over the limit.
    pub fn enforce_max_items(&self, max_items: usize) -> SqlResult<usize> {
        let affected = self.conn.execute(
            "DELETE FROM clipboard_items WHERE id IN (
                SELECT id FROM clipboard_items
                WHERE pinned = 0
                ORDER BY timestamp DESC
                LIMIT -1 OFFSET ?1
            )",
            [max_items],
        )?;
        Ok(affected)
    }

    /// Delete non-pinned items older than the given timestamp.
    pub fn delete_expired(&self, before: &chrono::DateTime<chrono::Utc>) -> SqlResult<usize> {
        let affected = self.conn.execute(
            "DELETE FROM clipboard_items WHERE pinned = 0 AND timestamp < ?1",
            [before.to_rfc3339()],
        )?;
        Ok(affected)
    }

    /// Get database statistics.
    pub fn get_stats(&self) -> SqlResult<DbStats> {
        let total_items: usize =
            self.conn
                .query_row("SELECT COUNT(*) FROM clipboard_items", [], |row| row.get(0))?;
        let pinned_items: usize = self.conn.query_row(
            "SELECT COUNT(*) FROM clipboard_items WHERE pinned = 1",
            [],
            |row| row.get(0),
        )?;
        let total_size_bytes: u64 = self.conn.query_row(
            "SELECT COALESCE(SUM(LENGTH(content)), 0) FROM clipboard_items",
            [],
            |row| row.get(0),
        )?;
        Ok(DbStats {
            total_items,
            pinned_items,
            total_size_bytes,
        })
    }

    // ── Helpers ────────────────────────────────────────────────────────

    fn row_to_item(row: &rusqlite::Row<'_>) -> SqlResult<ClipboardItem> {
        Ok(ClipboardItem {
            id: row.get(0)?,
            content_hash: row.get::<_, i64>(1)?.cast_unsigned(),
            content: row.get(2)?,
            mime_type: row.get(3)?,
            timestamp: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(4)?)
                .map_or_else(|_| chrono::Utc::now(), |dt| dt.with_timezone(&chrono::Utc)),
            pinned: row.get::<_, i32>(5)? != 0,
            source_app: row.get(6)?,
        })
    }

    fn collect_items<P: rusqlite::Params>(
        stmt: &mut rusqlite::Statement<'_>,
        params: P,
    ) -> SqlResult<Vec<ClipboardItem>> {
        let items = stmt.query_map(params, Self::row_to_item)?;
        items.collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_db() -> Database {
        Database::open_in_memory().unwrap()
    }

    #[test]
    fn test_insert_and_query() {
        let db = make_db();
        let item = ClipboardItem::new_text("hello world".to_string());
        let id = db.insert_item(&item).unwrap();
        assert!(id > 0);

        let items = db.get_recent(10).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].content, "hello world");
        assert!(items[0].content_hash > 0);
    }

    #[test]
    fn test_dedup_insert_or_bump() {
        let db = make_db();
        let item1 = ClipboardItem::new_text("duplicate".to_string());
        let id1 = db.insert_or_bump(&item1).unwrap();

        let item2 = ClipboardItem::new_text("duplicate".to_string());
        let id2 = db.insert_or_bump(&item2).unwrap();

        assert_eq!(id1, id2, "Same content should return same id");
        assert_eq!(
            db.get_recent(10).unwrap().len(),
            1,
            "Should still be 1 item"
        );
    }

    #[test]
    fn test_search() {
        let db = make_db();
        db.insert_item(&ClipboardItem::new_text("hello world".to_string()))
            .unwrap();
        db.insert_item(&ClipboardItem::new_text("foo bar".to_string()))
            .unwrap();
        db.insert_item(&ClipboardItem::new_text("hello rust".to_string()))
            .unwrap();

        let results = db.search("hello", 10).unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_pin_toggle() {
        let db = make_db();
        let id = db
            .insert_item(&ClipboardItem::new_text("pin me".to_string()))
            .unwrap();

        let pinned = db.toggle_pin(id).unwrap();
        assert!(pinned);

        let pinned = db.toggle_pin(id).unwrap();
        assert!(!pinned);
    }

    #[test]
    fn test_delete() {
        let db = make_db();
        let id = db
            .insert_item(&ClipboardItem::new_text("delete me".to_string()))
            .unwrap();
        assert!(db.delete_item(id).unwrap());
        assert_eq!(db.get_recent(10).unwrap().len(), 0);
    }

    #[test]
    fn test_clear_unpinned() {
        let db = make_db();
        let id1 = db
            .insert_item(&ClipboardItem::new_text("keep".to_string()))
            .unwrap();
        db.set_pinned(id1, true).unwrap();
        db.insert_item(&ClipboardItem::new_text("remove".to_string()))
            .unwrap();

        let cleared = db.clear_unpinned().unwrap();
        assert_eq!(cleared, 1);

        let items = db.get_recent(10).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].content, "keep");
    }

    #[test]
    fn test_enforce_max_items() {
        let db = make_db();
        for i in 0..10 {
            db.insert_item(&ClipboardItem::new_text(format!("item {i}")))
                .unwrap();
        }

        let deleted = db.enforce_max_items(5).unwrap();
        assert_eq!(deleted, 5);
        assert_eq!(db.get_recent(100).unwrap().len(), 5);
    }

    #[test]
    fn test_stats() {
        let db = make_db();
        db.insert_item(&ClipboardItem::new_text("hello".to_string()))
            .unwrap();
        let id = db
            .insert_item(&ClipboardItem::new_text("world".to_string()))
            .unwrap();
        db.set_pinned(id, true).unwrap();

        let stats = db.get_stats().unwrap();
        assert_eq!(stats.total_items, 2);
        assert_eq!(stats.pinned_items, 1);
        assert!(stats.total_size_bytes > 0);
    }

    #[test]
    fn test_multiple_items_ordering() {
        let db = make_db();
        db.insert_item(&ClipboardItem::new_text("first".to_string()))
            .unwrap();
        db.insert_item(&ClipboardItem::new_text("second".to_string()))
            .unwrap();
        db.insert_item(&ClipboardItem::new_text("third".to_string()))
            .unwrap();

        let items = db.get_recent(10).unwrap();
        assert_eq!(items.len(), 3);
        assert_eq!(items[0].content, "third");
    }
}
