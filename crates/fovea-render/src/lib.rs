//! Unified frame compositor, native AVFoundation frame decoder, and MP4/GIF hardware exporters.

pub mod compositor;
pub mod error;
pub mod exporter;
pub mod frame_decoder;
#[cfg(target_os = "macos")]
pub mod mp4_writer;

pub use compositor::composite_frame;
pub use error::{RenderError, RenderResult};
pub use exporter::{ExportFormat, ExportOptions, export_project};
pub use frame_decoder::{DecodedFrame, extract_video_frame};
