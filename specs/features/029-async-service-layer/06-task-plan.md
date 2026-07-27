# Task plan

1. Extend IPC request/response correlation and daemon response writing; verify shared round trips.
2. Add typed service errors, requests, results, worker, timeouts, and reconnect behavior; verify with transport tests.
3. Route clipboard history/search/actions through the service; verify stale search and explicit errors.
4. Route status, snippets, collections, settings, and refreshes through the service; verify callbacks do not perform I/O.
5. Add mock-service UI tests and rebuild documentation; run workspace verification commands.
