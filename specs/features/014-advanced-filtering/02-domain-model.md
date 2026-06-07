# Domain Model: Advanced Filtering & Saved Searches

> Data structures, state, and relationships for advanced filtering.

---

## Data Structures

### Search Query Types

```rust
// In shared/src/search.rs (new file)

/// A parsed search query with text and filters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    /// Raw text search query
    pub text: Option<String>,
    /// Composed filter chips
    pub filters: Vec<FilterChip>,
}

/// A single filter chip
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterChip {
    pub kind: FilterKind,
    pub value: FilterValue,
    pub negated: bool,  // for things like sensitive:false
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FilterKind {
    Type,
    Age,
    App,
    Pinned,
    Sensitive,
    Starred,
    Size,
    InCollection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FilterValue {
    Type(ContentType),
    Age(AgeValue),
    App(String),
    Boolean(bool),
    Size(SizeValue),
    Collection(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgeValue {
    Today,
    Week,
    Month,
    Seconds(u64),
    Minutes(u64),
    Hours(u64),
    Days(u64),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SizeValue {
    Small,   // < 1KB
    Medium,  // 1KB - 1MB
    Large,   // > 1MB
}

/// A saved search with a name
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedSearch {
    pub id: String,           // UUID
    pub name: String,         // User-facing name
    pub query: SearchQuery,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub use_count: u64,       // For sorting by popularity
}

/// Search history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchHistoryEntry {
    pub query: SearchQuery,
    pub timestamp: DateTime<Utc>,
    pub result_count: usize,
}
```

### Parser

```rust
/// Parse a search string into a SearchQuery
pub fn parse_search_query(input: &str) -> Result<SearchQuery, ParseError> {
    // Tokenize: separate text from filter chips
    // text:"foo" app:kitty -> SearchQuery { text: Some("foo"), filters: [Type(Text), App("kitty")] }
}

/// Generate filter suggestions based on current input
pub fn get_filter_suggestions(input: &str, cursor_pos: usize) -> Vec<Suggestion> {
    // If cursor is after "type:", return ["text", "image", "html", "files"]
    // If cursor is after "app:", return recently seen apps
    // etc.
}

/// Validate a search query
pub fn validate_query(query: &SearchQuery) -> Vec<ValidationError> {
    // Check for conflicting filters
    // Check for unknown filter kinds
    // Check for invalid values
}
```

---

## Database Changes

### New Tables

```sql
-- Saved searches
CREATE TABLE saved_searches (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    query_text TEXT,
    query_filters TEXT NOT NULL,  -- JSON array of FilterChip
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    use_count INTEGER NOT NULL DEFAULT 0
);

-- Search history
CREATE TABLE search_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    query_text TEXT,
    query_filters TEXT NOT NULL,  -- JSON array of FilterChip
    timestamp TEXT NOT NULL,
    result_count INTEGER NOT NULL
);

-- Create index for search history
CREATE INDEX idx_search_history_timestamp ON search_history(timestamp DESC);
```

### Schema Migration

```rust
// shared/src/db/migrations.rs
const MIGRATION_007: Migration = Migration {
    version: 7,
    description: "add saved searches and search history tables",
    operations: &[
        MigrationOp::CreateTable {
            name: "saved_searches",
            columns: &[
                ("id", "TEXT NOT NULL PRIMARY KEY"),
                ("name", "TEXT NOT NULL UNIQUE"),
                ("query_text", "TEXT"),
                ("query_filters", "TEXT NOT NULL"),
                ("created_at", "TEXT NOT NULL"),
                ("updated_at", "TEXT NOT NULL"),
                ("use_count", "INTEGER NOT NULL DEFAULT 0"),
            ],
        },
        MigrationOp::CreateTable {
            name: "search_history",
            columns: &[
                ("id", "INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT"),
                ("query_text", "TEXT"),
                ("query_filters", "TEXT NOT NULL"),
                ("timestamp", "TEXT NOT NULL"),
                ("result_count", "INTEGER NOT NULL"),
            ],
        },
        MigrationOp::CreateIndex {
            name: "idx_search_history_timestamp",
            table: "search_history",
            columns: &["timestamp DESC"],
        },
    ],
};
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

    // Search
    ParseSearchQuery { input: String },
    ExecuteSearch { query: SearchQuery, limit: Option<usize>, offset: Option<usize> },
    GetFilterSuggestions { input: String, cursor: usize },

    // Saved searches
    ListSavedSearches,
    SaveSearch { name: String, query: SearchQuery },
    DeleteSavedSearch { id: String },
    GetSavedSearch { id: String },
    IncrementSavedSearchUsage { id: String },

    // Search history
    GetSearchHistory { limit: Option<usize> },
    ClearSearchHistory,
}
```

---

## UI State

### Search State

```rust
// In applet/src/search.rs (new module)

pub struct SearchState {
    pub input_text: String,
    pub active_filters: Vec<FilterChip>,
    pub parsed_query: Option<SearchQuery>,
    pub validation_errors: Vec<ValidationError>,
    pub saved_searches: Vec<SavedSearch>,
    pub search_suggestions: Vec<Suggestion>,
    pub is_autocomplete_open: bool,
    pub autocomplete_position: usize,
}
```

---

**Last Updated**: Phase 15