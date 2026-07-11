use std::path::PathBuf;

use author_clipboard_shared::{ClipboardItem, Database};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args_os()
        .nth(1)
        .map_or_else(|| PathBuf::from("/tmp/author-clipboard-perf.db"), PathBuf::from);
    if path.exists() {
        std::fs::remove_file(&path)?;
    }
    let db = Database::open(&path)?;
    let started = std::time::Instant::now();
    for index in 0..5_000 {
        let kind = match index % 5 {
            0 => "cargo test workspace command",
            1 => "https://example.invalid/issues/clipboard",
            2 => "JSON response payload for local development",
            3 => "Wayland compositor diagnostic output",
            _ => "ordinary clipboard note",
        };
        let mut item = ClipboardItem::new_text(format!("seed-{index:04} {kind}"));
        item.starred = index % 97 == 0;
        item.pinned = index % 251 == 0;
        db.insert_item(&item)?;
    }
    println!(
        "seeded=5000 path={} elapsed_ms={}",
        path.display(),
        started.elapsed().as_millis()
    );
    Ok(())
}
