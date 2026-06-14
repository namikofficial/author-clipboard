//! Build script for the unified GTK4 UI library.
//!
//! Compiles the `GResource` bundle (CSS stylesheet, SVG icons, .ui
//! templates) into a single binary resource that gets linked into
//! the crate at build time.

fn main() {
    glib_build_tools::compile_resources(
        &["data", "assets"],
        "data/resources.gresource.xml",
        "compiled.gresource",
    );
}
