use std::path::PathBuf;

use author_clipboard_shared::config::Config;
use author_clipboard_shared::picker::{load_entries, PickerAction, PickerOptions, PickerSource};
use author_clipboard_shared::Database;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/tmp/author-clipboard-perf.db"));
    let db = Database::open(&path)?;
    let config = Config::default();
    for query in [None, Some("cargo".to_string()), Some("Wayland".to_string())] {
        let options = PickerOptions {
            source: PickerSource::History,
            limit: 100,
            query: query.clone(),
            include_sensitive: false,
            action: PickerAction::Copy,
        };
        let started = std::time::Instant::now();
        let entries = load_entries(&db, &config, &options)?;
        println!(
            "query={:?} results={} elapsed_us={}",
            query,
            entries.len(),
            started.elapsed().as_micros()
        );
    }
    Ok(())
}
