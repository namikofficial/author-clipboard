# Domain Model: Collections, Pinning, and Starring

> Data structures, state, and relationships for collections and three-tier organization.

---

## Data Structures

### New Types

```rust
// In shared/src/types.rs

/// A named collection of clipboard items
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Collection {
    pub id: String,           // UUID
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub item_count: usize,    // Denormalized for performance
}

/// A membership of an item in a collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionMembership {
    pub collection_id: String,
    pub item_id: i64,
    pub added_at: DateTime<Utc>,
}
```

### Changes to ClipboardItem

```rust
// Add starred field to existing ClipboardItem
pub struct ClipboardItem {
    pub id: i64,
    pub content_hash: u64,
    pub content: String,
    pub mime_type: String,
    pub content_type: ContentType,
    pub timestamp: DateTime<Utc>,
    pub pinned: bool,          // Existing field
    pub starred: bool,         // NEW: Star flag
    pub source_app: Option<String>,
    pub sensitive: bool,
    pub plain_text: Option<String>,
    // ... existing fields ...
}
```

---

## Database Changes

### New Tables

```sql
-- Collections
CREATE TABLE collections (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Collection memberships (many-to-many)
CREATE TABLE collection_memberships (
    collection_id TEXT NOT NULL,
    item_id INTEGER NOT NULL,
    added_at TEXT NOT NULL,
    PRIMARY KEY (collection_id, item_id),
    FOREIGN KEY (collection_id) REFERENCES collections(id) ON DELETE CASCADE,
    FOREIGN KEY (item_id) REFERENCES clipboard_items(id) ON DELETE CASCADE
);

-- Index for fast membership lookups
CREATE INDEX idx_collection_memberships_collection ON collection_memberships(collection_id);
CREATE INDEX idx_collection_memberships_item ON collection_memberships(item_id);
```

### Schema Migration

```rust
// shared/src/db/migrations.rs
const MIGRATION_008: Migration = Migration {
    version: 8,
    description: "add starred field and collections",
    operations: &[
        // Add starred column to clipboard_items
        MigrationOp::RunSql("ALTER TABLE clipboard_items ADD COLUMN starred INTEGER NOT NULL DEFAULT 0"),
        MigrationOp::CreateTable {
            name: "collections",
            columns: &[
                ("id", "TEXT NOT NULL PRIMARY KEY"),
                ("name", "TEXT NOT NULL UNIQUE"),
                ("created_at", "TEXT NOT NULL"),
                ("updated_at", "TEXT NOT NULL"),
            ],
        },
        MigrationOp::CreateTable {
            name: "collection_memberships",
            columns: &[
                ("collection_id", "TEXT NOT NULL"),
                ("item_id", "INTEGER NOT NULL"),
                ("added_at", "TEXT NOT NULL"),
                ("PRIMARY KEY", "(collection_id, item_id)"),
            ],
        },
        MigrationOp::CreateIndex {
            name: "idx_collection_memberships_collection",
            table: "collection_memberships",
            columns: &["collection_id"],
        },
        MigrationOp::CreateIndex {
            name: "idx_collection_memberships_item",
            table: "collection_memberships",
            columns: &["item_id"],
        },
    ],
};
```

---

## Query Patterns

### Get Recent Items (with starred ranking)

```sql
SELECT *,
    CASE WHEN starred = 1 THEN 1 ELSE 0 END as star_rank
FROM clipboard_items
ORDER BY pinned DESC, starred DESC, timestamp DESC
LIMIT 50;
```

### Get Items in Collection

```sql
SELECT ci.*
FROM clipboard_items ci
JOIN collection_memberships cm ON ci.id = cm.item_id
WHERE cm.collection_id = ?
ORDER BY cm.added_at DESC;
```

### Get Collections for Item

```sql
SELECT c.*
FROM collections c
JOIN collection_memberships cm ON c.id = cm.collection_id
WHERE cm.item_id = ?;
```

### Get All Collections with Item Counts

```sql
SELECT c.*, COUNT(cm.item_id) as item_count
FROM collections c
LEFT JOIN collection_memberships cm ON c.id = cm.collection_id
GROUP BY c.id
ORDER BY c.name;
```

---

## IPC Protocol Changes

### New IPC Commands

```rust
// In shared/src/ipc.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "cmd", content = "args")]
pub enum IpcCommand {
    // ... existing commands ...

    // Pin/Unpin
    Pin { id: i64 },
    Unpin { id: i64 },

    // Star/Unstar
    Star { id: i64 },
    Unstar { id: i64 },

    // Collections
    ListCollections,
    CreateCollection { name: String },
    RenameCollection { id: String, new_name: String },
    DeleteCollection { id: String },
    GetCollectionItems { id: String, limit: Option<usize>, offset: Option<usize> },
    AddToCollection { collection_id: String, item_id: i64 },
    RemoveFromCollection { collection_id: String, item_id: i64 },
    GetItemCollections { item_id: i64 },
}
```

---

## UI State

### Collections Tab State

```rust
// In applet/src/collections.rs (new module)

pub struct CollectionsState {
    pub collections: Vec<Collection>,
    pub selected_collection: Option<String>,
    pub items_in_collection: Vec<ClipboardItem>,
    pub is_loading: bool,
}
```

### Item Display State

```rust
// In applet/src/item.rs

pub struct ItemDisplayState {
    pub is_pinned: bool,
    pub is_starred: bool,
    pub collections: Vec<Collection>,
    // Visual indicators
    pub pin_icon_visible: bool,
    pub star_icon_visible: bool,
    pub collection_badges: Vec<String>,  // Collection name badges
}
```

---

**Last Updated**: Phase 15