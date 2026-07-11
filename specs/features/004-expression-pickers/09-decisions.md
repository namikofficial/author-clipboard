# Decisions: Expression Pickers

- Use the existing SQLite `recently_used` table rather than introduce settings storage.
- Keep separate navigation pages while sharing their implementation; this preserves
  established manager navigation and avoids a redundant nested tab bar.
