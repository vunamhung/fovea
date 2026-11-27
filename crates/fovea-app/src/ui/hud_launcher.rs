use std::path::PathBuf;
use std::time::{Duration, Instant};

use gpui_kit::*;

use fovea_capture::{
    CaptureBackend, MacOsCaptureBackend, PermissionKind, TelemetryTapConfig, TelemetryTapHandle,
};
use fovea_core::FoveaBundle;

use crate::state::{SessionMode, StudioState};
use crate::ui::countdown::CountdownOverlay;
use crate::ui::hud_bar::render_hud_pill;
use crate::ui::studio::open_studio_editor_window;

pub struct HudLauncherView {
    pub state: StudioState,
    backend: MacOsCaptureBackend,
    telemetry_tap: Option<TelemetryTapHandle>,
    pub counting_down: bool,
    started_at: Option<Instant>,
    pub elapsed_sec: u64,
}

impl HudLauncherView {
    pub fn new() -> Self {
        let backend = MacOsCaptureBackend::new();
        let perms = backend.check_permissions();
        let sources = if perms.all_required_granted() {
            backend.list_sources().unwrap_or_default()
        } else {
            Vec::new()
        };
        Self {
            state: StudioState::new(perms, sources),
            backend,
            telemetry_tap: None,
            counting_down: false,
            started_at: None,
            elapsed_sec: 0,
        }
    }

    pub fn refresh_sources(&mut self, cx: &mut Context<Self>) {
        self.state.permissions = self.backend.check_permissions();
        if self.state.permissions.all_required_granted() {
            self.state.sources = self.backend.list_sources().unwrap_or_default();
            self.state.selected_source_idx = 0;
            self.state.last_status = format!("{} capture sources ready", self.state.sources.len());
        } else {
            self.state.last_status = "Grant Screen Recording first".into();
        }
        cx.notify();
    }

    pub fn start_countdown_and_record(&mut self, cx: &mut Context<Self>) {
        if self.counting_down || self.state.mode == SessionMode::Recording {
            return;
        }
        self.state.permissions = self.backend.check_permissions();
        if !self.state.permissions.all_required_granted() {
            self.request_screen_permission(cx);
            return;
        }
        if self.state.sources.is_empty() {
            self.state.sources = self.backend.list_sources().unwrap_or_default();
        }
        if !self.state.countdown_enabled {
            self.begin_live_capture(cx);
            return;
        }
        self.counting_down = true;
        self.state.last_status = "Ready".into();
        cx.notify();

        let bounds = Bounds::centered(None, size(px(164.0), px(164.0)), cx);
        let overlay_win = cx
            .open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    titlebar: None,
                    focus: false,
                    show: true,
                    kind: WindowKind::PopUp,
                    is_movable: false,
                    is_resizable: false,
                    window_background: WindowBackgroundAppearance::Transparent,
                    ..Default::default()
                },
                |_, cx| cx.new(|_| CountdownOverlay::new(3)),
            )
            .ok();

        cx.spawn(async move |this, cx| {
            for step in [3_u8, 2, 1] {
                if let Some(ref handle) = overlay_win {
                    let _ = handle.update(cx, |v, _, cx| { v.count = step; cx.notify(); });
                }
                cx.background_executor().timer(Duration::from_secs(1)).await;
            }
            if let Some(handle) = overlay_win {
                let _ = handle.update(cx, |_, win, _| win.remove_window());
            }
            let _ = this.update(cx, |v, cx| { v.counting_down = false; v.begin_live_capture(cx); });
        })
        .detach();
    }

    fn begin_live_capture(&mut self, cx: &mut Context<Self>) {
        let cfg = self.state.build_capture_config();
        let bundle_path = StudioState::next_bundle_path();
        let title = bundle_path.file_stem().and_then(|s| s.to_str()).unwrap_or("recording");
        let Ok(bundle) = FoveaBundle::create(&bundle_path, title, cfg.width, cfg.height, cfg.fps) else {
            self.state.last_status = "Could not create .fovea bundle".into();
            cx.notify();
            return;
        };
        if let Err(err) = self.backend.start(cfg.clone(), &bundle) {
            self.state.last_status = format!("Capture error: {err}");
            cx.notify();
            return;
        }
        let tap_cfg = TelemetryTapConfig {
            capture_keystrokes: self.state.capture_keystrokes,
            shortcuts_only: true,
            capture_ax_context: true,
            bounds_x: 0.0, bounds_y: 0.0,
            bounds_w: cfg.width as f64, bounds_h: cfg.height as f64,
        };
        self.telemetry_tap = TelemetryTapHandle::start(tap_cfg, &bundle).ok();
        self.state.active_bundle = Some(bundle);
        self.state.mark_recording_started(bundle_path);
        self.started_at = Some(Instant::now());
        self.elapsed_sec = 0;
        cx.notify();

        cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor().timer(Duration::from_millis(250)).await;
                let keep = this.update(cx, |v, cx| {
                    if v.state.mode != SessionMode::Recording { return false; }
                    if let Some(s) = v.started_at { v.elapsed_sec = s.elapsed().as_secs(); cx.notify(); }
                    true
                }).unwrap_or(false);
                if !keep { break; }
            }
        })
        .detach();
    }

    pub fn request_screen_permission(&mut self, cx: &mut Context<Self>) {
        let _ = self
            .backend
            .open_permission_settings(PermissionKind::ScreenRecording);
        self.state.permissions = self.backend.check_permissions();
        if self.state.permissions.all_required_granted() {
            self.state.sources = self.backend.list_sources().unwrap_or_default();
            self.state.last_status = "Ready".into();
        } else {
            self.state.last_status = "Toggle Fovea ON in Screen Recording, then reopen".into();
        }
        cx.notify();
    }

    pub fn cancel_recording(&mut self, cx: &mut Context<Self>) {
        if let Some(mut tap) = self.telemetry_tap.take() {
            let _ = tap.stop();
        }
        let _ = self.backend.stop();
        if let Some(path) = self.state.active_bundle_path.take() {
            let _ = std::fs::remove_dir_all(path);
        }
        self.state.active_bundle = None;
        self.state.mode = SessionMode::Idle;
        self.started_at = None;
        self.state.last_status = "Recording cancelled".into();
        cx.notify();
    }

    pub fn stop_and_open_editor(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(mut tap) = self.telemetry_tap.take() {
            let _ = tap.stop();
        }
        let path: Option<PathBuf> = match self.backend.stop() {
            Ok(stats) if stats.video_frames > 0 => {
                self.state
                    .mark_recording_stopped(stats.video_frames, stats.duration_ms);
                self.state.active_bundle_path.clone()
            }
            Ok(_) => {
                self.state.mode = SessionMode::Idle;
                self.state.last_status = "0 video frames captured — check Screen Recording".into();
                cx.notify();
                return;
            }
            Err(e) => {
                self.state.mode = SessionMode::Idle;
                self.state.last_status = format!("Stop error: {e}");
                cx.notify();
                return;
            }
        };
        open_studio_editor_window(path, cx);
        window.remove_window();
    }
}

impl Default for HudLauncherView {
    fn default() -> Self {
        Self::new()
    }
}

impl Render for HudLauncherView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        render_hud_pill(self, cx)
    }
}

pub fn open_hud_launcher_window(cx: &mut App) {
    let display_bounds = cx
        .primary_display()
        .map(|d| d.bounds())
        .unwrap_or_else(|| Bounds::new(point(px(0.0), px(0.0)), size(px(1440.0), px(900.0))));
    let hud_w = px(660.0);
    let hud_h = px(96.0);
    let x = display_bounds.origin.x + (display_bounds.size.width - hud_w) * 0.5;
    let y = display_bounds.origin.y + display_bounds.size.height - hud_h - px(44.0);
    let bounds = Bounds::new(point(x, y), size(hud_w, hud_h));

    let _ = cx.open_window(
        WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            titlebar: None,
            focus: true,
            show: true,
            kind: WindowKind::Floating,
            is_movable: true,
            app_owns_titlebar_drag: true,
            is_resizable: false,
            window_background: WindowBackgroundAppearance::Transparent,
            ..Default::default()
        },
        |_, cx| cx.new(|_| HudLauncherView::new()),
    );
}
