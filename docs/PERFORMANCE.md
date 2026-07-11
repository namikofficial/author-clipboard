# Performance Checks

Performance claims must be reproducible on a local machine and are not release
guarantees. The benchmark fixture is synthetic, contains no user clipboard
data, and always writes `/tmp/author-clipboard-perf.db` unless a path is passed
directly to the examples.

```bash
just perf-seed
just perf-picker
```

`perf-seed` recreates a 5,000-item database with a stable mix of command, URL,
JSON, diagnostic, pinned, and starred entries. `perf-picker` reports cold local
load/search timings for an unfiltered view and two queries. Record hardware,
build profile, compositor, and observed timings when using results in a release
review. Scroll and frame pacing still require the manual UI matrix; this CLI
benchmark deliberately does not claim to measure rendering.

To retain and compare a fixture explicitly:

```bash
cargo run -p author-clipboard-shared --example perf_seed -- /tmp/ac-perf-a.db
cargo run -p author-clipboard-shared --example perf_picker -- /tmp/ac-perf-a.db
```

