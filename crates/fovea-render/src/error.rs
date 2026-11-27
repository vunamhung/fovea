use thiserror::Error;

pub type RenderResult<T> = Result<T, RenderError>;

#[derive(Debug, Error)]
pub enum RenderError {
    #[error("media file not found: {0}")]
    NotFound(String),

    #[error("native video decode error: {0}")]
    Decode(String),
}
