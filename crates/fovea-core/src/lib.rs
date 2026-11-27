//! Core domain schema, timeline math, physics, and undoable command engine for Fovea.

pub mod bundle;
pub mod error;
pub mod schema;
pub mod style;
pub mod telemetry;

pub use bundle::FoveaBundle;
pub use error::{CoreError, CoreResult};
pub use schema::{
    AnnotationItem, AnnotationKind, CaptionWord, ClipSegment, FreezeSegment, ProjectDoc,
    SCHEMA_VERSION, SourceMedia, SpeedSegment, VoiceClip, VoiceSyncMode, ZoomRegion,
};
pub use style::{
    AspectRatio, BackgroundStyle, CropRect, CursorStyle, FrameStyle, KeystrokeHudStyle,
    WebcamConfig,
};
pub use telemetry::{AxClickContext, MouseButton, TelemetryEvent, TelemetryReadReport};
