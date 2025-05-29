use thiserror::Error;

#[derive(Error, Debug)]
pub enum ImpositionError {
    #[error("PDF处理错误: {0}")]
    PDF(#[from] lopdf::Error),

    #[error("IO错误: {0}")]
    IO(#[from] std::io::Error),

    #[error("无效的页面尺寸")]
    InvalidPageSize,

    #[error("无效的页面数量: {0}")]
    InvalidPageCount(usize),

    #[error("页面索引超出范围: {0}")]
    PageIndexOutOfRange(usize),

    #[error("无法创建空白页")]
    FailedToCreateBlankPage,

    #[error("无法获取页面尺寸")]
    FailedToGetPageSize,

    #[error("无法创建拼版内容")]
    FailedToCreateImpositionContent,

    #[error("无法完成文档处理")]
    FailedToFinalizeDocument,
}