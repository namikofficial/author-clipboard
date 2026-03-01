//! Database operations using `SQLite`

use crate::types::{ClipboardItem, ContentType, DbStats};
use rusqlite::{Connection, Result as SqlResult};
use std::path::Path;

/// SQLite-backed clipboard history database.
pub struct Database {
    conn: Connection,
}

impl Database {
    /// Open or create a database at the given path, running migrations.
    pub fn open(path: &Path) -> SqlResult<Self> {
        let conn = Connection::open(path)?;
        let db = Self { conn };
        db.init_schema()?;
        db.migrate()?;
        Ok(db)
    }

    /// Create an in-memory database (useful for testing)
    pub fn open_in_memory() -> SqlResult<Self> {
        let conn = Connection::open_in_memory()?;
        let db = Self { conn };
        db.init_schema()?;
        db.migrate()?;
        Ok(db)
    }

    fn init_schema(&self) -> SqlResult<()> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS clipboard_items (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                content_hash INTEGER NOT NULL,
                content TEXT NOT NULL,
                mime_type TEXT NOT NULL,
                content_type TEXT NOT NULL DEFAULT 'text',
                timestamp TEXT NOT NULL,
                pinned INTEGER NOT NULL DEFAULT 0,
                source_app TEXT,
                sensitive INTEGER NOT NULL DEFAULT 0
            );
            CREATE INDEX IF NOT EXISTS idx_timestamp ON clipboard_items(timestamp DESC);
            CREATE INDEX IF NOT EXISTS idx_content_hash ON clipboard_items(content_hash);
            CREATE INDEX IF NOT EXISTS idx_pinned ON clipboard_items(pinned);
            CREATE INDEX IF NOT EXISTS idx_content_type ON clipboard_items(content_type);

            CREATE TABLE IF NOT EXISTS recently_used (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                category TEXT NOT NULL,
                content TEXT NOT NULL,
                used_at TEXT NOT NULL,
                use_count INTEGER NOT NULL DEFAULT 1,
                UNIQUE(category, content)
            );
            CREATE INDEX IF NOT EXISTS idx_recently_category ON recently_used(category, used_at DESC);

            CREATE TABLE IF NOT EXISTS schema_version (
                version INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS audit_log (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                event_kind TEXT NOT NULL,
                details TEXT,
                timestamp TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_audit_timestamp ON audit_log(timestamp DESC);",
        )?;
        Ok(())
    }

    /// Run versioned schema migrations for existing databases.
    fn migrate(&self) -> SqlResult<()> {
        let version = self.get_schema_version();

        if version < 1 {
            // v1: Add content_type column (legacy migration)
            let has_content_type = self
                .conn
                .prepare("SELECT content_type FROM clipboard_items LIMIT 0")
                .is_ok();
            if !has_content_type {
                self.conn.execute_batch(
                    "ALTER TABLE clipboard_items ADD COLUMN content_type TEXT NOT NULL DEFAULT 'text';
                     CREATE INDEX IF NOT EXISTS idx_content_type ON clipboard_items(content_type);",
                )?;
            }
            self.set_schema_version(1)?;
        }

        if version < 2 {
            // v2: Add sensitive column
            let has_sensitive = self
                .conn
                .prepare("SELECT sensitive FROM clipboard_items LIMIT 0")
                .is_ok();
            if !has_sensitive {
                self.conn.execute_batch(
                    "ALTER TABLE clipboard_items ADD COLUMN sensitive INTEGER NOT NULL DEFAULT 0;",
                )?;
            }
            self.set_schema_version(2)?;
        }

        if version < 3 {
            // v3: Add plain_text column for HTML search indexing
            let has_plain_text = self
                .conn
                .prepare("SELECT plain_text FROM clipboard_items LIMIT 0")
                .is_ok();
            if !has_plain_text {
                self.conn
                    .execute_batch("ALTER TABLE clipboard_items ADD COLUMN plain_text TEXT;")?;
            }
            self.set_schema_version(3)?;
        }

        Ok(())
    }

    fn get_schema_version(&self) -> i64 {
        let result = self.conn.query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_version",
            [],
            |row| row.get(0),
        );
        result.unwrap_or_default()
    }

    fn set_schema_version(&self, version: i64) -> SqlResult<()> {
        self.conn.execute("DELETE FROM schema_version", [])?;
        self.conn.execute(
            "INSERT INTO schema_version (version) VALUES (?1)",
            [version],
        )?;
        Ok(())
    }

    // ── Insert / Dedup ────────────────────────────────────────────────

    /// Insert a new item. Returns the row id.
    pub fn insert_item(&self, item: &ClipboardItem) -> SqlResult<i64> {
        self.conn.execute(
            "INSERT INTO clipboard_items
                (content_hash, content, mime_type, content_type, timestamp, pinned, source_app, sensitive, plain_text)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            (
                item.content_hash.cast_signed(),
                &item.content,
                &item.mime_type,
                item.content_type.as_str(),
                item.timestamp.to_rfc3339(),
                i32::from(item.pinned),
                &item.source_app,
                i32::from(item.sensitive),
                &item.plain_text,
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
            "SELECT id, content_hash, content, mime_type, content_type, timestamp, pinned, source_app, sensitive, plain_text
             FROM clipboard_items
             ORDER BY pinned DESC, timestamp DESC
             LIMIT ?1",
        )?;
        Self::collect_items(&mut stmt, [limit])
    }

    /// Search items by content substring (case-insensitive). Only searches text items.
    pub fn search(&self, query: &str, limit: usize) -> SqlResult<Vec<ClipboardItem>> {
        let pattern = format!("%{query}%");
        let mut stmt = self.conn.prepare(
            "SELECT id, content_hash, content, mime_type, content_type, timestamp, pinned, source_app, sensitive, plain_text
             FROM clipboard_items
             WHERE (content LIKE ?1 OR plain_text LIKE ?1)
             ORDER BY pinned DESC, timestamp DESC
             LIMIT ?2",
        )?;
        Self::collect_items(&mut stmt, (&pattern as &dyn rusqlite::ToSql, &limit))
    }

    /// Get a single item by id.
    pub fn get_by_id(&self, id: i64) -> SqlResult<Option<ClipboardItem>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, content_hash, content, mime_type, content_type, timestamp, pinned, source_app, sensitive, plain_text
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

    /// Delete all non-pinned sensitive items (used on screen lock).
    pub fn clear_sensitive(&self) -> SqlResult<usize> {
        let affected = self.conn.execute(
            "DELETE FROM clipboard_items WHERE pinned = 0 AND sensitive = 1",
            [],
        )?;
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

    // ── Recently Used ─────────────────────────────────────────────────

    /// Record that an emoji/symbol/kaomoji was used (upsert).
    pub fn record_usage(&self, category: &str, content: &str) -> SqlResult<()> {
        self.conn.execute(
            "INSERT INTO recently_used (category, content, used_at, use_count)
             VALUES (?1, ?2, ?3, 1)
             ON CONFLICT(category, content) DO UPDATE SET
                used_at = ?3,
                use_count = use_count + 1",
            rusqlite::params![category, content, chrono::Utc::now().to_rfc3339()],
        )?;
        Ok(())
    }

    /// Get recently used items for a category, ordered by most recent.
    pub fn get_recently_used(&self, category: &str, limit: usize) -> SqlResult<Vec<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT content FROM recently_used
             WHERE category = ?1
             ORDER BY used_at DESC
             LIMIT ?2",
        )?;
        let items = stmt
            .query_map(rusqlite::params![category, limit], |row| row.get(0))?
            .collect::<SqlResult<Vec<String>>>()?;
        Ok(items)
    }

    /// Get frequently used items for a category, ordered by use count.
    pub fn get_frequently_used(&self, category: &str, limit: usize) -> SqlResult<Vec<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT content FROM recently_used
             WHERE category = ?1
             ORDER BY use_count DESC, used_at DESC
             LIMIT ?2",
        )?;
        let items = stmt
            .query_map(rusqlite::params![category, limit], |row| row.get(0))?
            .collect::<SqlResult<Vec<String>>>()?;
        Ok(items)
    }

    // ── Audit Log ─────────────────────────────────────────────────────

    /// Record a security audit event.
    pub fn log_audit_event(
        &self,
        kind: &crate::types::AuditEventKind,
        details: Option<&str>,
    ) -> SqlResult<()> {
        self.conn.execute(
            "INSERT INTO audit_log (event_kind, details, timestamp) VALUES (?1, ?2, ?3)",
            rusqlite::params![kind.as_str(), details, chrono::Utc::now().to_rfc3339()],
        )?;
        Ok(())
    }

    /// Get recent audit events.
    pub fn get_audit_log(&self, limit: usize) -> SqlResult<Vec<crate::types::AuditEvent>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, event_kind, details, timestamp FROM audit_log ORDER BY timestamp DESC LIMIT ?1",
        )?;
        let events = stmt
            .query_map([limit], |row| {
                Ok(crate::types::AuditEvent {
                    id: row.get(0)?,
                    event_kind: row.get(1)?,
                    details: row.get(2)?,
                    timestamp: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(3)?)
                        .map_or_else(|_| chrono::Utc::now(), |dt| dt.with_timezone(&chrono::Utc)),
                })
            })?
            .collect::<SqlResult<Vec<_>>>()?;
        Ok(events)
    }

    /// Clear old audit log entries (keep last N).
    pub fn trim_audit_log(&self, keep: usize) -> SqlResult<usize> {
        let affected = self.conn.execute(
            "DELETE FROM audit_log WHERE id NOT IN (SELECT id FROM audit_log ORDER BY timestamp DESC LIMIT ?1)",
            [keep],
        )?;
        Ok(affected)
    }

    // ── Export / Import ───────────────────────────────────────────────

    /// Export all clipboard items as JSON string.
    pub fn export_items(&self) -> SqlResult<String> {
        let items = self.get_recent(i32::MAX as usize)?;
        let json = serde_json::to_string_pretty(&items)
            .unwrap_or_else(|e| format!("{{\"error\": \"{e}\"}}"));
        Ok(json)
    }

    /// Import clipboard items from JSON string. Returns count of imported items.
    pub fn import_items(&self, json: &str) -> Result<usize, String> {
        let items: Vec<crate::types::ClipboardItem> =
            serde_json::from_str(json).map_err(|e| format!("Invalid JSON: {e}"))?;

        let mut count = 0;
        for item in &items {
            match self.insert_or_bump(item) {
                Ok(_) => count += 1,
                Err(e) => {
                    tracing::warn!("Failed to import item: {e}");
                }
            }
        }
        Ok(count)
    }

    // ── Helpers ────────────────────────────────────────────────────────

    fn row_to_item(row: &rusqlite::Row<'_>) -> SqlResult<ClipboardItem> {
        Ok(ClipboardItem {
            id: row.get(0)?,
            content_hash: row.get::<_, i64>(1)?.cast_unsigned(),
            content: row.get(2)?,
            mime_type: row.get(3)?,
            content_type: row
                .get::<_, String>(4)?
                .parse()
                .unwrap_or(ContentType::Text),
            timestamp: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(5)?)
                .map_or_else(|_| chrono::Utc::now(), |dt| dt.with_timezone(&chrono::Utc)),
            pinned: row.get::<_, i32>(6)? != 0,
            source_app: row.get(7)?,
            sensitive: row.get::<_, i32>(8).unwrap_or(0) != 0,
            plain_text: row.get(9).ok(),
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

    #[test]
    fn test_recently_used() {
        let db = make_db();
        db.record_usage("emoji", "😀").unwrap();
        db.record_usage("emoji", "😂").unwrap();
        db.record_usage("emoji", "😀").unwrap(); // bump count
        db.record_usage("symbol", "→").unwrap();

        let recent = db.get_recently_used("emoji", 10).unwrap();
        assert_eq!(recent.len(), 2);
        assert_eq!(recent[0], "😀"); // most recently used

        let frequent = db.get_frequently_used("emoji", 10).unwrap();
        assert_eq!(frequent[0], "😀"); // most frequently used (count=2)

        let symbols = db.get_recently_used("symbol", 10).unwrap();
        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0], "→");
    }

    #[test]
    fn test_clear_sensitive() {
        let db = make_db();
        // Regular item
        db.insert_item(&ClipboardItem::new_text("normal text".to_string()))
            .unwrap();
        // Sensitive item
        let mut sensitive = ClipboardItem::new_text("not actually sensitive".to_string());
        sensitive.sensitive = true;
        db.insert_item(&sensitive).unwrap();
        // Pinned sensitive item (should NOT be cleared)
        let mut pinned_sensitive = ClipboardItem::new_text("pinned secret".to_string());
        pinned_sensitive.sensitive = true;
        let pinned_id = db.insert_item(&pinned_sensitive).unwrap();
        db.set_pinned(pinned_id, true).unwrap();

        assert_eq!(db.get_recent(10).unwrap().len(), 3);

        let cleared = db.clear_sensitive().unwrap();
        assert_eq!(cleared, 1); // Only the unpinned sensitive item

        let remaining = db.get_recent(10).unwrap();
        assert_eq!(remaining.len(), 2); // normal + pinned sensitive
    }

    #[test]
    fn test_audit_log() {
        use crate::types::AuditEventKind;
        let db = make_db();
        db.log_audit_event(&AuditEventKind::IncognitoToggled, Some("enabled"))
            .unwrap();
        db.log_audit_event(&AuditEventKind::HistoryCleared, None)
            .unwrap();

        let events = db.get_audit_log(10).unwrap();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].event_kind, "history_cleared");
        assert_eq!(events[1].event_kind, "incognito_toggled");
    }

    #[test]
    fn test_trim_audit_log() {
        use crate::types::AuditEventKind;
        let db = make_db();
        for _ in 0..10 {
            db.log_audit_event(&AuditEventKind::ItemDeleted, None)
                .unwrap();
        }
        let trimmed = db.trim_audit_log(5).unwrap();
        assert_eq!(trimmed, 5);
        assert_eq!(db.get_audit_log(100).unwrap().len(), 5);
    }

    #[test]
    fn test_schema_version() {
        let db = make_db();
        // After init, version should be 3 (latest)
        let version = db.get_schema_version();
        assert_eq!(version, 3);
    }

    #[test]
    fn test_html_item() {
        let db = make_db();
        let item = ClipboardItem::new_html("<b>Hello</b>".to_string(), "Hello".to_string());
        let id = db.insert_item(&item).unwrap();
        let stored = db.get_by_id(id).unwrap().unwrap();
        assert_eq!(stored.content_type, ContentType::Html);
        assert_eq!(stored.plain_text, Some("Hello".to_string()));
        assert_eq!(stored.mime_type, "text/html");
    }

    #[test]
    fn test_files_item() {
        let db = make_db();
        let item = ClipboardItem::new_files(
            "file:///home/user/doc.pdf\nfile:///home/user/img.png".to_string(),
        );
        let id = db.insert_item(&item).unwrap();
        let stored = db.get_by_id(id).unwrap().unwrap();
        assert_eq!(stored.content_type, ContentType::Files);
        assert_eq!(stored.mime_type, "text/uri-list");
        let names = stored.file_names();
        assert_eq!(names, vec!["doc.pdf", "img.png"]);
    }

    #[test]
    fn test_search_html_plain_text() {
        let db = make_db();
        let item = ClipboardItem::new_html(
            "<p>Some HTML content</p>".to_string(),
            "Some HTML content".to_string(),
        );
        db.insert_item(&item).unwrap();

        // Should find via plain_text search
        let results = db.search("HTML content", 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].content_type, ContentType::Html);
    }

    #[test]
    fn test_export_import() {
        let db = make_db();
        db.insert_item(&ClipboardItem::new_text("export test 1".to_string()))
            .unwrap();
        db.insert_item(&ClipboardItem::new_text("export test 2".to_string()))
            .unwrap();

        let json = db.export_items().unwrap();
        assert!(json.contains("export test 1"));
        assert!(json.contains("export test 2"));

        // Import into fresh db
        let db2 = make_db();
        let count = db2.import_items(&json).unwrap();
        assert_eq!(count, 2);

        let items = db2.get_recent(10).unwrap();
        assert_eq!(items.len(), 2);
    }
}
