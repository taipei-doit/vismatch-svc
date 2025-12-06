pub mod api;
pub mod vec_ops;
pub mod metric;
pub mod image_hash;

use std::fs::DirEntry;  // filesystem utils

// Some common ext for images.
const IMAGE_EXTENSIONS: [&str; 8] = [
    "png", "jpg", "jpeg", "gif", "bmp", "ico", "webp", "tiff" // We could consider accept only top-3 later?
];

/// Check if a given file is an image file
pub fn is_image_file(file: &DirEntry) -> bool {
    match file.path().is_file() {
        false => false,
        true => {
            match file.path().extension() {
                None => false,
                Some(ext) => {
                    IMAGE_EXTENSIONS.contains(
                        &ext.to_string_lossy()
                            .to_lowercase()
                            .as_str())
                },
            }
        },
    }
}