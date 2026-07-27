# Requirements

## User stories

- As a user, typing, filtering, copying, or changing collections must leave the GTK interface responsive while the daemon is slow.
- As a user, I must see an explicit offline, timeout, protocol, validation, or database failure instead of an apparently empty history.
- As a user, a restarted daemon should become usable again without restarting the UI.
- As a user, an older search result must never replace the result for a newer query.

## Acceptance criteria

1. GTK callbacks enqueue service requests and return without socket or database waits.
2. Service operations use typed inputs/results and return errors by category.
3. Connection, write, and response timeouts are enforced at 500 ms, 500 ms, and 2 s by default.
4. Every request has an ID; responses are validated against the ID and protocol version.
5. Search requests carry a generation and stale generations are discarded.
6. The daemon can be stopped and restarted while an existing UI service handle reconnects.
7. Empty results are rendered as empty only for successful responses.
8. UI tests use a mock service and cover delayed responses, unavailable/restarting daemon, out-of-order searches, malformed responses, timeout, and rapid filtering.
