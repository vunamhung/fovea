//! Cross-platform capture traits and native macOS ScreenCaptureKit + telemetry backend.

pub mod backend;
pub mod error;
pub mod keystroke_filter;
pub mod permissions;
pub mod telemetry_tap;

#[cfg(target_os = "macos")]
pub mod macos;

pub use backend::{
    CaptureBackend, CaptureConfig, CaptureSource, CaptureSourceKind, CaptureStats, CaptureTarget,
    WriterGuard,
};
pub use error::{CaptureError, CaptureResult};
pub use keystroke_filter::{KeyModifiers, RawKeyEvent, filter_keystroke};
pub use permissions::{PermissionEntry, PermissionKind, PermissionReport, PermissionState};
pub use telemetry_tap::{TelemetrySink, TelemetryTapConfig, TelemetryTapHandle};

#[cfg(target_os = "macos")]
pub use macos::MacOsCaptureBackend;
