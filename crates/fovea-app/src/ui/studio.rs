use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;
use uuid::Uuid;

use fovea_capture::{
    CaptureBackend, MacOsCaptureBackend, PermissionKind, TelemetryTapConfig, TelemetryTapHandle,
};
use fovea_core::{AnnotationItem, AnnotationKind, EditCommand, FoveaBundle, ZoomRegion};

use crate::state::{SessionMode, StudioState};
use crate::ui::inspector::render_inspector;
use crate::ui::permissions::render_permissions_banner;
use crate::ui::stage::render_stage;
use crate::ui::timeline::render_timeline;
use crate::ui::topbar::render_topbar;

pub struct StudioView {
    pub state: StudioState,
    backend: MacOsCaptureBackend,
    telemetry_tap: Option<TelemetryTapHandle>,
}

impl StudioView {
    pub fn new(open_path: Option<std::path::PathBuf>) -> Self {
        let backend = MacOsCaptureBackend::new();
        let perms = backend.check_permissions();
        let sources = if perms.all_required_granted() {
            backend.list_sources().unwrap_or_default()
        } else {
            Vec::new()
        };
        let mut state = StudioState::new(perms, sources);
        if let Some(path) = open_path {
            match FoveaBundle::open(&path) {
                Ok(bundle) => state.load_bundle(bundle),
                Err(err) => state.last_status = format!("Could not open project: {err}"),
            }
        }
        Self {
            state,
            backend,
            telemetry_tap: None,
        }
    }

    pub fn refresh_permissions(&mut self, cx: &mut Context<Self>) {
        self.state.permissions = self.backend.check_permissions();
        if self.state.permissions.all_required_granted() && self.state.sources.is_empty() {
            self.state.sources = self.backend.list_sources().unwrap_or_default();
        }
        cx.notify();
    }

    pub fn refresh_sources(&mut self, cx: &mut Context<Self>) {
        match self.backend.list_sources() {
            Ok(list) => {
                self.state.sources = list;
                self.state.selected_source_idx = 0;
                self.state.last_status = format!("Found {} capture source(s)", self.state.sources.len());
            }
            Err(err) => {
                self.state.last_status = format!("Source refresh failed: {err}");
            }
        }
        cx.notify();
    }

    pub fn open_permission_settings(&mut self, kind: PermissionKind, cx: &mut Context<Self>) {
        if let Err(err) = self.backend.open_permission_settings(kind) {
            self.state.last_status = format!("Failed to open settings: {err}");
        }
        cx.notify();
    }

    pub fn toggle_recording(&mut self, cx: &mut Context<Self>) {
        if self.state.mode == SessionMode::Recording {
            self.stop_recording(cx);
        } else {
            self.start_recording(cx);
        }
    }

    fn start_recording(&mut self, cx: &mut Context<Self>) {
        let cfg = self.state.build_capture_config();
        let bundle_path = StudioState::next_bundle_path();
        let bundle = match FoveaBundle::create(
            &bundle_path,
            self.state.project().title.clone(),
            cfg.width,
            cfg.height,
            cfg.fps,
        ) {
            Ok(b) => b,
            Err(err) => {
                self.state.last_status = format!("Failed to create bundle: {err}");
                cx.notify();
                return;
            }
        };

        if let Err(err) = self.backend.start(cfg.clone(), &bundle) {
            self.state.show_permissions = true;
            self.state.last_status = format!("Start capture failed: {err}");
            cx.notify();
            return;
        }

        let tap_cfg = TelemetryTapConfig {
            capture_keystrokes: self.state.capture_keystrokes,
            shortcuts_only: self.state.project().keystroke_hud.shortcuts_only,
            capture_ax_context: true,
            bounds_x: 0.0,
            bounds_y: 0.0,
            bounds_w: cfg.width as f64,
            bounds_h: cfg.height as f64,
        };
        self.telemetry_tap = TelemetryTapHandle::start(tap_cfg, &bundle).ok();
        self.state.active_bundle = Some(bundle);
        self.state.mark_recording_started(bundle_path);
        cx.notify();
    }

    fn stop_recording(&mut self, cx: &mut Context<Self>) {
        if let Some(mut tap) = self.telemetry_tap.take() {
            let _ = tap.stop();
        }
        match self.backend.stop() {
            Ok(stats) => {
                self.state
                    .mark_recording_stopped(stats.video_frames, stats.duration_ms);
            }
            Err(err) => {
                self.state.mode = SessionMode::Idle;
                self.state.last_status = format!("Stop capture error: {err}");
            }
        }
        cx.notify();
    }

    pub fn add_demo_zoom(&mut self, cx: &mut Context<Self>) {
        let _ = self.state.apply_edit(EditCommand::AddZoomRegion(ZoomRegion {
            id: Uuid::new_v4(),
            start_ms: 1_000,
            end_ms: 3_500,
            scale: 2.0,
            focus_x: 0.5,
            focus_y: 0.5,
            auto_generated: true,
        }));
        self.state.last_status = "Added 2.0x spring zoom region".into();
        cx.notify();
    }

    pub fn add_demo_privacy_blur(&mut self, cx: &mut Context<Self>) {
        let _ = self
            .state
            .apply_edit(EditCommand::AddAnnotation(AnnotationItem {
                id: Uuid::new_v4(),
                start_ms: 0,
                end_ms: 5_000,
                x: 0.65,
                y: 0.1,
                width: 0.28,
                height: 0.12,
                payload: AnnotationKind::PrivacyBlur {
                    blur_strength: 16.0,
                },
            }));
        self.state.last_status = "Added Privacy Blur redaction region".into();
        cx.notify();
    }
}

impl Render for StudioView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let show_perms = self.state.show_permissions;
        div()
            .size_full()
            .bg(rgb(0x0b0d12))
            .text_color(rgb(0xf8fafc))
            .flex()
            .flex_col()
            .child(render_topbar(self, cx))
            .when(show_perms, |el| el.child(render_permissions_banner(self, cx)))
            .child(
                div()
                    .flex_1()
                    .w_full()
                    .flex()
                    .child(render_stage(self))
                    .child(render_inspector(self, cx)),
            )
            .child(render_timeline(self, cx))
    }
}
