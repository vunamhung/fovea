use gpui_kit::component::IconName;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use fovea_core::AspectRatio;

use crate::ui::presets::background_to_gpui;
use crate::ui::studio::StudioView;

pub fn render_stage(view: &StudioView, cx: &mut Context<StudioView>) -> impl IntoElement {
    let project = view.state.project();
    let frame = &project.frame;
    let playhead_ms = view.state.playhead_ms;
    let active_zoom = project
        .zoom_regions
        .iter()
        .find(|z| playhead_ms >= z.start_ms && playhead_ms <= z.end_ms)
        .map(|z| z.scale);

    let ratio_label = match frame.aspect_ratio {
        AspectRatio::Ratio16x9 => "16:9",
        AspectRatio::Ratio9x16 => "9:16",
        AspectRatio::Ratio4x3 => "4:3",
        AspectRatio::Ratio1x1 => "1:1",
        AspectRatio::Source => "Source",
    };

    let (canvas_w, canvas_h) = match frame.aspect_ratio {
        AspectRatio::Ratio9x16 => (280.0, 460.0),
        AspectRatio::Ratio1x1 => (440.0, 440.0),
        AspectRatio::Ratio4x3 => (600.0, 450.0),
        _ => (780.0, 438.0),
    };

    let pad_px = if view.state.remove_background {
        0.0
    } else {
        (canvas_h * frame.padding_ratio.clamp(0.0, 0.30)).round()
    };
    let shadow_alpha = (frame.shadow_intensity.clamp(0.0, 1.0) * 0.42).max(0.02);
    let shadow_blur = px(12.0 + frame.shadow_intensity * 36.0);
    let shadow_y = px(4.0 + frame.shadow_intensity * 16.0);
    let bg_fill = if view.state.remove_background {
        rgb(0xf1f5f9).into()
    } else {
        background_to_gpui(&frame.background)
    };

    div()
        .flex_1()
        .h_full()
        .bg(rgb(0xf8fafc))
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap_3()
        .px_5()
        .py_3()
        .child(render_stage_toolbar(ratio_label, active_zoom, cx))
        .child(
            div()
                .w(px(canvas_w))
                .h(px(canvas_h))
                .rounded(px(12.0))
                .bg(bg_fill)
                .p(px(pad_px))
                .overflow_hidden()
                .flex()
                .items_center()
                .justify_center()
                .child(
                    div()
                        .size_full()
                        .rounded(px(frame.corner_radius_px))
                        .overflow_hidden()
                        .bg(rgb(0xffffff))
                        .shadow(vec![BoxShadow {
                            color: hsla(0.0, 0.0, 0.0, shadow_alpha),
                            offset: point(px(0.0), shadow_y),
                            blur_radius: shadow_blur,
                            spread_radius: px(0.0),
                            inset: false,
                        }])
                        .child(if let Some(ref frame_img) = view.preview_frame {
                            img(frame_img.clone())
                                .size_full()
                                .object_fit(ObjectFit::Cover)
                                .into_any_element()
                        } else {
                            render_sample_macos_window().into_any_element()
                        }),
                ),
        )
}

fn render_stage_toolbar(
    ratio_label: &'static str,
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
                .bg(rgb(0xffffff))
                .border_1()
                .border_color(rgb(0xe5e7eb))
                .cursor_pointer()
                .flex()
                .items_center()
                .gap_1p5()
                .text_xs()
                .font_weight(FontWeight::MEDIUM)
                .text_color(rgb(0x334155))
                .child(IconName::Frame)
                .child("Crop Video")
                .on_click(cx.listener(|this, _, _, cx| {
                    this.state.last_status =
                        "Interactive stage crop handles arrive in Milestone 2".into();
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
                    .child(format!("Zoom {scale:.1}x")),
            )
        })
}

fn render_sample_macos_window() -> impl IntoElement {
    div()
        .size_full()
        .bg(rgb(0xffffff))
        .flex()
        .flex_col()
        .child(
            div()
                .h(px(30.0))
                .px_3()
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
                        .gap_1p5()
                        .child(div().w(px(9.0)).h(px(9.0)).rounded_full().bg(rgb(0xff5f57)))
                        .child(div().w(px(9.0)).h(px(9.0)).rounded_full().bg(rgb(0xfebc2e)))
                        .child(div().w(px(9.0)).h(px(9.0)).rounded_full().bg(rgb(0x28c840))),
                )
                .child(
                    div()
                        .text_xs()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(rgb(0x475569))
                        .child("Activity Monitor — Fovea Native Engine"),
                )
                .child(div().w(px(36.0))),
        )
        .child(
            div()
                .flex_1()
                .p_4()
                .flex()
                .flex_col()
                .gap_2()
                .child(sample_row("fovea-app (Pure Rust + GPUI)", "14 MB", "0.4% CPU", true))
                .child(sample_row("ScreenCaptureKit Stream", "60 fps", "Zero-Copy", false))
                .child(sample_row("VideoToolbox H.264 / HEVC", "Hardware", "Ready", false))
                .child(sample_row("Vbee Neural Voiceover", "vi-VN / en-US", "Auto-Duck", false))
                .child(sample_row("Claude / AGY Live MCP", "live.sock", "1-Click Undo", false)),
        )
}

fn sample_row(name: &'static str, col1: &'static str, col2: &'static str, active: bool) -> impl IntoElement {
    div()
        .h(px(28.0))
        .px_3()
        .rounded(px(6.0))
        .bg(if active { rgb(0x2563eb) } else { rgb(0xf8fafc) })
        .flex()
        .items_center()
        .justify_between()
        .text_xs()
        .text_color(if active { rgb(0xffffff) } else { rgb(0x334155) })
        .child(div().font_weight(FontWeight::MEDIUM).child(name))
        .child(div().flex().gap_6().child(col1).child(col2))
}
