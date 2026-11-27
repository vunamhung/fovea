use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use fovea_capture::{CaptureConfig, CaptureSource, CaptureSourceKind, CaptureTarget, PermissionReport};
use fovea_core::{
    AspectRatio, CommandEngine, CoreResult, EditCommand, FoveaBundle, ProjectDoc, TimelineMap,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionMode {
    Idle,
    Recording,
    Editing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RightPanelTab {
    Inspector,
    AiChat,
}

pub struct StudioState {
    pub engine: CommandEngine,
    pub active_bundle: Option<FoveaBundle>,
    pub active_bundle_path: Option<PathBuf>,
    pub permissions: PermissionReport,
    pub sources: Vec<CaptureSource>,
    pub selected_source_idx: usize,
    pub mode: SessionMode,
    pub active_tab: RightPanelTab,
    pub show_permissions: bool,
    pub capture_system_audio: bool,
    pub hide_os_cursor: bool,
    pub capture_keystrokes: bool,
    pub last_status: String,
    pub recorded_frames: u64,
}

impl StudioState {
    pub fn new(permissions: PermissionReport, sources: Vec<CaptureSource>) -> Self {
        let (w, h) = sources
            .first()
            .map(|s| (s.width.max(2) & !1, s.height.max(2) & !1))
            .unwrap_or((1920, 1080));
        let show_permissions = !permissions.all_required_granted();
        let doc = ProjectDoc::new("Untitled Demo", w, h, 60);
        Self {
            engine: CommandEngine::new(doc),
            active_bundle: None,
            active_bundle_path: None,
            permissions,
            sources,
            selected_source_idx: 0,
            mode: SessionMode::Idle,
            active_tab: RightPanelTab::Inspector,
            show_permissions,
            capture_system_audio: true,
            hide_os_cursor: true,
            capture_keystrokes: true,
            last_status: "Ready to record".into(),
            recorded_frames: 0,
        }
    }

    pub fn load_bundle(&mut self, bundle: FoveaBundle) {
        self.active_bundle_path = Some(bundle.root().to_path_buf());
        self.engine = CommandEngine::new(bundle.project().clone());
        self.active_bundle = Some(bundle);
        self.mode = SessionMode::Editing;
        self.last_status = "Project loaded".into();
    }

    pub fn project(&self) -> &ProjectDoc {
        self.engine.project()
    }

    pub fn selected_source(&self) -> Option<&CaptureSource> {
        self.sources.get(self.selected_source_idx).or_else(|| self.sources.first())
    }

    pub fn select_source(&mut self, idx: usize) {
        if idx < self.sources.len() {
            self.selected_source_idx = idx;
        }
    }

    pub fn set_tab(&mut self, tab: RightPanelTab) {
        self.active_tab = tab;
    }

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

    pub fn undo(&mut self) -> bool {
        self.engine.undo()
    }

    pub fn redo(&mut self) -> bool {
        self.engine.redo()
    }

    pub fn can_undo(&self) -> bool {
        self.engine.can_undo()
    }

    pub fn can_redo(&self) -> bool {
        self.engine.can_redo()
    }

    pub fn output_duration_ms(&self) -> u64 {
        TimelineMap::from_project(self.project()).output_duration_ms()
    }

    pub fn build_capture_config(&self) -> CaptureConfig {
        let src = self.selected_source();
        let (target, w, h) = match src {
            Some(s) if s.kind == CaptureSourceKind::Window => (
                CaptureTarget::Window { window_id: s.id },
                (s.width.max(2) & !1),
                (s.height.max(2) & !1),
            ),
            Some(s) => (
                CaptureTarget::Display { display_id: s.id },
                (s.width.max(2) & !1),
                (s.height.max(2) & !1),
            ),
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
        self.last_status = "Recording live via ScreenCaptureKit...".into();
    }

    pub fn mark_recording_stopped(&mut self, video_frames: u64, duration_ms: u64) {
        self.recorded_frames = video_frames;
        let mut doc = self.project().clone();
        doc.source.duration_ms = duration_ms;
        if let Some(first) = doc.clips.first_mut() {
            first.source_start_ms = 0;
            first.source_end_ms = duration_ms;
        }
        self.engine = CommandEngine::new(doc.clone());
        if let Some(bundle) = self.active_bundle.as_mut() {
            bundle.set_project(doc);
            let _ = bundle.save_project();
        }
        self.mode = SessionMode::Editing;
        self.last_status = format!("Saved {video_frames} frames ({:.1}s)", duration_ms as f64 / 1000.0);
    }

    pub fn next_bundle_path() -> PathBuf {
        let home = std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("/tmp"));
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        home.join("Movies")
            .join("Fovea")
            .join(format!("fovea-{ts}.fovea"))
    }
}
