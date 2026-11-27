use std::path::PathBuf;

use fovea_core::FoveaBundle;
use serde::{Deserialize, Serialize};

use crate::error::{CaptureError, CaptureResult};
use crate::permissions::{PermissionKind, PermissionReport};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CaptureSourceKind {
    Display,
    Window,
    Region,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CaptureSource {
    pub id: u32,
    pub kind: CaptureSourceKind,
    pub name: String,
    pub app_name: Option<String>,
    pub width: u32,
    pub height: u32,
}

impl CaptureSource {
    pub fn display(id: u32, name: impl Into<String>, width: u32, height: u32) -> Self {
        Self {
            id,
            kind: CaptureSourceKind::Display,
            name: name.into(),
            app_name: None,
            width,
            height,
        }
    }

    pub fn window(
        id: u32,
        name: impl Into<String>,
        app_name: Option<String>,
        width: u32,
        height: u32,
    ) -> Self {
        Self {
            id,
            kind: CaptureSourceKind::Window,
            name: name.into(),
            app_name,
            width,
            height,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CaptureTarget {
    Display { display_id: u32 },
    Window { window_id: u32 },
    Region {
        display_id: u32,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CaptureConfig {
    pub target: CaptureTarget,
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub capture_system_audio: bool,
    pub capture_microphone: bool,
    pub capture_webcam: bool,
    pub hide_os_cursor: bool,
}

impl CaptureConfig {
    pub fn default_for_display(display_id: u32, width: u32, height: u32) -> Self {
        Self {
            target: CaptureTarget::Display { display_id },
            width: width & !1,
            height: height & !1,
            fps: 60,
            capture_system_audio: true,
            capture_microphone: false,
            capture_webcam: false,
            hide_os_cursor: true,
        }
    }

    pub fn validate(&self) -> CaptureResult<()> {
        if self.width < 2 || self.height < 2 {
            return Err(CaptureError::InvalidConfig(
                "width and height must be at least 2px".into(),
            ));
        }
        if !self.width.is_multiple_of(2) || !self.height.is_multiple_of(2) {
            return Err(CaptureError::InvalidConfig(
                "width and height must be even for hardware H.264/HEVC".into(),
            ));
        }
        if self.fps == 0 || self.fps > 240 {
            return Err(CaptureError::InvalidConfig(
                "fps must be between 1 and 240".into(),
            ));
        }
        if let CaptureTarget::Region { width, height, .. } = self.target
            && (width < 2 || height < 2)
        {
            return Err(CaptureError::InvalidConfig(
                "region width and height must be at least 2px".into(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CaptureStats {
    pub video_frames: u64,
    pub audio_buffers: u64,
    pub duration_ms: u64,
}

#[derive(Debug)]
pub struct WriterGuard {
    output_path: PathBuf,
    video_frames: u64,
    audio_buffers: u64,
    duration_ms: u64,
}

impl WriterGuard {
    pub fn new(output_path: PathBuf) -> Self {
        Self {
            output_path,
            video_frames: 0,
            audio_buffers: 0,
            duration_ms: 0,
        }
    }

    pub fn record_video_frame(&mut self, duration_ms: u64) {
        self.video_frames += 1;
        self.duration_ms = self.duration_ms.max(duration_ms);
    }

    pub fn record_audio_buffer(&mut self) {
        self.audio_buffers += 1;
    }

    pub fn video_frames(&self) -> u64 {
        self.video_frames
    }

    pub fn stats(&self) -> CaptureStats {
        CaptureStats {
            video_frames: self.video_frames,
            audio_buffers: self.audio_buffers,
            duration_ms: self.duration_ms,
        }
    }

    pub fn finish_if_empty(&mut self) -> CaptureResult<CaptureStats> {
        if self.video_frames == 0 && self.output_path.exists() {
            let _ = std::fs::remove_file(&self.output_path);
        }
        Ok(self.stats())
    }
}

pub trait CaptureBackend {
    fn check_permissions(&self) -> PermissionReport;
    fn open_permission_settings(&self, kind: PermissionKind) -> CaptureResult<()>;
    fn list_sources(&self) -> CaptureResult<Vec<CaptureSource>>;
    fn start(&mut self, config: CaptureConfig, bundle: &FoveaBundle) -> CaptureResult<()>;
    fn stop(&mut self) -> CaptureResult<CaptureStats>;
}
