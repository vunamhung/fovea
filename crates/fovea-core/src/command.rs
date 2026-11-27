use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::schema::{
    AnnotationItem, CaptionWord, FreezeSegment, SpeedSegment, VoiceClip, ZoomRegion,
};
use crate::style::{CursorStyle, FrameStyle, KeystrokeHudStyle, WebcamConfig};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum EditCommand {
    SetTitle { title: String },
    TrimClip {
        clip_id: Uuid,
        source_start_ms: u64,
        source_end_ms: u64,
    },
    SplitClip {
        clip_id: Uuid,
        split_source_ms: u64,
    },
    DeleteClip { clip_id: Uuid },
    ReorderClip { clip_id: Uuid, new_index: usize },
    AddSpeedSegment(SpeedSegment),
    AddFreezeSegment(FreezeSegment),
    AddZoomRegion(ZoomRegion),
    RemoveZoomRegion { id: Uuid },
    AddAnnotation(AnnotationItem),
    RemoveAnnotation { id: Uuid },
    UpsertCaptions { words: Vec<CaptionWord> },
    InsertVoiceClip(VoiceClip),
    UpdateFrameStyle(FrameStyle),
    UpdateCursorStyle(CursorStyle),
    UpdateKeystrokeHud(KeystrokeHudStyle),
    UpdateWebcam(WebcamConfig),
}
