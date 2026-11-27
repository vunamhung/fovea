//! Unified GPU compositor, native AVFoundation frame decoder, and hardware exporters.

pub mod error;
pub mod frame_decoder;

pub use error::{RenderError, RenderResult};
pub use frame_decoder::{DecodedFrame, extract_video_frame};
