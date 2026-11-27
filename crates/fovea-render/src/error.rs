use thiserror::Error;

pub type RenderResult<T> = Result<T, RenderError>;

#[derive(Debug, Error)]
pub enum RenderError {
    #[error("media file not found: {0}")]
    NotFound(String),

    #[error("native video decode error: {0}")]
    Decode(String),

    #[error("video export error: {0}")]
    Encode(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}
