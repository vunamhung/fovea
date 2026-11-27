use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use fovea_capture::{CaptureConfig, CaptureSource, CaptureSourceKind, CaptureTarget, PermissionReport};
use fovea_core::{
    AspectRatio, CommandEngine, CoreResult, EditCommand, FoveaBundle, ProjectDoc, TelemetryEvent,
    TimelineMap, generate_auto_zooms,
};
use fovea_render::ExportFormat;
use uuid::Uuid;

use crate::demo::build_demo_project;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionMode { Idle, Recording, Editing }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RightPanelTab { Inspector, Cursor, Webcam, VoiceCaptions, AiChat, Permissions, Export }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackgroundTab { Image, Video, Color, Gradient }

pub struct StudioState {
    pub engine: CommandEngine,
    pub active_bundle: Option<FoveaBundle>,
    pub active_bundle_path: Option<PathBuf>,
    pub telemetry: Vec<TelemetryEvent>,
    pub permissions: PermissionReport,
    pub sources: Vec<CaptureSource>,
    pub selected_source_idx: usize,
    pub mode: SessionMode,
    pub active_tab: RightPanelTab,
    pub bg_tab: BackgroundTab,
    pub selected_preset_idx: usize,
    pub selected_zoom_id: Option<Uuid>,
    pub selected_clip_id: Option<Uuid>,
    pub bg_blur_px: f32,
    pub remove_background: bool,
    pub crop_mode: bool,
    pub volume_pct: u8,
    pub playhead_ms: u64,
    pub is_playing: bool,
    pub countdown_enabled: bool,
    pub capture_system_audio: bool,
    pub hide_os_cursor: bool,
    pub capture_keystrokes: bool,
    pub export_format: ExportFormat,
    pub export_height: u32,
    pub export_fps: u32,
    pub export_progress: Option<f32>,
    pub last_export_path: Option<PathBuf>,
    pub last_status: String,
    pub recorded_frames: u64,
}

impl StudioState {
    pub fn new(permissions: PermissionReport, sources: Vec<CaptureSource>) -> Self {
        let (w, h) = sources.first().map(|s| (s.width.max(2) & !1, s.height.max(2) & !1)).unwrap_or((1920, 1080));
        let doc = build_demo_project(w, h);
        let first_clip = doc.clips.first().map(|c| c.id);
        Self {
            engine: CommandEngine::new(doc),
            active_bundle: None,
            active_bundle_path: None,
            telemetry: Vec::new(),
            permissions,
            sources,
            selected_source_idx: 0,
            mode: SessionMode::Idle,
            active_tab: RightPanelTab::Inspector,
            bg_tab: BackgroundTab::Image,
            selected_preset_idx: 0,
            selected_zoom_id: None,
            selected_clip_id: first_clip,
            bg_blur_px: 0.0,
            remove_background: false,
            crop_mode: false,
            volume_pct: 100,
            playhead_ms: 0,
            is_playing: false,
            countdown_enabled: true,
            capture_system_audio: true,
            hide_os_cursor: true,
            capture_keystrokes: true,
            export_format: ExportFormat::Mp4,
            export_height: 1080,
            export_fps: 30,
            export_progress: None,
            last_export_path: None,
            last_status: "Ready".into(),
            recorded_frames: 0,
        }
    }

    pub fn load_bundle(&mut self, bundle: FoveaBundle) {
        self.telemetry = bundle.read_telemetry_lenient().map(|r| r.events).unwrap_or_default();
        self.active_bundle_path = Some(bundle.root().to_path_buf());
        let doc = bundle.project().clone();
        self.selected_clip_id = doc.clips.first().map(|c| c.id);
        self.selected_zoom_id = None;
        self.engine = CommandEngine::new(doc);
        self.active_bundle = Some(bundle);
        self.playhead_ms = 0;
        self.mode = SessionMode::Editing;
        self.last_status = "Recording loaded".into();
    }

    pub fn project(&self) -> &ProjectDoc { self.engine.project() }
    pub fn selected_source(&self) -> Option<&CaptureSource> {
        self.sources.get(self.selected_source_idx).or_else(|| self.sources.first())
    }
    pub fn cycle_source(&mut self) {
        if !self.sources.is_empty() {
            self.selected_source_idx = (self.selected_source_idx + 1) % self.sources.len();
        }
    }
    pub fn set_tab(&mut self, tab: RightPanelTab) { self.active_tab = tab; }
    pub fn undo(&mut self) -> bool { self.engine.undo() }
    pub fn redo(&mut self) -> bool { self.engine.redo() }
    pub fn can_undo(&self) -> bool { self.engine.can_undo() }
    pub fn can_redo(&self) -> bool { self.engine.can_redo() }

    pub fn apply_edit(&mut self, cmd: EditCommand) -> CoreResult<()> {
        self.engine.apply(cmd)?;
        if let Some(bundle) = self.active_bundle.as_mut() {
            bundle.set_project(self.engine.project().clone());
            let _ = bundle.save_project();
        }
        Ok(())
    }

    pub fn set_aspect_ratio(&mut self, ratio: AspectRatio) -> CoreResult<()> {
        let mut frame = self.project().frame.clone();
        frame.aspect_ratio = ratio;
        self.apply_edit(EditCommand::UpdateFrameStyle(frame))
    }

    pub fn cycle_aspect_ratio(&mut self) -> CoreResult<()> {
        let next = match self.project().frame.aspect_ratio {
            AspectRatio::Ratio16x9 => AspectRatio::Ratio9x16,
            AspectRatio::Ratio9x16 => AspectRatio::Ratio4x3,
            AspectRatio::Ratio4x3 => AspectRatio::Ratio1x1,
            AspectRatio::Ratio1x1 | AspectRatio::Source => AspectRatio::Ratio16x9,
        };
        self.set_aspect_ratio(next)
    }

    pub fn output_duration_ms(&self) -> u64 {
        TimelineMap::from_project(self.project()).output_duration_ms().max(1_000)
    }

    pub fn build_capture_config(&self) -> CaptureConfig {
        let (target, w, h) = match self.selected_source() {
            Some(s) if s.kind == CaptureSourceKind::Window => {
                (CaptureTarget::Window { window_id: s.id }, s.width.max(2) & !1, s.height.max(2) & !1)
            }
            Some(s) => {
                (CaptureTarget::Display { display_id: s.id }, s.width.max(2) & !1, s.height.max(2) & !1)
            }
            None => (CaptureTarget::Display { display_id: 1 }, 1920, 1080),
        };
        CaptureConfig {
            target,
            width: w,
            height: h,
            fps: 60,
            capture_system_audio: self.capture_system_audio,
            capture_microphone: false,
            capture_webcam: false,
            hide_os_cursor: self.hide_os_cursor,
        }
    }

    pub fn mark_recording_started(&mut self, bundle_path: PathBuf) {
        self.active_bundle_path = Some(bundle_path);
        self.mode = SessionMode::Recording;
        self.last_status = "Recording...".into();
    }

    pub fn mark_recording_stopped(&mut self, video_frames: u64, duration_ms: u64) {
        self.recorded_frames = video_frames;
        let events = self.active_bundle.as_ref().and_then(|b| b.read_telemetry_lenient().ok()).map(|r| r.events).unwrap_or_default();
        let mut doc = self.active_bundle.as_ref().map(|b| b.project().clone()).unwrap_or_else(|| self.project().clone());
        doc.source.duration_ms = duration_ms.max(500);
        if let Some(first) = doc.clips.first_mut() {
            first.source_start_ms = 0;
            first.source_end_ms = doc.source.duration_ms;
        }
        doc.zoom_regions = generate_auto_zooms(&events, doc.source.duration_ms);
        self.telemetry = events;
        self.selected_clip_id = doc.clips.first().map(|c| c.id);
        self.engine = CommandEngine::new(doc.clone());
        if let Some(bundle) = self.active_bundle.as_mut() {
            bundle.set_project(doc);
            let _ = bundle.save_project();
        }
        self.mode = SessionMode::Editing;
        self.last_status = format!("Saved {video_frames} frames ({:.1}s)", duration_ms as f64 / 1000.0);
    }

    pub fn next_bundle_path() -> PathBuf {
        let home = std::env::var_os("HOME").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("/tmp"));
        let ts = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
        home.join("Movies").join("Fovea").join(format!("recording-{ts}.fovea"))
    }

    pub fn next_export_path(&self) -> PathBuf {
        let home = std::env::var_os("HOME").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("/tmp"));
        let ts = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
        let ext = match self.export_format { ExportFormat::Mp4 => "mp4", ExportFormat::Gif => "gif" };
        home.join("Movies").join("Fovea").join("exports").join(format!("{}-{ts}.{ext}", self.project().title))
    }
}
