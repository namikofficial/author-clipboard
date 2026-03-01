//! Image file storage and thumbnail generation

use std::path::{Path, PathBuf};

use tracing::{debug, warn};

const THUMBNAIL_SIZE: u32 = 128;

/// Ensure the image and thumbnail directories exist.
pub fn ensure_dirs(data_dir: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(data_dir.join("images"))?;
    std::fs::create_dir_all(data_dir.join("thumbnails"))?;
    Ok(())
}

/// Derive a filename from the content hash and MIME type.
pub fn filename_for(hash: u64, mime_type: &str) -> String {
    let ext = match mime_type {
        "image/jpeg" | "image/jpg" => "jpg",
        "image/gif" => "gif",
        "image/bmp" => "bmp",
        "image/webp" => "webp",
        "image/svg+xml" => "svg",
        _ => "png",
    };
    format!("{hash:016x}.{ext}")
}

/// Save image data to disk and generate a thumbnail.
/// Returns the relative filename (e.g. "00abcdef01234567.png").
pub fn save_image(
    data_dir: &Path,
    data: &[u8],
    mime_type: &str,
    hash: u64,
) -> Result<String, String> {
    let filename = filename_for(hash, mime_type);
    let image_path = data_dir.join("images").join(&filename);
    let thumb_path = data_dir.join("thumbnails").join(&filename);

    // Skip if already stored (dedup)
    if image_path.exists() {
        debug!("Image already stored: {filename}");
        return Ok(filename);
    }

    // Write the full image
    std::fs::write(&image_path, data).map_err(|e| format!("Failed to write image: {e}"))?;
    debug!(
        "Saved image: {} ({} bytes)",
        image_path.display(),
        data.len()
    );

    // Generate thumbnail
    if let Err(e) = generate_thumbnail(&image_path, &thumb_path) {
        warn!("Thumbnail generation failed: {e}");
        // Copy the original as fallback thumbnail
        let _ = std::fs::copy(&image_path, &thumb_path);
    }

    Ok(filename)
}

/// Generate a thumbnail from an image file.
fn generate_thumbnail(source: &Path, dest: &Path) -> Result<(), String> {
    let img = image::open(source).map_err(|e| format!("Failed to open image: {e}"))?;
    let thumb = img.thumbnail(THUMBNAIL_SIZE, THUMBNAIL_SIZE);
    thumb
        .save(dest)
        .map_err(|e| format!("Failed to save thumbnail: {e}"))?;
    debug!("Generated thumbnail: {}", dest.display());
    Ok(())
}

/// Get the full path to a stored image.
pub fn image_path(data_dir: &Path, filename: &str) -> PathBuf {
    data_dir.join("images").join(filename)
}

/// Get the full path to a thumbnail.
pub fn thumbnail_path(data_dir: &Path, filename: &str) -> PathBuf {
    data_dir.join("thumbnails").join(filename)
}

/// Delete image and thumbnail files.
pub fn delete_image_files(data_dir: &Path, filename: &str) {
    let img = image_path(data_dir, filename);
    let thumb = thumbnail_path(data_dir, filename);
    if img.exists() {
        let _ = std::fs::remove_file(&img);
    }
    if thumb.exists() {
        let _ = std::fs::remove_file(&thumb);
    }
}

/// Check if a MIME type is a supported image format.
pub fn is_image_mime(mime: &str) -> bool {
    matches!(
        mime,
        "image/png"
            | "image/jpeg"
            | "image/jpg"
            | "image/gif"
            | "image/bmp"
            | "image/webp"
            | "image/tiff"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filename_for() {
        assert_eq!(filename_for(42, "image/png"), "000000000000002a.png");
        assert_eq!(filename_for(42, "image/jpeg"), "000000000000002a.jpg");
        assert_eq!(filename_for(42, "image/gif"), "000000000000002a.gif");
        assert_eq!(filename_for(42, "text/plain"), "000000000000002a.png");
    }

    #[test]
    fn test_is_image_mime() {
        assert!(is_image_mime("image/png"));
        assert!(is_image_mime("image/jpeg"));
        assert!(!is_image_mime("text/plain"));
        assert!(!is_image_mime("image/svg+xml"));
    }
}
