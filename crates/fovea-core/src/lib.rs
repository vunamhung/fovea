//! Core domain schema, timeline math, physics, and undoable command engine for Fovea.

pub mod bundle;
pub mod command;
pub mod engine;
pub mod error;
pub mod physics;
pub mod schema;
pub mod stage;
pub mod style;
pub mod telemetry;
pub mod timeline;

pub use bundle::FoveaBundle;
pub use command::EditCommand;
pub use engine::CommandEngine;
pub use error::{CoreError, CoreResult};
pub use physics::{
    CameraPose, CursorPose, SpringState, evaluate_camera_pose, evaluate_cursor_pose,
};
pub use schema::{
    AnnotationItem, AnnotationKind, CaptionWord, ClipSegment, FreezeSegment, ProjectDoc,
    SCHEMA_VERSION, SourceMedia, SpeedSegment, VoiceClip, VoiceSyncMode, ZoomRegion,
};
pub use stage::{RectF, StageTransform};
pub use style::{
    AspectRatio, BackgroundStyle, CropRect, CursorStyle, FrameStyle, KeystrokeHudStyle,
    WebcamConfig,
};
pub use telemetry::{AxClickContext, MouseButton, TelemetryEvent, TelemetryReadReport};
pub use timeline::{MappedFrame, TimelineMap};
