# Domain Model: Expression Pickers

- `ExpressionItem`: display value, searchable description, and category.
- `ExpressionKind`: emoji, symbol, or kaomoji; also the persistence category key.
- `PickerState`: current normalized search query and optional category selection.
- `recently_used`: persisted expression value, kind, last-used timestamp, and use count.
