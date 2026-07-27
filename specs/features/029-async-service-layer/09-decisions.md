# Decisions

- Use a Tokio worker with GLib main-context handoff because Tokio is already a workspace dependency and the transport is Unix-socket I/O.
- Use generation gating plus cooperative queued-search replacement; cancellation is an optimization, while generation matching is the correctness guarantee.
- Use persistent status/error presentation plus toast notifications so failures cannot be mistaken for empty data.
- Keep the existing one-request socket framing for compatibility with the daemon's synchronous server loop; the service channel is persistent and reconnects per operation after daemon restart.
