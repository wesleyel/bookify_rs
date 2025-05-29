use thiserror::Error;

/// 小册子排版错误类型
#[derive(Debug, Error)]
pub enum ImpositionError {
    /// PDF 处理相关错误
    #[error("PDF processing error: {0}")]
    PdfError(#[from] lopdf::Error),
    /// IO 错误
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    /// 其他错误
    #[error("Other error: {0}")]
    Other(String),
}
