use gpui_kit::component::IconName;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use crate::ui::studio::StudioView;

pub fn render_stage_toolbar(
    ratio_label: &'static str,
    crop_mode: bool,
    active_zoom: Option<f32>,
    cx: &mut Context<StudioView>,
) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .gap_2p5()
        .child(
            div()
                .id("stage-aspect-cycle")
                .px_2p5()
                .py_1()
                .rounded(px(7.0))
                .bg(rgb(0xffffff))
                .border_1()
                .border_color(rgb(0xe5e7eb))
                .cursor_pointer()
                .flex()
                .items_center()
                .gap_1()
                .text_xs()
                .font_weight(FontWeight::MEDIUM)
                .text_color(rgb(0x334155))
                .child(ratio_label)
                .child(IconName::ChevronDown)
                .on_click(cx.listener(|this, _, _, cx| {
                    let _ = this.state.cycle_aspect_ratio();
                    cx.notify();
                })),
        )
        .child(
            div()
                .id("stage-crop-btn")
                .px_2p5()
                .py_1()
                .rounded(px(7.0))
                .bg(if crop_mode { rgb(0xeff6ff) } else { rgb(0xffffff) })
                .border_1()
                .border_color(if crop_mode { rgb(0x2563eb) } else { rgb(0xe5e7eb) })
                .cursor_pointer()
                .flex()
                .items_center()
                .gap_1p5()
                .text_xs()
                .font_weight(FontWeight::MEDIUM)
                .text_color(if crop_mode { rgb(0x1d4ed8) } else { rgb(0x334155) })
                .child(IconName::Frame)
                .child(if crop_mode { "Done Crop" } else { "Crop Video" })
                .on_click(cx.listener(|this, _, _, cx| {
                    this.state.crop_mode = !this.state.crop_mode;
                    this.state.last_status = if this.state.crop_mode {
                        "Crop frame active".into()
                    } else {
                        "Ready".into()
                    };
                    cx.notify();
                })),
        )
        .when_some(active_zoom, |d, scale| {
            d.child(
                div()
                    .px_2()
                    .py_1()
                    .rounded(px(7.0))
                    .bg(rgb(0xf3e8ff))
                    .border_1()
                    .border_color(rgb(0xd8b4fe))
                    .text_xs()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(rgb(0x6b21a8))
                    .child(format!("Zoom {scale:.2}x")),
            )
        })
}

pub fn render_webcam_bubble(cam_sz: f32) -> impl IntoElement {
    div()
        .absolute()
        .right_4()
        .bottom_4()
        .w(px(cam_sz))
        .h(px(cam_sz))
        .rounded_full()
        .bg(linear_gradient(
            135.0,
            linear_color_stop(rgb(0x1e293b), 0.0),
            linear_color_stop(rgb(0x334155), 1.0),
        ))
        .border_2()
        .border_color(rgb(0xffffff))
        .shadow_md()
        .flex()
        .items_center()
        .justify_center()
        .text_xs()
        .text_color(rgb(0x94a3b8))
        .child("CAM")
}

pub fn render_sample_macos_window(zoom_scale: f32) -> impl IntoElement {
    let z = zoom_scale.clamp(1.0, 3.0);
    div()
        .size_full()
        .bg(rgb(0xffffff))
        .flex()
        .flex_col()
        .child(
            div()
                .h(px(30.0 * z))
                .px(px(12.0 * z))
                .bg(rgb(0xf6f7f9))
                .border_b_1()
                .border_color(rgb(0xe5e7eb))
                .flex()
                .items_center()
                .justify_between()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(6.0 * z))
                        .child(div().w(px(9.0 * z)).h(px(9.0 * z)).rounded_full().bg(rgb(0xff5f57)))
                        .child(div().w(px(9.0 * z)).h(px(9.0 * z)).rounded_full().bg(rgb(0xfebc2e)))
                        .child(div().w(px(9.0 * z)).h(px(9.0 * z)).rounded_full().bg(rgb(0x28c840))),
                )
                .child(
                    div()
                        .text_size(px(12.0 * z))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(rgb(0x475569))
                        .child(format!("Activity Monitor — Fovea ({z:.2}x)")),
                )
                .child(div().w(px(36.0 * z))),
        )
        .child(
            div()
                .flex_1()
                .p(px(16.0 * z))
                .flex()
                .flex_col()
                .gap(px(8.0 * z))
                .child(sample_row("fovea-app (Pure Rust + GPUI)", "14 MB", "0.4% CPU", true, z))
                .child(sample_row("ScreenCaptureKit Stream", "60 fps", "Zero-Copy", false, z))
                .child(sample_row("VideoToolbox H.264 / HEVC", "Hardware", "Ready", false, z))
                .child(sample_row("Vbee Neural Voiceover", "vi-VN / en-US", "Auto-Duck", false, z))
                .child(sample_row("Claude / AGY Live MCP", "live.sock", "1-Click Undo", false, z)),
        )
}

fn sample_row(
    name: &'static str,
    col1: &'static str,
    col2: &'static str,
    active: bool,
    z: f32,
) -> impl IntoElement {
    div()
        .h(px(30.0 * z))
        .px(px(12.0 * z))
        .rounded(px(6.0 * z))
        .bg(if active { rgb(0x2563eb) } else { rgb(0xf8fafc) })
        .flex()
        .items_center()
        .justify_between()
        .text_size(px(12.0 * z))
        .text_color(if active { rgb(0xffffff) } else { rgb(0x334155) })
        .child(div().font_weight(FontWeight::MEDIUM).child(name))
        .child(div().flex().gap(px(24.0 * z)).child(col1).child(col2))
}
