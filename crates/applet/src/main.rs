//! author-clipboard: COSMIC clipboard manager applet
//!
//! A graphical interface for browsing and selecting from clipboard history.
//! Currently a placeholder — Phase 1 will implement the full COSMIC UI.

use tracing::info;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    info!("author-clipboard applet starting...");

    // TODO: Phase 1 - Implement COSMIC UI
    // 1. Create libcosmic application
    // 2. Build UI: search bar + list + actions
    // 3. Connect to database
    // 4. Handle selection and paste

    println!("author-clipboard applet placeholder — UI coming in Phase 1!");
}
