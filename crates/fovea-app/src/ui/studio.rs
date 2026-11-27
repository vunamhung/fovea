use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use gpui_kit::*;

use fovea_capture::{CaptureBackend, MacOsCaptureBackend, PermissionKind};
use fovea_core::{BackgroundStyle, EditCommand, FoveaBundle, TimelineMap};
use fovea_render::DecodedFrame;

use crate::state::StudioState;
use crate::ui::inspector::render_floating_inspector;
use crate::ui::presets::WALLPAPER_PRESETS;
use crate::ui::rail::render_left_rail;
use crate::ui::stage::render_stage;
use crate::ui::studio_actions;
use crate::ui::studio_io::{
    decode_render_image, export_project_bundle, pick_custom_wallpaper, reveal_path_in_finder,
    spawn_video_export,
};
use crate::ui::timeline::render_timeline;
use crate::ui::topbar::render_topbar;

pub struct StudioView {
    pub state: StudioState,
    pub preview_frame: Option<Arc<RenderImage>>,
    pub custom_bg_image: Option<Arc<RenderImage>>,
    pub custom_bg_decoded: Option<DecodedFrame>,
    last_decoded_ms: u64,
    decode_in_flight: bool,
    backend: MacOsCaptureBackend,
}

impl StudioView {
    pub fn new(open_path: Option<PathBuf>) -> Self {
        let backend = MacOsCaptureBackend::new();
        let perms = backend.check_permissions();
        let sources = if perms.all_required_granted() {
            backend.list_sources().unwrap_or_default()
        } else {
            Vec::new()
        };
        let mut state = StudioState::new(perms, sources);
        let mut preview_frame = None;
        if let Some(path) = open_path {
            match FoveaBundle::open(&path) {
                Ok(bundle) => {
                    preview_frame = decode_render_image(&bundle.screen_video_path(), 0);
                    state.load_bundle(bundle);
                }
                Err(err) => state.last_status = format!("Could not open project: {err}"),
            }
        }
        Self {
            state,
            preview_frame,
            custom_bg_image: None,
            custom_bg_decoded: None,
            last_decoded_ms: 0,
            decode_in_flight: false,
            backend,
        }
    }

    pub fn refresh_permissions(&mut self, cx: &mut Context<Self>) {
        self.state.permissions = self.backend.check_permissions();
        if self.state.permissions.all_required_granted() {
            self.state.sources = self.backend.list_sources().unwrap_or_default();
            self.state.last_status = "All required permissions granted".into();
        } else {
            self.state.last_status = "Toggle Fovea ON in System Settings, then reopen".into();
        }
        cx.notify();
    }

    pub fn open_permission_settings(&mut self, kind: PermissionKind, cx: &mut Context<Self>) {
        let _ = self.backend.open_permission_settings(kind);
        self.state.permissions = self.backend.check_permissions();
        self.state.last_status = format!("Requested {} — toggle Fovea ON in Settings", kind.label());
        cx.notify();
    }

    pub fn apply_preset_index(&mut self, idx: usize, cx: &mut Context<Self>) {
        let preset = WALLPAPER_PRESETS[idx % WALLPAPER_PRESETS.len()];
        let mut frame = self.state.project().frame.clone();
        frame.background = BackgroundStyle::Gradient {
            from_hex: preset.from_hex.into(),
            to_hex: preset.to_hex.into(),
            angle_deg: preset.angle_deg,
        };
        self.custom_bg_image = None;
        self.custom_bg_decoded = None;
        self.state.selected_preset_idx = idx % WALLPAPER_PRESETS.len();
        self.state.last_status = format!("Preset: {}", preset.name);
        let _ = self.state.apply_edit(EditCommand::UpdateFrameStyle(frame));
        cx.notify();
    }

    pub fn apply_next_preset(&mut self, cx: &mut Context<Self>) {
        let next = (self.state.selected_preset_idx + 1) % WALLPAPER_PRESETS.len();
        self.apply_preset_index(next, cx);
    }

    pub fn pick_custom_wallpaper(&mut self, cx: &mut Context<Self>) {
        pick_custom_wallpaper(cx);
    }

    pub fn start_video_export(&mut self, cx: &mut Context<Self>) {
        spawn_video_export(self, cx);
    }

    pub fn export_project_bundle(&mut self, cx: &mut Context<Self>) {
        export_project_bundle(&mut self.state);
        cx.notify();
    }

    pub fn reveal_last_export(&mut self, cx: &mut Context<Self>) {
        if let Some(ref p) = self.state.last_export_path {
            reveal_path_in_finder(p);
            self.state.last_status = "Opened export in Finder".into();
            cx.notify();
        }
    }

    pub fn seek_to_ms(&mut self, ms: u64, cx: &mut Context<Self>) {
        let total = self.state.output_duration_ms();
        self.state.playhead_ms = ms.min(total);
        if let Some(ref bundle) = self.state.active_bundle
            && !self.decode_in_flight
            && self.state.playhead_ms.abs_diff(self.last_decoded_ms) >= 120
        {
            let timeline = TimelineMap::from_project(self.state.project());
            let src_ms = timeline
                .map_output_to_source(self.state.playhead_ms)
                .map(|m| m.source_ms)
                .unwrap_or(self.state.playhead_ms);
            let video_path = bundle.screen_video_path();
            self.last_decoded_ms = self.state.playhead_ms;
            self.decode_in_flight = true;
            cx.spawn(async move |this, cx| {
                let frame = cx
                    .background_executor()
                    .spawn(async move { decode_render_image(&video_path, src_ms) })
                    .await;
                let _ = this.update(cx, |view, cx| {
                    view.decode_in_flight = false;
                    if frame.is_some() {
                        view.preview_frame = frame;
                        cx.notify();
                    }
                });
            })
            .detach();
        }
        cx.notify();
    }

    pub fn toggle_playback(&mut self, cx: &mut Context<Self>) {
        self.state.is_playing = !self.state.is_playing;
        cx.notify();
        if !self.state.is_playing {
            return;
        }
        cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor().timer(Duration::from_millis(33)).await;
                let keep_going = this
                    .update(cx, |view, cx| {
                        if !view.state.is_playing {
                            return false;
                        }
                        let total = view.state.output_duration_ms();
                        let next = view.state.playhead_ms + 33;
                        if next >= total {
                            view.state.playhead_ms = 0;
                            view.state.is_playing = false;
                            cx.notify();
                            return false;
                        }
                        view.seek_to_ms(next, cx);
                        true
                    })
                    .unwrap_or(false);
                if !keep_going {
                    break;
                }
            }
        })
        .detach();
    }

    pub fn split_clip_at_playhead(&mut self, cx: &mut Context<Self>) { studio_actions::split_clip_at_playhead(&mut self.state); cx.notify(); }
    pub fn delete_selected_clip(&mut self, cx: &mut Context<Self>) { studio_actions::delete_selected_clip(&mut self.state); cx.notify(); }
    pub fn set_selected_zoom_scale(&mut self, scale: f32, cx: &mut Context<Self>) { studio_actions::set_selected_zoom_scale(&mut self.state, scale); cx.notify(); }
    pub fn delete_selected_zoom(&mut self, cx: &mut Context<Self>) { studio_actions::delete_selected_zoom(&mut self.state); cx.notify(); }
    pub fn add_demo_zoom(&mut self, cx: &mut Context<Self>) { let ms = studio_actions::add_zoom_at_playhead(&mut self.state); self.seek_to_ms(ms, cx); }
    pub fn add_demo_privacy_blur(&mut self, cx: &mut Context<Self>) { studio_actions::add_privacy_blur_at_playhead(&mut self.state); cx.notify(); }
}

impl Render for StudioView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .bg(rgb(0xf8fafc))
            .text_color(rgb(0x0f172a))
            .flex()
            .flex_col()
            .child(render_topbar(self, cx))
            .child(
                div()
                    .flex_1()
                    .w_full()
                    .flex()
                    .overflow_hidden()
                    .child(render_left_rail(self, cx))
                    .child(render_floating_inspector(self, cx))
                    .child(render_stage(self, cx)),
            )
            .child(render_timeline(self, cx))
    }
}

pub fn open_studio_editor_window(open_path: Option<PathBuf>, cx: &mut App) {
    let bounds = Bounds::centered(None, size(px(1360.0), px(840.0)), cx);
    let options = WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(bounds)),
        app_owns_titlebar_drag: true,
        titlebar: Some(TitlebarOptions {
            title: Some("Fovea".into()),
            appears_transparent: true,
            traffic_light_position: Some(point(px(16.0), px(18.0))),
        }),
        ..Default::default()
    };
    let _ = gpui_kit::open_window(options, cx, move |_, cx| {
        cx.new(|_| StudioView::new(open_path.clone()))
    });
}
