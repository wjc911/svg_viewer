use thiserror::Error;

#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum SvgError {
    #[error("Failed to read file: {0}")]
    Io(#[from] std::io::Error),

    #[error("Failed to parse SVG: {0}")]
    Parse(String),

    #[error("Failed to render SVG: {0}")]
    Render(String),

    #[error("Failed to export image: {0}")]
    Export(String),

    #[error("Clipboard error: {0}")]
    Clipboard(String),

    #[error("No file loaded")]
    NoFile,
}

pub type Result<T> = std::result::Result<T, SvgError>;
