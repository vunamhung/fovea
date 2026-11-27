use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::style::{CursorStyle, FrameStyle, KeystrokeHudStyle, WebcamConfig};

pub const SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SourceMedia {
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub duration_ms: u64,
    pub screen_video: String,
    pub webcam_video: Option<String>,
    pub system_audio: Option<String>,
    pub mic_audio: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClipSegment {
    pub id: Uuid,
    pub source_start_ms: u64,
    pub source_end_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpeedSegment {
    pub id: Uuid,
    pub start_ms: u64,
    pub end_ms: u64,
    pub rate: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FreezeSegment {
    pub id: Uuid,
    pub source_timestamp_ms: u64,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ZoomRegion {
    pub id: Uuid,
    pub start_ms: u64,
    pub end_ms: u64,
    pub scale: f32,
    pub focus_x: f32,
    pub focus_y: f32,
    pub auto_generated: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AnnotationKind {
    Text { content: String, font_size: f32, color_hex: String },
    Shape { shape: String, stroke_hex: String, stroke_width: f32 },
    Image { path: String },
    PrivacyBlur { blur_strength: f32 },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnnotationItem {
    pub id: Uuid,
    pub start_ms: u64,
    pub end_ms: u64,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub payload: AnnotationKind,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CaptionWord {
    pub text: String,
    pub start_ms: u64,
    pub end_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VoiceSyncMode {
    KeepVideo,
    FreezeToFit,
    SlowToFit,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VoiceClip {
    pub id: Uuid,
    pub start_ms: u64,
    pub duration_ms: u64,
    pub audio_path: String,
    pub script: String,
    pub provider: String,
    pub voice_code: String,
    pub auto_duck: bool,
    pub duck_gain_db: f32,
    pub video_sync: VoiceSyncMode,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectDoc {
    pub schema_version: u32,
    pub id: Uuid,
    pub title: String,
    pub source: SourceMedia,
    pub clips: Vec<ClipSegment>,
    pub speed_segments: Vec<SpeedSegment>,
    pub freeze_segments: Vec<FreezeSegment>,
    pub zoom_regions: Vec<ZoomRegion>,
    pub annotations: Vec<AnnotationItem>,
    pub captions: Vec<CaptionWord>,
    pub voice_clips: Vec<VoiceClip>,
    pub frame: FrameStyle,
    pub cursor: CursorStyle,
    pub keystroke_hud: KeystrokeHudStyle,
    pub webcam: WebcamConfig,
}

impl ProjectDoc {
    pub fn new(title: impl Into<String>, width: u32, height: u32, fps: u32) -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            id: Uuid::new_v4(),
            title: title.into(),
            source: SourceMedia {
                width,
                height,
                fps: fps.max(1),
                duration_ms: 0,
                screen_video: "screen.mov".into(),
                webcam_video: None,
                system_audio: None,
                mic_audio: None,
            },
            clips: vec![ClipSegment {
                id: Uuid::new_v4(),
                source_start_ms: 0,
                source_end_ms: 0,
            }],
            speed_segments: Vec::new(),
            freeze_segments: Vec::new(),
            zoom_regions: Vec::new(),
            annotations: Vec::new(),
            captions: Vec::new(),
            voice_clips: Vec::new(),
            frame: FrameStyle::default(),
            cursor: CursorStyle::default(),
            keystroke_hud: KeystrokeHudStyle::default(),
            webcam: WebcamConfig::default(),
        }
    }
}
