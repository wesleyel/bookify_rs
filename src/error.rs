use std::path::PathBuf;
use thiserror::Error;

/// Bookify error type
#[derive(Debug, Error)]
pub enum BookifyError {
    /// PDF processing error
    #[error("PDF processing error: {0}")]
    PdfError(#[from] lopdf::Error),

    /// IO error with file path
    #[error("IO error (file: {path}): {source}")]
    IoError {
        #[source]
        source: std::io::Error,
        path: PathBuf,
    },

    /// PDF file not found
    #[error("PDF file not found: {path}")]
    PdfFileNotFound { path: PathBuf },

    /// Invalid PDF format
    #[error("Invalid PDF format: {message}")]
    InvalidPdfFormat { message: String },

    /// PDF processing failed
    #[error("PDF processing failed: {operation} - {details}")]
    PdfProcessingFailed { operation: String, details: String },

    /// Other error with context
    #[error("Other error: {context} - {message}")]
    Other { context: String, message: String },
}

impl BookifyError {
    /// Create an IO error with file path
    pub fn io_error(source: std::io::Error, path: impl Into<PathBuf>) -> Self {
        Self::IoError {
            source,
            path: path.into(),
        }
    }

    /// Create a PDF file not found error
    pub fn pdf_file_not_found(path: impl Into<PathBuf>) -> Self {
        Self::PdfFileNotFound { path: path.into() }
    }

    /// Create an invalid PDF format error
    pub fn invalid_pdf_format(message: impl Into<String>) -> Self {
        Self::InvalidPdfFormat {
            message: message.into(),
        }
    }

    /// Create a PDF processing failed error
    pub fn pdf_processing_failed(operation: impl Into<String>, details: impl Into<String>) -> Self {
        Self::PdfProcessingFailed {
            operation: operation.into(),
            details: details.into(),
        }
    }

    /// Create an other error with context
    pub fn other(context: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Other {
            context: context.into(),
            message: message.into(),
        }
    }
}
