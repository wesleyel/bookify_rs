use thiserror::Error;
use std::path::PathBuf;

/// Bookify error type
#[derive(Debug, Error)]
pub enum BookifyError {
    /// PDF processing error
    #[error("PDF处理错误: {0}")]
    PdfError(#[from] lopdf::Error),

    /// IO error with file path
    #[error("IO错误 (文件: {path}): {source}")]
    IoError {
        #[source]
        source: std::io::Error,
        path: PathBuf,
    },

    /// PDF file not found
    #[error("PDF文件未找到: {path}")]
    PdfFileNotFound {
        path: PathBuf,
    },

    /// Invalid PDF format
    #[error("无效的PDF格式: {message}")]
    InvalidPdfFormat {
        message: String,
    },

    /// PDF processing failed
    #[error("PDF处理失败: {operation} - {details}")]
    PdfProcessingFailed {
        operation: String,
        details: String,
    },

    /// Other error with context
    #[error("其他错误: {context} - {message}")]
    Other {
        context: String,
        message: String,
    },
}

impl BookifyError {
    /// 创建一个带有文件路径的IO错误
    pub fn io_error(source: std::io::Error, path: impl Into<PathBuf>) -> Self {
        Self::IoError {
            source,
            path: path.into(),
        }
    }

    /// 创建一个PDF文件未找到错误
    pub fn pdf_file_not_found(path: impl Into<PathBuf>) -> Self {
        Self::PdfFileNotFound {
            path: path.into(),
        }
    }

    /// 创建一个无效PDF格式错误
    pub fn invalid_pdf_format(message: impl Into<String>) -> Self {
        Self::InvalidPdfFormat {
            message: message.into(),
        }
    }

    /// 创建一个PDF处理失败错误
    pub fn pdf_processing_failed(operation: impl Into<String>, details: impl Into<String>) -> Self {
        Self::PdfProcessingFailed {
            operation: operation.into(),
            details: details.into(),
        }
    }

    /// 创建一个带有上下文的其他错误
    pub fn other(context: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Other {
            context: context.into(),
            message: message.into(),
        }
    }
}
