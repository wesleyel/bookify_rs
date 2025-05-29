use thiserror::Error;

#[derive(Error, Debug)]
pub enum ImpositionError {
    #[error("error processing pdf file: {0}")]
    PDF(#[from] lopdf::Error),
    #[error("{0}")]
    IO(#[from] std::io::Error),
}