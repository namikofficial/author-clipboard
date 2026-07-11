# API Contract: Expression Pickers

No IPC or CLI changes. The UI uses `Database::record_usage` and
`Database::get_recently_used`, and restores expressions with
`clipboard::set_clipboard_text`.
