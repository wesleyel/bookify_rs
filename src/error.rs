use thiserror::Error;

/// Bookify error type
#[derive(Debug, Error)]
pub enum BookifyError {
    /// PDF processing error
    #[error("PDF processing error: {0}")]
    PdfError(#[from] lopdf::Error),
    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    /// Other error
    #[error("Other error: {0}")]
    Other(String),
}
