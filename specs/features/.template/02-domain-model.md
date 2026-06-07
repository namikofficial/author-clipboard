# Domain Model: {feature-name}

> Data structures, state, and relationships for this feature.

---

## Data Structures

### New Types

```rust
// In shared/src/types.rs or new module
struct NewType {
    field: Type,
}
```

### Changes to Existing Types

```rust
// clipboard-daemon/src/content.rs
enum ContentType {
    Text,
    Image,
    Html,
    Files,
    // NEW: Add variant
}
```

---

## State Machine

```
[State A] --> [Event] --> [State B]
     |
     v
[State C]
```

---

## Database Changes

### New Tables

```sql
-- If needed
CREATE TABLE new_table (
    id INTEGER PRIMARY KEY,
    created_at INTEGER NOT NULL,
);
```

### Schema Migrations

```rust
// shared/src/db/migrations.rs
const MIGRATION_XXX: Migration = Migration {
    version: XXX,
    description: "add feature tables",
    operations: &[
        MigrationOp::CreateTable { .. },
    ],
};
```

---

## IPC Protocol Changes

### New Commands

```json
{"cmd": "new_command", "args": {"param": "value"}}
```

### Response

```json
{"ok": true, "data": {...}, "error": null}
```

---

**Last Updated**: {date}