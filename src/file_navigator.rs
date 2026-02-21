use std::path::{Path, PathBuf};

pub struct FileNavigator {
    pub files: Vec<PathBuf>,
    pub current_index: usize,
}

impl FileNavigator {
    pub fn new() -> Self {
        Self {
            files: Vec::new(),
            current_index: 0,
        }
    }

    /// Scan the directory of the given file for SVG files and set the current index.
    pub fn scan_directory(&mut self, file_path: &Path) {
        let dir = match file_path.parent() {
            Some(d) => d,
            None => return,
        };

        let mut svg_files: Vec<PathBuf> = Vec::new();

        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Some(ext) = path.extension() {
                        let ext_lower = ext.to_string_lossy().to_lowercase();
                        if ext_lower == "svg" || ext_lower == "svgz" {
                            svg_files.push(path);
                        }
                    }
                }
            }
        }

        // Natural sort
        svg_files.sort_by(|a, b| {
            let a_name = a.file_name().unwrap_or_default().to_string_lossy();
            let b_name = b.file_name().unwrap_or_default().to_string_lossy();
            natord::compare(&a_name, &b_name)
        });

        // Find current file index
        let canonical = file_path.canonicalize().ok();
        self.current_index = svg_files
            .iter()
            .position(|p| {
                if let (Some(ref c), Ok(pc)) = (&canonical, p.canonicalize()) {
                    c == &pc
                } else {
                    p == file_path
                }
            })
            .unwrap_or(0);

        self.files = svg_files;
    }

    pub fn next(&mut self) -> Option<&Path> {
        if self.files.is_empty() {
            return None;
        }
        self.current_index = (self.current_index + 1) % self.files.len();
        Some(&self.files[self.current_index])
    }

    pub fn prev(&mut self) -> Option<&Path> {
        if self.files.is_empty() {
            return None;
        }
        self.current_index = if self.current_index == 0 {
            self.files.len() - 1
        } else {
            self.current_index - 1
        };
        Some(&self.files[self.current_index])
    }

    #[allow(dead_code)]
    pub fn current(&self) -> Option<&Path> {
        self.files.get(self.current_index).map(|p| p.as_path())
    }

    pub fn position_display(&self) -> String {
        if self.files.is_empty() {
            String::new()
        } else {
            format!("{}/{}", self.current_index + 1, self.files.len())
        }
    }

    pub fn file_count(&self) -> usize {
        self.files.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_navigator() {
        let nav = FileNavigator::new();
        assert!(nav.files.is_empty());
        assert_eq!(nav.current_index, 0);
        assert_eq!(nav.position_display(), "");
        assert_eq!(nav.file_count(), 0);
    }

    #[test]
    fn test_next_prev_empty() {
        let mut nav = FileNavigator::new();
        assert!(nav.next().is_none());
        assert!(nav.prev().is_none());
    }

    #[test]
    fn test_next_wraps() {
        let mut nav = FileNavigator::new();
        nav.files = vec![
            PathBuf::from("/a.svg"),
            PathBuf::from("/b.svg"),
            PathBuf::from("/c.svg"),
        ];
        nav.current_index = 2;
        nav.next();
        assert_eq!(nav.current_index, 0);
    }

    #[test]
    fn test_prev_wraps() {
        let mut nav = FileNavigator::new();
        nav.files = vec![
            PathBuf::from("/a.svg"),
            PathBuf::from("/b.svg"),
            PathBuf::from("/c.svg"),
        ];
        nav.current_index = 0;
        nav.prev();
        assert_eq!(nav.current_index, 2);
    }

    #[test]
    fn test_position_display() {
        let mut nav = FileNavigator::new();
        nav.files = vec![PathBuf::from("/a.svg"), PathBuf::from("/b.svg")];
        nav.current_index = 0;
        assert_eq!(nav.position_display(), "1/2");
        nav.current_index = 1;
        assert_eq!(nav.position_display(), "2/2");
    }

    #[test]
    fn test_scan_directory() {
        let fixtures_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("assets")
            .join("test_fixtures");
        let test_file = fixtures_dir.join("simple_rect.svg");

        if test_file.exists() {
            let mut nav = FileNavigator::new();
            nav.scan_directory(&test_file);
            assert!(nav.file_count() >= 1);
            // Should find at least the simple_rect.svg
            assert!(nav
                .files
                .iter()
                .any(|f| f.file_name().unwrap().to_string_lossy() == "simple_rect.svg"));
        }
    }

    #[test]
    fn test_natural_sort_order() {
        let mut nav = FileNavigator::new();
        nav.files = vec![
            PathBuf::from("/dir/file10.svg"),
            PathBuf::from("/dir/file2.svg"),
            PathBuf::from("/dir/file1.svg"),
        ];
        // Simulate what scan_directory does
        nav.files.sort_by(|a, b| {
            let a_name = a.file_name().unwrap_or_default().to_string_lossy();
            let b_name = b.file_name().unwrap_or_default().to_string_lossy();
            natord::compare(&a_name, &b_name)
        });
        assert_eq!(
            nav.files[0].file_name().unwrap().to_string_lossy(),
            "file1.svg"
        );
        assert_eq!(
            nav.files[1].file_name().unwrap().to_string_lossy(),
            "file2.svg"
        );
        assert_eq!(
            nav.files[2].file_name().unwrap().to_string_lossy(),
            "file10.svg"
        );
    }
}
