use std::path::{Path, PathBuf};
use usvg::{Options, Tree};

use crate::error::{Result, SvgError};

#[allow(dead_code)]
pub struct SvgDocument {
    pub tree: Tree,
    pub path: PathBuf,
    pub raw_data: Vec<u8>,
    pub width: f32,
    pub height: f32,
    pub file_size: u64,
}

impl SvgDocument {
    pub fn load(path: &Path) -> Result<Self> {
        let raw_data = std::fs::read(path)?;
        let file_size = raw_data.len() as u64;

        let tree = Tree::from_data(&raw_data, &Options::default())
            .map_err(|e| SvgError::Parse(e.to_string()))?;

        let size = tree.size();
        let width = size.width();
        let height = size.height();

        Ok(SvgDocument {
            tree,
            path: path.to_path_buf(),
            raw_data,
            width,
            height,
            file_size,
        })
    }

    pub fn filename(&self) -> &str {
        self.path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
    }

    pub fn file_size_display(&self) -> String {
        if self.file_size < 1024 {
            format!("{} B", self.file_size)
        } else if self.file_size < 1024 * 1024 {
            format!("{:.1} KB", self.file_size as f64 / 1024.0)
        } else {
            format!("{:.1} MB", self.file_size as f64 / (1024.0 * 1024.0))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_path(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("assets")
            .join("test_fixtures")
            .join(name)
    }

    #[test]
    fn test_load_simple_rect() {
        let doc = SvgDocument::load(&fixture_path("simple_rect.svg")).unwrap();
        assert_eq!(doc.width, 200.0);
        assert_eq!(doc.height, 150.0);
        assert_eq!(doc.filename(), "simple_rect.svg");
        assert!(doc.file_size > 0);
    }

    #[test]
    fn test_load_gradient() {
        let doc = SvgDocument::load(&fixture_path("gradient.svg")).unwrap();
        assert_eq!(doc.width, 200.0);
        assert_eq!(doc.height, 200.0);
    }

    #[test]
    fn test_load_transparent() {
        let doc = SvgDocument::load(&fixture_path("transparent.svg")).unwrap();
        assert_eq!(doc.width, 100.0);
        assert_eq!(doc.height, 100.0);
    }

    #[test]
    fn test_load_malformed_fails() {
        let result = SvgDocument::load(&fixture_path("malformed.svg"));
        assert!(result.is_err());
    }

    #[test]
    fn test_load_nonexistent_fails() {
        let result = SvgDocument::load(&fixture_path("does_not_exist.svg"));
        assert!(result.is_err());
    }

    #[test]
    fn test_file_size_display_bytes() {
        let doc = SvgDocument::load(&fixture_path("transparent.svg")).unwrap();
        let display = doc.file_size_display();
        // Should be a few hundred bytes
        assert!(display.contains("B"));
    }

    #[test]
    fn test_filename() {
        let doc = SvgDocument::load(&fixture_path("simple_rect.svg")).unwrap();
        assert_eq!(doc.filename(), "simple_rect.svg");
    }
}
