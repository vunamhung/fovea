use thiserror::Error;

pub type CaptureResult<T> = Result<T, CaptureError>;

#[derive(Debug, Error)]
pub enum CaptureError {
    #[error("invalid capture configuration: {0}")]
    InvalidConfig(String),

    #[error("capture session is already running")]
    AlreadyRecording,

    #[error("required permission denied: {0}")]
    PermissionDenied(String),

    #[error("capture source not found: {0}")]
    SourceNotFound(String),

    #[error("native capture backend error: {0}")]
    Backend(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("core error: {0}")]
    Core(#[from] fovea_core::CoreError),
}
