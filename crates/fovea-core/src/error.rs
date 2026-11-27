use thiserror::Error;

pub type CoreResult<T> = Result<T, CoreError>;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("unsupported project schema version: {0}")]
    UnsupportedSchemaVersion(u32),

    #[error("invalid range or parameter: {0}")]
    InvalidRange(String),

    #[error("item not found: {0}")]
    NotFound(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
}
