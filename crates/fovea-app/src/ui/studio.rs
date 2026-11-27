use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use gpui_kit::*;
use image::{Frame, RgbaImage};
use smallvec::smallvec;
use uuid::Uuid;

use fovea_capture::{CaptureBackend, MacOsCaptureBackend, PermissionKind};
use fovea_core::{
    AnnotationItem, AnnotationKind, BackgroundStyle, EditCommand, FoveaBundle, ZoomRegion,
};
use fovea_render::extract_video_frame;

use crate::state::StudioState;
use crate::ui::inspector::render_floating_inspector;
use crate::ui::presets::WALLPAPER_PRESETS;
use crate::ui::rail::render_left_rail;
use crate::ui::stage::render_stage;
use crate::ui::timeline::render_timeline;
use crate::ui::topbar::render_topbar;

pub struct StudioView {
    pub state: StudioState,
    pub preview_frame: Option<Arc<RenderImage>>,
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
            backend,
        }
    }

    pub fn refresh_permissions(&mut self, cx: &mut Context<Self>) {
        self.state.permissions = self.backend.check_permissions();
        cx.notify();
    }

    pub fn open_permission_settings(&mut self, kind: PermissionKind, cx: &mut Context<Self>) {
        let _ = self.backend.open_permission_settings(kind);
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
        self.state.selected_preset_idx = idx % WALLPAPER_PRESETS.len();
        let _ = self.state.apply_edit(EditCommand::UpdateFrameStyle(frame));
        cx.notify();
    }

    pub fn apply_next_preset(&mut self, cx: &mut Context<Self>) {
        let next = (self.state.selected_preset_idx + 1) % WALLPAPER_PRESETS.len();
        self.apply_preset_index(next, cx);
    }

    pub fn seek_to_ms(&mut self, ms: u64, cx: &mut Context<Self>) {
        let total = self.state.output_duration_ms();
        self.state.playhead_ms = ms.min(total);
        if let Some(ref bundle) = self.state.active_bundle {
            self.preview_frame = decode_render_image(&bundle.screen_video_path(), ms);
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
                cx.background_executor().timer(Duration::from_millis(120)).await;
                let keep_going = this
                    .update(cx, |view, cx| {
                        if !view.state.is_playing {
                            return false;
                        }
                        let total = view.state.output_duration_ms();
                        let next = view.state.playhead_ms + 120;
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

    pub fn add_demo_zoom(&mut self, cx: &mut Context<Self>) {
        let start = self.state.playhead_ms;
        let _ = self.state.apply_edit(EditCommand::AddZoomRegion(ZoomRegion {
            id: Uuid::new_v4(),
            start_ms: start,
            end_ms: start + 2_500,
            scale: 1.8,
            focus_x: 0.5,
            focus_y: 0.5,
            auto_generated: false,
        }));
        cx.notify();
    }

    pub fn add_demo_privacy_blur(&mut self, cx: &mut Context<Self>) {
        let start = self.state.playhead_ms;
        let _ = self
            .state
            .apply_edit(EditCommand::AddAnnotation(AnnotationItem {
                id: Uuid::new_v4(),
                start_ms: start,
                end_ms: start + 4_000,
                x: 0.65,
                y: 0.12,
                width: 0.25,
                height: 0.10,
                payload: AnnotationKind::PrivacyBlur {
                    blur_strength: 18.0,
                },
            }));
        cx.notify();
    }
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

fn decode_render_image(path: &std::path::Path, ms: u64) -> Option<Arc<RenderImage>> {
    let decoded = extract_video_frame(path, ms, 1280).ok()?;
    let buf = RgbaImage::from_raw(decoded.width, decoded.height, decoded.bgra)?;
    Some(Arc::new(RenderImage::new(smallvec![Frame::new(buf)])))
}

pub fn open_studio_editor_window(open_path: Option<PathBuf>, cx: &mut App) {
    let bounds = Bounds::centered(None, size(px(1360.0), px(840.0)), cx);
    let options = WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(bounds)),
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
