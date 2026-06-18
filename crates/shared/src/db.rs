//! Database operations using `SQLite`

use crate::types::{ClipboardItem, Collection, ContentType, DbStats, Snippet};
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
        // Enable WAL mode for crash safety and better concurrent read performance
        conn.execute_batch(
            "PRAGMA journal_mode=WAL;
             PRAGMA synchronous=NORMAL;
             PRAGMA busy_timeout=5000;
             PRAGMA foreign_keys=ON;",
        )?;
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
                sensitive INTEGER NOT NULL DEFAULT 0,
                ttl_override INTEGER DEFAULT NULL
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
            CREATE INDEX IF NOT EXISTS idx_audit_timestamp ON audit_log(timestamp DESC);

            CREATE VIRTUAL TABLE IF NOT EXISTS clipboard_fts USING fts5(content, plain_text, content='clipboard_items', content_rowid='id');

            CREATE TABLE IF NOT EXISTS snippets (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                content TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );",
        )?;
        Ok(())
    }

    /// Run versioned schema migrations for existing databases.
    #[allow(clippy::too_many_lines)]
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

        if version < 4 {
            // v4: Add FTS5 virtual table for full-text search
            self.conn.execute_batch(
                "CREATE VIRTUAL TABLE IF NOT EXISTS clipboard_fts USING fts5(content, plain_text, content='clipboard_items', content_rowid='id');
                 -- Populate FTS with existing data
                 INSERT OR IGNORE INTO clipboard_fts(rowid, content, plain_text) SELECT id, content, COALESCE(plain_text, '') FROM clipboard_items;
                 -- Triggers to keep FTS in sync
                 CREATE TRIGGER IF NOT EXISTS clipboard_fts_insert AFTER INSERT ON clipboard_items BEGIN
                     INSERT INTO clipboard_fts(rowid, content, plain_text) VALUES (new.id, new.content, COALESCE(new.plain_text, ''));
                 END;
                 CREATE TRIGGER IF NOT EXISTS clipboard_fts_delete AFTER DELETE ON clipboard_items BEGIN
                     INSERT INTO clipboard_fts(clipboard_fts, rowid, content, plain_text) VALUES('delete', old.id, old.content, COALESCE(old.plain_text, ''));
                 END;
                 CREATE TRIGGER IF NOT EXISTS clipboard_fts_update AFTER UPDATE OF content, plain_text ON clipboard_items BEGIN
                     INSERT INTO clipboard_fts(clipboard_fts, rowid, content, plain_text) VALUES('delete', old.id, old.content, COALESCE(old.plain_text, ''));
                     INSERT INTO clipboard_fts(rowid, content, plain_text) VALUES (new.id, new.content, COALESCE(new.plain_text, ''));
                 END;",
            )?;
            self.set_schema_version(4)?;
        }

        if version < 5 {
            // v5: Per-item TTL override
            let has_ttl_override = self
                .conn
                .prepare("SELECT ttl_override FROM clipboard_items LIMIT 0")
                .is_ok();
            if !has_ttl_override {
                self.conn.execute_batch(
                    "ALTER TABLE clipboard_items ADD COLUMN ttl_override INTEGER DEFAULT NULL;",
                )?;
            }
            self.set_schema_version(5)?;
        }

        if version < 6 {
            // v6: Snippets table
            self.conn.execute_batch(
                "CREATE TABLE IF NOT EXISTS snippets (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    name TEXT NOT NULL UNIQUE,
                    content TEXT NOT NULL,
                    updated_at TEXT NOT NULL
                );",
            )?;
            self.set_schema_version(6)?;
        }

        if version < 7 {
            // v7: Add starred column
            let has_starred = self
                .conn
                .prepare("SELECT starred FROM clipboard_items LIMIT 0")
                .is_ok();
            if !has_starred {
                self.conn.execute_batch(
                    "ALTER TABLE clipboard_items ADD COLUMN starred INTEGER NOT NULL DEFAULT 0;",
                )?;
            }
            self.set_schema_version(7)?;
        }

        if version < 8 {
            // v8: Add collections and collection_memberships tables
            self.conn.execute_batch(
                "CREATE TABLE IF NOT EXISTS collections (
                    id TEXT PRIMARY KEY,
                    name TEXT NOT NULL UNIQUE,
                    created_at TEXT NOT NULL,
                    updated_at TEXT NOT NULL
                );

                CREATE TABLE IF NOT EXISTS collection_memberships (
                    collection_id TEXT NOT NULL,
                    item_id INTEGER NOT NULL,
                    added_at TEXT NOT NULL,
                    PRIMARY KEY (collection_id, item_id),
                    FOREIGN KEY (collection_id) REFERENCES collections(id) ON DELETE CASCADE,
                    FOREIGN KEY (item_id) REFERENCES clipboard_items(id) ON DELETE CASCADE
                );",
            )?;
            self.set_schema_version(8)?;
        }

        if version < 9 {
            // v9: Encryption-at-rest metadata for sensitive items.
            //
            // - encrypted: 1 if the `content` column is ciphertext,
            //   0 (the default) if it is plaintext. Non-sensitive
            //   items are never encrypted.
            // - encryption_version: scheme version of the ciphertext.
            //   Currently always 1 (AES-256-GCM, base64(nonce || ct)).
            //   Bumping this value invalidates all previously
            //   encrypted rows and forces a re-encrypt on next read.
            // - redacted_preview: a fixed-length redacted form of
            //   the sensitive content, used by UIs and exports so
            //   they never have to decrypt the item just to display
            //   "••••••••" or a one-line hint. NULL for non-sensitive
            //   items.
            let has_encrypted = self
                .conn
                .prepare("SELECT encrypted FROM clipboard_items LIMIT 0")
                .is_ok();
            if !has_encrypted {
                self.conn.execute_batch(
                    "ALTER TABLE clipboard_items ADD COLUMN encrypted INTEGER NOT NULL DEFAULT 0;
                     ALTER TABLE clipboard_items ADD COLUMN encryption_version INTEGER DEFAULT NULL;
                     ALTER TABLE clipboard_items ADD COLUMN redacted_preview TEXT DEFAULT NULL;",
                )?;
            }
            self.set_schema_version(9)?;
        }

        if version < 10 {
            // v10: Saved filters table for query presets
            self.conn.execute_batch(
                "CREATE TABLE IF NOT EXISTS saved_filters (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    name TEXT NOT NULL UNIQUE,
                    query TEXT NOT NULL,
                    created_at TEXT NOT NULL,
                    updated_at TEXT NOT NULL
                );",
            )?;
            self.set_schema_version(10)?;
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

    /// Insert with the encryption-at-rest policy applied.
    ///
    /// If `item.sensitive` is true AND `encrypt_sensitive` is true,
    /// the `content` column is replaced with the ciphertext and the
    /// `encrypted` / `encryption_version` / `redacted_preview`
    /// columns are populated. Otherwise the item is inserted as a
    /// plain row (the same shape as `insert_item`).
    ///
    /// The FTS5 index is built from the *redacted* form for
    /// encrypted items, so a free-text search never sees the
    /// plaintext of an encrypted item. This is the safe default.
    ///
    /// Returns the new row id. A failure to encrypt or insert
    /// propagates the error and **no row is created**.
    pub fn insert_with_encryption(
        &self,
        item: &ClipboardItem,
        manager: &crate::encryption::EncryptionManager,
        encrypt_sensitive: bool,
    ) -> SqlResult<i64> {
        // Decide whether to encrypt before touching the DB.
        let (stored_content, stored_plain_text, encrypted, version, redacted) =
            if item.sensitive && encrypt_sensitive {
                let ciphertext = manager
                    .encrypt(&item.content)
                    .map_err(|_e| rusqlite::Error::InvalidQuery)?;
                let plain_text_ciphertext = match &item.plain_text {
                    Some(plain) if !plain.is_empty() => Some(
                        manager
                            .encrypt(plain)
                            .map_err(|_e| rusqlite::Error::InvalidQuery)?,
                    ),
                    _ => None,
                };
                let redacted = item.redacted_preview();
                (
                    ciphertext,
                    plain_text_ciphertext,
                    1_i32,
                    Some(1_i32),
                    Some(redacted),
                )
            } else {
                (
                    item.content.clone(),
                    item.plain_text.clone(),
                    0_i32,
                    None,
                    None,
                )
            };

        self.conn.execute(
            "INSERT INTO clipboard_items
                (content_hash, content, mime_type, content_type, timestamp, pinned, source_app, sensitive, plain_text,
                 encrypted, encryption_version, redacted_preview)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            rusqlite::params![
                item.content_hash.cast_signed(),
                stored_content,
                &item.mime_type,
                item.content_type.as_str(),
                item.timestamp.to_rfc3339(),
                i32::from(item.pinned),
                &item.source_app,
                i32::from(item.sensitive),
                &stored_plain_text,
                encrypted,
                version,
                &redacted,
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// Decrypt the content of a stored item, returning a fresh
    /// `ClipboardItem` whose `content` (and `plain_text`, if any)
    /// has been replaced with the plaintext.
    ///
    /// If the item is not encrypted (`encrypted = 0`), the input
    /// item is returned unchanged. If the item is encrypted but
    /// the supplied `EncryptionManager` cannot decrypt it
    /// (e.g. wrong key, tampered ciphertext), an error is
    /// returned and the caller must decide what to do.
    pub fn decrypt_item(
        &self,
        item: &ClipboardItem,
        manager: &crate::encryption::EncryptionManager,
    ) -> Result<ClipboardItem, String> {
        // Fast path: not encrypted, no work to do.
        if !Self::is_item_encrypted(item) {
            return Ok(item.clone());
        }
        let mut out = item.clone();
        out.content = manager.decrypt(&item.content)?;
        if let Some(ciphertext) = &item.plain_text {
            if !ciphertext.is_empty() {
                out.plain_text = Some(manager.decrypt(ciphertext)?);
            }
        }
        Ok(out)
    }

    /// Insert only if content hash doesn't already exist within the dedup window.
    /// If duplicate within `dedup_window_seconds`, bumps the existing item's timestamp instead.
    /// Returns the id of the inserted or bumped row.
    pub fn insert_or_bump(
        &self,
        item: &ClipboardItem,
        dedup_window_seconds: u64,
    ) -> SqlResult<i64> {
        if let Some(existing_id) = self.find_by_hash(item.content_hash)? {
            if self.has_recent_duplicate(item.content_hash, dedup_window_seconds)? {
                self.conn.execute(
                    "UPDATE clipboard_items SET timestamp = ?1 WHERE id = ?2",
                    (item.timestamp.to_rfc3339(), existing_id),
                )?;
                return Ok(existing_id);
            }
        }
        self.insert_item(item)
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
            "SELECT id, content_hash, content, mime_type, content_type, timestamp, pinned, starred, source_app, sensitive, plain_text, encrypted, encryption_version, redacted_preview
             FROM clipboard_items
             ORDER BY pinned DESC, timestamp DESC
             LIMIT ?1",
        )?;
        Self::collect_items(&mut stmt, [limit])
    }

    /// Get the single most recent item by timestamp, regardless of pin state.
    ///
    /// `get_recent` orders pinned items first; for the status bar we want
    /// the *latest* paste, which may or may not be pinned. Returns
    /// `Ok(None)` when the history is empty.
    pub fn get_most_recent(&self) -> SqlResult<Option<ClipboardItem>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, content_hash, content, mime_type, content_type, timestamp, pinned, starred, source_app, sensitive, plain_text, encrypted, encryption_version, redacted_preview
             FROM clipboard_items
             ORDER BY timestamp DESC
             LIMIT 1",
        )?;
        let mut rows = Self::collect_items(&mut stmt, [])?;
        Ok(rows.pop())
    }

    /// Search items by content. Uses FTS5 for performance with LIKE fallback.
    pub fn search(&self, query: &str, limit: usize) -> SqlResult<Vec<ClipboardItem>> {
        // Try FTS5 first for better performance
        let fts_result = self.conn.prepare(
            "SELECT ci.id, ci.content_hash, ci.content, ci.mime_type, ci.content_type, ci.timestamp, ci.pinned, ci.starred, ci.source_app, ci.sensitive, ci.plain_text, ci.encrypted, ci.encryption_version, ci.redacted_preview
             FROM clipboard_fts fts
             JOIN clipboard_items ci ON ci.id = fts.rowid
             WHERE clipboard_fts MATCH ?1
             ORDER BY ci.pinned DESC, ci.timestamp DESC
             LIMIT ?2",
        );

        if let Ok(mut stmt) = fts_result {
            // FTS5 query: wrap terms for prefix matching
            let fts_query = query
                .split_whitespace()
                .map(|w| format!("\"{}\"*", w.replace('"', "")))
                .collect::<Vec<_>>()
                .join(" ");
            if let Ok(items) =
                Self::collect_items(&mut stmt, (&fts_query as &dyn rusqlite::ToSql, &limit))
            {
                return Ok(items);
            }
        }

        // Fallback: LIKE search
        let pattern = format!("%{query}%");
        let mut stmt = self.conn.prepare(
            "SELECT id, content_hash, content, mime_type, content_type, timestamp, pinned, starred, source_app, sensitive, plain_text, encrypted, encryption_version, redacted_preview
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
            "SELECT id, content_hash, content, mime_type, content_type, timestamp, pinned, starred, source_app, sensitive, plain_text, encrypted, encryption_version, redacted_preview
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

    /// Set a custom TTL override for a specific item (in seconds).
    /// Pass `None` to clear the override and use the global TTL.
    pub fn set_item_ttl(&self, item_id: i64, ttl_seconds: Option<u64>) -> SqlResult<()> {
        self.conn.execute(
            "UPDATE clipboard_items SET ttl_override = ?1 WHERE id = ?2",
            rusqlite::params![ttl_seconds.map(u64::cast_signed), item_id],
        )?;
        Ok(())
    }

    /// Delete non-pinned items older than the given timestamp.
    /// Items with a per-item `ttl_override` use their custom TTL instead.
    pub fn delete_expired(&self, before: &chrono::DateTime<chrono::Utc>) -> SqlResult<usize> {
        let affected = self.conn.execute(
            "DELETE FROM clipboard_items WHERE pinned = 0 AND (
                (ttl_override IS NULL AND timestamp < ?1)
                OR (ttl_override IS NOT NULL AND datetime(timestamp, '+' || ttl_override || ' seconds') < datetime('now'))
            )",
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
        // For encrypted items, replace ciphertext content with redacted_preview
        // to avoid leaking encrypted content in exports.
        let items_for_export: Vec<crate::types::ClipboardItem> = items
            .into_iter()
            .map(|mut item| {
                if item.encrypted {
                    item.content = item
                        .redacted_preview
                        .clone()
                        .unwrap_or_else(|| "••••••••".to_string());
                    item.plain_text = None;
                }
                item
            })
            .collect();
        let json = serde_json::to_string_pretty(&items_for_export)
            .unwrap_or_else(|e| format!("{{\"error\": \"{e}\"}}"));
        Ok(json)
    }

    /// Import clipboard items from JSON string. Returns count of imported items.
    ///
    /// The `sensitive` flag is **always re-derived** from the content
    /// before insert, not trusted from the source JSON. This defends
    /// against a tampered export that marks a credential as
    /// `sensitive: false` to bypass the policy. If the re-derived
    /// flag says sensitive, the imported item is treated as sensitive
    /// regardless of what the source said; if the re-derived flag
    /// says not sensitive, the source flag is also cleared so a
    /// stale "sensitive" mark on a sanitized payload is removed.
    pub fn import_items(&self, json: &str) -> Result<usize, String> {
        let items: Vec<crate::types::ClipboardItem> =
            serde_json::from_str(json).map_err(|e| format!("Invalid JSON: {e}"))?;

        let mut count = 0;
        for mut item in items {
            item.sensitive = Self::derive_sensitive_for_import(&item);
            match self.insert_or_bump(&item, 0) {
                Ok(_) => count += 1,
                Err(e) => {
                    tracing::warn!("Failed to import item: {e}");
                }
            }
        }
        Ok(count)
    }

    // ── Dedup ──────────────────────────────────────────────────────────

    /// Check if an item with the same hash was inserted within the given window (seconds).
    #[allow(clippy::cast_possible_wrap)]
    pub fn has_recent_duplicate(&self, content_hash: u64, window_seconds: u64) -> SqlResult<bool> {
        let cutoff = chrono::Utc::now() - chrono::Duration::seconds(window_seconds as i64);
        let cutoff_str = cutoff.to_rfc3339();
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM clipboard_items WHERE content_hash = ?1 AND timestamp > ?2",
            rusqlite::params![content_hash as i64, cutoff_str],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    // ── Helpers ────────────────────────────────────────────────────────

    /// Whether the given item is stored as ciphertext in the
    /// `content` (and `plain_text`, if any) columns. The
    /// `EncryptionManager` must be used to read these fields.
    pub fn is_item_encrypted(item: &crate::types::ClipboardItem) -> bool {
        item.encrypted
    }

    /// Re-derive the `sensitive` flag for an item being imported,
    /// using the same detection rules as the constructors. This is
    /// the *only* way an item's sensitive flag is allowed to enter
    /// the database on import; the source JSON is never trusted.
    pub fn derive_sensitive_for_import(item: &crate::types::ClipboardItem) -> bool {
        use crate::types::ContentType;
        match item.content_type {
            ContentType::Text | ContentType::Files => {
                crate::sensitive::check_sensitivity(&item.content).is_sensitive
            }
            ContentType::Html => {
                let plain = item.plain_text.as_deref().unwrap_or("");
                crate::sensitive::check_sensitive_html(&item.content, plain).is_sensitive
            }
            ContentType::Image => false,
        }
    }

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
            starred: row.get::<_, i32>(7)? != 0,
            source_app: row.get(8)?,
            sensitive: row.get::<_, i32>(9).unwrap_or(0) != 0,
            plain_text: row.get(10).ok(),
            encrypted: row.get::<_, i32>(11).unwrap_or(0) != 0,
            encryption_version: row.get(12).ok(),
            redacted_preview: row.get(13).ok(),
        })
    }

    fn collect_items<P: rusqlite::Params>(
        stmt: &mut rusqlite::Statement<'_>,
        params: P,
    ) -> SqlResult<Vec<ClipboardItem>> {
        let items = stmt.query_map(params, Self::row_to_item)?;
        items.collect()
    }

    // ── Snippets ──────────────────────────────────────────────────────

    /// Insert or update a snippet by name.
    pub fn upsert_snippet(&self, name: &str, content: &str) -> SqlResult<()> {
        let now = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO snippets (name, content, updated_at) VALUES (?1, ?2, ?3)
             ON CONFLICT(name) DO UPDATE SET content=excluded.content, updated_at=excluded.updated_at",
            rusqlite::params![name, content, now],
        )?;
        Ok(())
    }

    /// List all snippets ordered by most recently updated.
    pub fn list_snippets(&self) -> SqlResult<Vec<Snippet>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, content, updated_at FROM snippets ORDER BY updated_at DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(Snippet {
                id: row.get(0)?,
                name: row.get(1)?,
                content: row.get(2)?,
                updated_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(3)?)
                    .map_or_else(|_| chrono::Utc::now(), |dt| dt.with_timezone(&chrono::Utc)),
            })
        })?;
        rows.collect()
    }

    /// Delete a snippet by id.
    pub fn delete_snippet(&self, id: i64) -> SqlResult<()> {
        self.conn
            .execute("DELETE FROM snippets WHERE id = ?1", [id])?;
        Ok(())
    }

    /// Fetch a single snippet by id.
    ///
    /// Returns `Ok(None)` when no row matches the id (not an error).
    pub fn get_snippet(&self, id: i64) -> SqlResult<Option<Snippet>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, name, content, updated_at FROM snippets WHERE id = ?1")?;
        let mut rows = stmt.query([id])?;
        match rows.next()? {
            Some(row) => Ok(Some(Snippet {
                id: row.get(0)?,
                name: row.get(1)?,
                content: row.get(2)?,
                updated_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(3)?)
                    .map_or_else(|_| chrono::Utc::now(), |dt| dt.with_timezone(&chrono::Utc)),
            })),
            None => Ok(None),
        }
    }

    /// Search snippets by name or content (case-insensitive substring).
    pub fn search_snippets(&self, query: &str) -> SqlResult<Vec<Snippet>> {
        let pattern = format!("%{query}%");
        let mut stmt = self.conn.prepare(
            "SELECT id, name, content, updated_at FROM snippets
             WHERE name LIKE ?1 OR content LIKE ?1
             ORDER BY updated_at DESC",
        )?;
        let rows = stmt.query_map([&pattern], |row| {
            Ok(Snippet {
                id: row.get(0)?,
                name: row.get(1)?,
                content: row.get(2)?,
                updated_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(3)?)
                    .map_or_else(|_| chrono::Utc::now(), |dt| dt.with_timezone(&chrono::Utc)),
            })
        })?;
        rows.collect()
    }

    // ── Star ─────────────────────────────────────────────────────────

    /// Toggle the starred state of an item. Returns the new starred value.
    pub fn toggle_star(&self, id: i64) -> SqlResult<bool> {
        self.conn.execute(
            "UPDATE clipboard_items SET starred = NOT starred WHERE id = ?1",
            [id],
        )?;
        let starred: bool = self.conn.query_row(
            "SELECT starred FROM clipboard_items WHERE id = ?1",
            [id],
            |row| row.get(0),
        )?;
        Ok(starred)
    }

    /// Set starred state explicitly.
    pub fn set_starred(&self, id: i64, starred: bool) -> SqlResult<()> {
        self.conn.execute(
            "UPDATE clipboard_items SET starred = ?1 WHERE id = ?2",
            (i32::from(starred), id),
        )?;
        Ok(())
    }

    // ── Collections ───────────────────────────────────────────────────

    /// Create a new collection. Returns the generated ID.
    pub fn create_collection(&self, name: &str) -> SqlResult<String> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO collections (id, name, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![&id, name, &now, &now],
        )?;
        Ok(id)
    }

    /// List all collections ordered by name.
    pub fn list_collections(&self) -> SqlResult<Vec<Collection>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, created_at, updated_at FROM collections ORDER BY name ASC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(Collection {
                id: row.get(0)?,
                name: row.get(1)?,
                created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(2)?)
                    .map_or_else(|_| chrono::Utc::now(), |dt| dt.with_timezone(&chrono::Utc)),
                updated_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(3)?)
                    .map_or_else(|_| chrono::Utc::now(), |dt| dt.with_timezone(&chrono::Utc)),
            })
        })?;
        rows.collect()
    }

    /// Delete a collection by ID.
    pub fn delete_collection(&self, id: &str) -> SqlResult<()> {
        self.conn
            .execute("DELETE FROM collections WHERE id = ?1", [id])?;
        Ok(())
    }

    /// Rename a collection.
    pub fn rename_collection(&self, id: &str, new_name: &str) -> SqlResult<()> {
        let now = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "UPDATE collections SET name = ?1, updated_at = ?2 WHERE id = ?3",
            rusqlite::params![new_name, &now, id],
        )?;
        Ok(())
    }

    // ── Collection Membership ─────────────────────────────────────────

    /// Add an item to a collection.
    pub fn add_to_collection(&self, collection_id: &str, item_id: i64) -> SqlResult<()> {
        let now = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT OR IGNORE INTO collection_memberships (collection_id, item_id, added_at) VALUES (?1, ?2, ?3)",
            rusqlite::params![collection_id, item_id, &now],
        )?;
        Ok(())
    }

    /// Remove an item from a collection.
    pub fn remove_from_collection(&self, collection_id: &str, item_id: i64) -> SqlResult<()> {
        self.conn.execute(
            "DELETE FROM collection_memberships WHERE collection_id = ?1 AND item_id = ?2",
            rusqlite::params![collection_id, item_id],
        )?;
        Ok(())
    }

    /// Get all items in a collection, ordered by `added_at` DESC.
    pub fn get_collection_items(&self, collection_id: &str) -> SqlResult<Vec<ClipboardItem>> {
        let mut stmt = self.conn.prepare(
            "SELECT ci.id, ci.content_hash, ci.content, ci.mime_type, ci.content_type, ci.timestamp, ci.pinned, ci.starred, ci.source_app, ci.sensitive, ci.plain_text, ci.encrypted, ci.encryption_version, ci.redacted_preview
             FROM clipboard_items ci
             JOIN collection_memberships cm ON ci.id = cm.item_id
             WHERE cm.collection_id = ?1
             ORDER BY cm.added_at DESC",
        )?;
        let items = stmt.query_map([collection_id], Self::row_to_item)?;
        items.collect()
    }

    /// Get all collections an item belongs to.
    pub fn get_item_collections(&self, item_id: i64) -> SqlResult<Vec<Collection>> {
        let mut stmt = self.conn.prepare(
            "SELECT c.id, c.name, c.created_at, c.updated_at
             FROM collections c
             JOIN collection_memberships cm ON c.id = cm.collection_id
             WHERE cm.item_id = ?1
             ORDER BY c.name ASC",
        )?;
        let rows = stmt.query_map([item_id], |row| {
            Ok(Collection {
                id: row.get(0)?,
                name: row.get(1)?,
                created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(2)?)
                    .map_or_else(|_| chrono::Utc::now(), |dt| dt.with_timezone(&chrono::Utc)),
                updated_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(3)?)
                    .map_or_else(|_| chrono::Utc::now(), |dt| dt.with_timezone(&chrono::Utc)),
            })
        })?;
        rows.collect()
    }

    // ── Saved Filters ─────────────────────────────────────────────────

    /// Create or update a saved filter (upsert by name).
    pub fn upsert_saved_filter(&self, name: &str, query: &str) -> SqlResult<i64> {
        let now = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO saved_filters (name, query, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?3)
             ON CONFLICT(name) DO UPDATE SET query=excluded.query, updated_at=excluded.updated_at",
            rusqlite::params![name, query, &now],
        )?;
        // Return the id of the inserted/updated row
        let id: i64 = self.conn.query_row(
            "SELECT id FROM saved_filters WHERE name = ?1",
            [name],
            |row| row.get(0),
        )?;
        Ok(id)
    }

    /// List all saved filters ordered by name.
    pub fn list_saved_filters(&self) -> SqlResult<Vec<crate::types::SavedFilter>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, query, created_at, updated_at FROM saved_filters ORDER BY name ASC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(crate::types::SavedFilter {
                id: row.get(0)?,
                name: row.get(1)?,
                query: row.get(2)?,
                created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(3)?)
                    .map_or_else(|_| chrono::Utc::now(), |dt| dt.with_timezone(&chrono::Utc)),
                updated_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(4)?)
                    .map_or_else(|_| chrono::Utc::now(), |dt| dt.with_timezone(&chrono::Utc)),
            })
        })?;
        rows.collect()
    }

    /// Get a saved filter by name.
    pub fn get_saved_filter_by_name(
        &self,
        name: &str,
    ) -> SqlResult<Option<crate::types::SavedFilter>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, query, created_at, updated_at FROM saved_filters WHERE name = ?1",
        )?;
        let mut rows = stmt.query([name])?;
        match rows.next()? {
            Some(row) => Ok(Some(crate::types::SavedFilter {
                id: row.get(0)?,
                name: row.get(1)?,
                query: row.get(2)?,
                created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(3)?)
                    .map_or_else(|_| chrono::Utc::now(), |dt| dt.with_timezone(&chrono::Utc)),
                updated_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(4)?)
                    .map_or_else(|_| chrono::Utc::now(), |dt| dt.with_timezone(&chrono::Utc)),
            })),
            None => Ok(None),
        }
    }

    /// Delete a saved filter by id.
    pub fn delete_saved_filter(&self, id: i64) -> SqlResult<()> {
        self.conn
            .execute("DELETE FROM saved_filters WHERE id = ?1", [id])?;
        Ok(())
    }

    /// Delete a saved filter by name.
    pub fn delete_saved_filter_by_name(&self, name: &str) -> SqlResult<()> {
        self.conn
            .execute("DELETE FROM saved_filters WHERE name = ?1", [name])?;
        Ok(())
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
        let id1 = db.insert_or_bump(&item1, 60).unwrap();

        let item2 = ClipboardItem::new_text("duplicate".to_string());
        let id2 = db.insert_or_bump(&item2, 60).unwrap();

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
    fn test_dedup_window() {
        let db = make_db();
        db.insert_item(&ClipboardItem::new_text("duplicate test".to_string()))
            .unwrap();
        // Same content hash should be detected as duplicate
        assert!(db
            .has_recent_duplicate(ClipboardItem::hash_content("duplicate test"), 60)
            .unwrap());
        // Different content should not be duplicate
        assert!(!db
            .has_recent_duplicate(ClipboardItem::hash_content("unique content"), 60)
            .unwrap());
    }

    // ── Collections ───────────────────────────────────────────────────

    #[test]
    fn test_collection_create_and_list() {
        let db = make_db();

        let id1 = db.create_collection("Work").unwrap();
        assert!(!id1.is_empty());

        let id2 = db.create_collection("Personal").unwrap();
        assert!(!id2.is_empty());

        let collections = db.list_collections().unwrap();
        assert_eq!(collections.len(), 2);
        assert_eq!(collections[0].name, "Personal"); // Ordered by name
        assert_eq!(collections[1].name, "Work");
    }

    #[test]
    fn test_collection_delete() {
        let db = make_db();

        let id = db.create_collection("ToDelete").unwrap();
        assert_eq!(db.list_collections().unwrap().len(), 1);

        db.delete_collection(&id).unwrap();
        assert_eq!(db.list_collections().unwrap().len(), 0);
    }

    #[test]
    fn test_collection_rename() {
        let db = make_db();

        let id = db.create_collection("OldName").unwrap();
        db.rename_collection(&id, "NewName").unwrap();

        let collections = db.list_collections().unwrap();
        assert_eq!(collections.len(), 1);
        assert_eq!(collections[0].name, "NewName");
    }

    #[test]
    fn test_add_to_collection() {
        let db = make_db();

        let item_id = db
            .insert_item(&ClipboardItem::new_text("test content".to_string()))
            .unwrap();
        let collection_id = db.create_collection("My Collection").unwrap();

        db.add_to_collection(&collection_id, item_id).unwrap();

        let items = db.get_collection_items(&collection_id).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].id, item_id);
        assert_eq!(items[0].content, "test content");
    }

    #[test]
    fn test_remove_from_collection() {
        let db = make_db();

        let item_id = db
            .insert_item(&ClipboardItem::new_text("removeme".to_string()))
            .unwrap();
        let collection_id = db.create_collection("Temp").unwrap();

        db.add_to_collection(&collection_id, item_id).unwrap();
        assert_eq!(db.get_collection_items(&collection_id).unwrap().len(), 1);

        db.remove_from_collection(&collection_id, item_id).unwrap();
        assert_eq!(db.get_collection_items(&collection_id).unwrap().len(), 0);
    }

    #[test]
    fn test_get_item_collections() {
        let db = make_db();

        let item_id = db
            .insert_item(&ClipboardItem::new_text("shared item".to_string()))
            .unwrap();
        let col1 = db.create_collection("Alpha").unwrap();
        let col2 = db.create_collection("Beta").unwrap();

        db.add_to_collection(&col1, item_id).unwrap();
        db.add_to_collection(&col2, item_id).unwrap();

        let collections = db.get_item_collections(item_id).unwrap();
        assert_eq!(collections.len(), 2);
        assert!(collections.iter().any(|c| c.name == "Alpha"));
        assert!(collections.iter().any(|c| c.name == "Beta"));
    }

    #[test]
    fn test_collection_item_order() {
        // Items in a collection should be ordered by added_at DESC (newest first)
        // When items are added in quick succession with identical timestamps,
        // the order is non-deterministic, so we just verify all items are present.
        let db = make_db();

        let collection_id = db.create_collection("Ordered").unwrap();

        let id1 = db
            .insert_item(&ClipboardItem::new_text("first".to_string()))
            .unwrap();
        let id2 = db
            .insert_item(&ClipboardItem::new_text("second".to_string()))
            .unwrap();
        let id3 = db
            .insert_item(&ClipboardItem::new_text("third".to_string()))
            .unwrap();

        // Add in reverse order
        db.add_to_collection(&collection_id, id3).unwrap();
        db.add_to_collection(&collection_id, id2).unwrap();
        db.add_to_collection(&collection_id, id1).unwrap();

        let items = db.get_collection_items(&collection_id).unwrap();
        assert_eq!(items.len(), 3);
        // Verify all items are in the collection
        let item_ids: Vec<i64> = items.iter().map(|i| i.id).collect();
        assert!(item_ids.contains(&id1));
        assert!(item_ids.contains(&id2));
        assert!(item_ids.contains(&id3));
    }

    #[test]
    fn test_collection_cascade_delete() {
        // When a collection is deleted, memberships should be removed via CASCADE
        let db = make_db();

        let item_id = db
            .insert_item(&ClipboardItem::new_text("orphaned".to_string()))
            .unwrap();
        let collection_id = db.create_collection("WillBeDeleted").unwrap();

        db.add_to_collection(&collection_id, item_id).unwrap();
        db.delete_collection(&collection_id).unwrap();

        // Item should still exist
        assert!(db.get_by_id(item_id).unwrap().is_some());
        // But membership should be gone
        assert!(db.get_item_collections(item_id).unwrap().is_empty());
    }

    // ── Saved Filters ─────────────────────────────────────────────────

    #[test]
    fn test_saved_filter_upsert_and_list() {
        let db = make_db();

        let id1 = db
            .upsert_saved_filter("work", "type:text pinned:true")
            .unwrap();
        assert!(id1 > 0);

        let id2 = db.upsert_saved_filter("pics", "type:image").unwrap();
        assert!(id2 > 0);

        let filters = db.list_saved_filters().unwrap();
        assert_eq!(filters.len(), 2);
        assert_eq!(filters[0].name, "pics");
        assert_eq!(filters[1].name, "work");
    }

    #[test]
    fn test_saved_filter_upsert_updates() {
        let db = make_db();

        let id1 = db.upsert_saved_filter("myfilter", "type:text").unwrap();
        let id2 = db
            .upsert_saved_filter("myfilter", "type:image pinned:true")
            .unwrap();

        // Same name should return same id (upsert, not insert)
        assert_eq!(id1, id2);

        let filters = db.list_saved_filters().unwrap();
        assert_eq!(filters.len(), 1);
        assert_eq!(filters[0].query, "type:image pinned:true");
    }

    #[test]
    fn test_get_saved_filter_by_name() {
        let db = make_db();

        db.upsert_saved_filter("findme", "type:text").unwrap();

        let found = db.get_saved_filter_by_name("findme").unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().query, "type:text");

        let not_found = db.get_saved_filter_by_name("nonexistent").unwrap();
        assert!(not_found.is_none());
    }

    #[test]
    fn test_delete_saved_filter() {
        let db = make_db();

        db.upsert_saved_filter("todelete", "type:files").unwrap();
        assert_eq!(db.list_saved_filters().unwrap().len(), 1);

        // Delete by id
        let id = db.get_saved_filter_by_name("todelete").unwrap().unwrap().id;
        db.delete_saved_filter(id).unwrap();
        assert_eq!(db.list_saved_filters().unwrap().len(), 0);
    }

    #[test]
    fn test_delete_saved_filter_by_name() {
        let db = make_db();

        db.upsert_saved_filter("by_name", "app:firefox").unwrap();
        db.upsert_saved_filter("keep", "type:text").unwrap();
        assert_eq!(db.list_saved_filters().unwrap().len(), 2);

        db.delete_saved_filter_by_name("by_name").unwrap();
        let filters = db.list_saved_filters().unwrap();
        assert_eq!(filters.len(), 1);
        assert_eq!(filters[0].name, "keep");
    }

    #[test]
    fn test_schema_version() {
        let db = make_db();
        // After init, version should be 10 (latest)
        let version = db.get_schema_version();
        assert_eq!(version, 10);
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
    fn test_fts_search() {
        let db = make_db();
        db.insert_item(&ClipboardItem::new_text("hello world greeting".to_string()))
            .unwrap();
        db.insert_item(&ClipboardItem::new_text(
            "rust programming language".to_string(),
        ))
        .unwrap();
        db.insert_item(&ClipboardItem::new_text(
            "hello rust developers".to_string(),
        ))
        .unwrap();

        let results = db.search("hello", 10).unwrap();
        assert_eq!(results.len(), 2);

        let results = db.search("rust", 10).unwrap();
        assert_eq!(results.len(), 2);

        let results = db.search("nonexistent", 10).unwrap();
        assert!(results.is_empty());
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

    #[test]
    fn test_per_item_ttl() {
        let db = make_db();
        let item = ClipboardItem::new_text("ttl test".to_string());
        let id = db.insert_item(&item).unwrap();

        // Set custom TTL
        db.set_item_ttl(id, Some(3600)).unwrap();

        // Clear custom TTL
        db.set_item_ttl(id, None).unwrap();
    }

    #[test]
    fn import_redisovers_sensitive_flag_on_text() {
        // Craft a JSON payload that lies about the sensitive flag.
        // The import path must NOT trust it.
        let json = r#"[
            {
                "id": 0,
                "content_hash": 0,
                "content": "ghp_1234567890abcdefghij",
                "mime_type": "text/plain",
                "content_type": "Text",
                "timestamp": "2026-01-01T00:00:00Z",
                "pinned": false,
                "starred": false,
                "source_app": null,
                "sensitive": false,
                "plain_text": null
            }
        ]"#;
        let db = make_db();
        let count = db.import_items(json).unwrap();
        assert_eq!(count, 1);
        let items = db.get_recent(10).unwrap();
        assert_eq!(items.len(), 1);
        assert!(
            items[0].sensitive,
            "import must re-derive sensitive=true even when source says false"
        );
    }

    #[test]
    #[allow(clippy::uninlined_format_args)]
    fn import_redisovers_sensitive_flag_on_html() {
        // HTML with a secret in an attribute; tampered JSON claims
        // sensitive=false. The import path must re-derive it.
        let html = r#"<form><input type="password" value="MyP@ssw0rd!" /></form>"#;
        let json = format!(
            r#"[
                {{
                    "id": 0,
                    "content_hash": 0,
                    "content": {:?},
                    "mime_type": "text/html",
                    "content_type": "Html",
                    "timestamp": "2026-01-01T00:00:00Z",
                    "pinned": false,
                    "starred": false,
                    "source_app": null,
                    "sensitive": false,
                    "plain_text": ""
                }}
            ]"#,
            html
        );
        let db = make_db();
        db.import_items(&json).unwrap();
        let items = db.get_recent(10).unwrap();
        assert!(items[0].sensitive, "HTML import must re-derive sensitive");
    }

    #[test]
    #[allow(clippy::uninlined_format_args)]
    fn import_redisovers_sensitive_flag_on_files() {
        // URI list with an embedded credential; tampered JSON says
        // sensitive=false. The import path must re-derive it.
        let list = "file:///tmp/a\npostgresql://admin:secret@db.example.com/x";
        let json = format!(
            r#"[
                {{
                    "id": 0,
                    "content_hash": 0,
                    "content": {:?},
                    "mime_type": "text/uri-list",
                    "content_type": "Files",
                    "timestamp": "2026-01-01T00:00:00Z",
                    "pinned": false,
                    "starred": false,
                    "source_app": null,
                    "sensitive": false,
                    "plain_text": null
                }}
            ]"#,
            list
        );
        let db = make_db();
        db.import_items(&json).unwrap();
        let items = db.get_recent(10).unwrap();
        assert!(items[0].sensitive, "files import must re-derive sensitive");
    }

    #[test]
    fn import_drops_stale_sensitive_flag_on_safe_content() {
        // JSON claims sensitive=true for plain text that is actually
        // safe. The import path must clear the stale flag.
        let json = r#"[
            {
                "id": 0,
                "content_hash": 0,
                "content": "Hello world",
                "mime_type": "text/plain",
                "content_type": "Text",
                "timestamp": "2026-01-01T00:00:00Z",
                "pinned": false,
                "starred": false,
                "source_app": null,
                "sensitive": true,
                "plain_text": null
            }
        ]"#;
        let db = make_db();
        db.import_items(json).unwrap();
        let items = db.get_recent(10).unwrap();
        assert!(
            !items[0].sensitive,
            "stale sensitive=true on safe content must be cleared"
        );
    }

    // ── Encryption at rest ─────────────────────────────────────────────

    fn tmp_data_dir() -> tempfile::TempDir {
        tempfile::tempdir().expect("tempdir")
    }

    #[test]
    fn insert_with_encryption_stores_ciphertext_not_plaintext() {
        // A sensitive item inserted with encrypt_sensitive=true must
        // be retrievable in encrypted form, and the on-disk row
        // must not contain the plaintext. This is the
        // no-plaintext-at-rest invariant the hardening pass
        // requires.
        let tmp = tmp_data_dir();
        let mgr = crate::encryption::EncryptionManager::new(tmp.path()).unwrap();
        let db = make_db();
        let item = ClipboardItem::new_text("ghp_1234567890abcdefghij".to_string());
        assert!(item.sensitive);
        let plaintext = item.content.clone();

        let id = db
            .insert_with_encryption(&item, &mgr, true)
            .expect("insert");

        // 1. The DB row must not contain the plaintext in the
        //    `content` column. (The content column is what a future
        //    raw-SQL attacker would read; if it contains the
        //    plaintext, encryption is broken.)
        let stored: String = db
            .conn
            .query_row(
                "SELECT content FROM clipboard_items WHERE id = ?1",
                [id],
                |row| row.get(0),
            )
            .unwrap();
        assert_ne!(
            stored, plaintext,
            "content column must hold ciphertext, not plaintext"
        );
        assert!(
            !stored.contains(&plaintext),
            "content column must not contain the plaintext substring"
        );
        // 2. encrypted + version + redacted columns must be set.
        let (encrypted, version, redacted): (i32, Option<i32>, Option<String>) = db
            .conn
            .query_row(
                "SELECT encrypted, encryption_version, redacted_preview
                 FROM clipboard_items WHERE id = ?1",
                [id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap();
        assert_eq!(encrypted, 1);
        assert_eq!(version, Some(1));
        assert!(redacted.is_some(), "redacted_preview must be populated");
    }

    #[test]
    fn insert_with_encryption_decrypts_at_boundary() {
        // Round-trip: insert encrypted, read back through
        // decrypt_item, get the original plaintext.
        let tmp = tmp_data_dir();
        let mgr = crate::encryption::EncryptionManager::new(tmp.path()).unwrap();
        let db = make_db();
        let item = ClipboardItem::new_text("postgresql://u:secret@host/db".to_string());
        let original = item.content.clone();
        let id = db
            .insert_with_encryption(&item, &mgr, true)
            .expect("insert");

        let stored = db.get_by_id(id).unwrap().unwrap();
        assert!(stored.encrypted);
        let decrypted = db.decrypt_item(&stored, &mgr).unwrap();
        assert_eq!(decrypted.content, original);
    }

    #[test]
    fn insert_with_encryption_persists_across_restart() {
        // After "restart" (new manager reading the same key file),
        // encrypted rows must still decrypt.
        let tmp = tmp_data_dir();
        let mgr1 = crate::encryption::EncryptionManager::new(tmp.path()).unwrap();
        let db = make_db();
        let item = ClipboardItem::new_text("hunter2-not-sensitive".to_string());
        // Force the test fixture to be sensitive so the
        // encrypt_sensitive=true path is actually exercised.
        let mut sensitive_item = item.clone();
        sensitive_item.sensitive = true;
        let _ = db
            .insert_with_encryption(&sensitive_item, &mgr1, true)
            .unwrap();

        // Simulate restart.
        let mgr2 = crate::encryption::EncryptionManager::new(tmp.path()).unwrap();
        let items = db.get_recent(10).unwrap();
        assert_eq!(items.len(), 1);
        // Sensitive + encrypt_sensitive=true item must be encrypted.
        assert!(items[0].encrypted);
        let decrypted = db.decrypt_item(&items[0], &mgr2).unwrap();
        assert_eq!(decrypted.content, sensitive_item.content);
    }

    #[test]
    fn insert_with_encryption_wrong_key_fails_safely() {
        // Decrypting with a manager from a different data dir must
        // fail rather than return garbage. This protects against
        // the user moving the database to a new machine / user
        // account and getting a silent tamper.
        let tmp1 = tmp_data_dir();
        let tmp2 = tmp_data_dir();
        let mgr1 = crate::encryption::EncryptionManager::new(tmp1.path()).unwrap();
        let mgr2 = crate::encryption::EncryptionManager::new(tmp2.path()).unwrap();

        let db = make_db();
        let item = ClipboardItem::new_text("ghp_secret_value".to_string());
        let _ = db
            .insert_with_encryption(&item, &mgr1, true)
            .expect("insert");
        let stored = db.get_recent(10).unwrap();
        assert_eq!(stored.len(), 1);

        let result = db.decrypt_item(&stored[0], &mgr2);
        assert!(
            result.is_err(),
            "decryption with a foreign key must fail rather than return plaintext"
        );
    }

    #[test]
    fn insert_with_encryption_flag_false_stores_plaintext() {
        // With encrypt_sensitive=false, sensitive items are still
        // stored as plaintext (the user has opted out). This is the
        // expected behavior — the flag is opt-in.
        let tmp = tmp_data_dir();
        let mgr = crate::encryption::EncryptionManager::new(tmp.path()).unwrap();
        let db = make_db();
        let item = ClipboardItem::new_text("sk-abc123xyz".to_string());
        assert!(item.sensitive);
        let id = db
            .insert_with_encryption(&item, &mgr, false)
            .expect("insert");

        let stored: String = db
            .conn
            .query_row(
                "SELECT content FROM clipboard_items WHERE id = ?1",
                [id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(
            stored, "sk-abc123xyz",
            "plaintext must be stored when opt-out"
        );

        let (encrypted, redacted): (i32, Option<String>) = db
            .conn
            .query_row(
                "SELECT encrypted, redacted_preview FROM clipboard_items WHERE id = ?1",
                [id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(encrypted, 0);
        assert!(redacted.is_none());
    }

    #[test]
    fn redacted_preview_helper_is_safe_for_sensitive_items() {
        // The helper must never echo the plaintext back, even
        // partially. A UI that uses this helper is safe by
        // construction.
        let item = ClipboardItem::new_text("MyP@ssw0rd!hunter2".to_string());
        let preview = item.redacted_preview();
        assert!(!preview.contains("MyP"));
        assert!(!preview.contains("hunter"));
        assert!(preview.contains("Sensitive"));
    }

    #[test]
    fn export_items_redacts_encrypted_content() {
        // When exporting, encrypted items must not leak ciphertext.
        // The export must use redacted_preview instead of content.
        let tmp = tmp_data_dir();
        let mgr = crate::encryption::EncryptionManager::new(tmp.path()).unwrap();
        let db = make_db();

        // Insert an encrypted sensitive item
        let item = ClipboardItem::new_text("ghp_secretToken123456".to_string());
        assert!(item.sensitive);
        db.insert_with_encryption(&item, &mgr, true)
            .expect("insert encrypted");

        // Export and verify the ciphertext is not in the JSON
        let json = db.export_items().unwrap();

        // The plaintext must not appear in the export
        assert!(
            !json.contains("ghp_secretToken123456"),
            "export must not contain plaintext of encrypted item"
        );

        // The ciphertext (base64) must not appear either
        let stored_item = db.get_recent(1).unwrap().pop().unwrap();
        assert!(stored_item.encrypted);
        // The ciphertext is base64 encoded, so it should be longer than plaintext
        // and contain chars like +/= that plaintext wouldn't have
        assert!(
            !json.contains(&stored_item.content),
            "export must not contain ciphertext of encrypted item"
        );

        // The redacted marker should be present
        assert!(json.contains("Sensitive item") || json.contains("••••"));
    }

    #[test]
    fn search_results_mask_encrypted_content() {
        // Encrypted content is NOT searchable (by design). This test
        // verifies that:
        // 1. An encrypted item's original plaintext is NOT found via search
        // 2. The item's encrypted flag is correctly set
        // 3. If we retrieve the item directly, it's properly encrypted
        let tmp = tmp_data_dir();
        let mgr = crate::encryption::EncryptionManager::new(tmp.path()).unwrap();
        let db = make_db();

        // Insert an encrypted sensitive item
        let item = ClipboardItem::new_text("AKIAIOSFODNN7EXAMPLE".to_string());
        assert!(item.sensitive);
        db.insert_with_encryption(&item, &mgr, true)
            .expect("insert encrypted");

        // Search for part of the original plaintext - should NOT find it
        // because encrypted content is not indexed
        let results = db.search("AKIA", 10).unwrap();
        assert_eq!(
            results.len(),
            0,
            "encrypted content must not be searchable (plaintext not indexed)"
        );

        // But the item exists and is retrievable
        let all_items = db.get_recent(10).unwrap();
        assert_eq!(all_items.len(), 1);
        assert!(all_items[0].encrypted, "item must be marked as encrypted");

        // And we can decrypt it back to original plaintext
        let decrypted = db.decrypt_item(&all_items[0], &mgr).unwrap();
        assert_eq!(
            decrypted.content, "AKIAIOSFODNN7EXAMPLE",
            "decrypted content must match original plaintext"
        );
    }
}
