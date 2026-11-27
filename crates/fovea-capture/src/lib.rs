//! Cross-platform capture traits and native macOS ScreenCaptureKit + telemetry backend.

pub mod backend;
pub mod error;
pub mod permissions;

#[cfg(target_os = "macos")]
pub mod macos;

pub use backend::{
    CaptureBackend, CaptureConfig, CaptureSource, CaptureSourceKind, CaptureStats, CaptureTarget,
    WriterGuard,
};
pub use error::{CaptureError, CaptureResult};
pub use permissions::{PermissionEntry, PermissionKind, PermissionReport, PermissionState};

#[cfg(target_os = "macos")]
pub use macos::MacOsCaptureBackend;
