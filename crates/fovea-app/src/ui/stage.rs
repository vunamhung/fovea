use gpui_kit::component::IconName;
use gpui_kit::*;

use fovea_core::AspectRatio;

use crate::state::SessionMode;
use crate::ui::studio::StudioView;

pub fn render_stage(view: &StudioView) -> impl IntoElement {
    let state = &view.state;
    let project = state.project();
    let ratio_label = match project.frame.aspect_ratio {
        AspectRatio::Ratio16x9 => "16:9",
        AspectRatio::Ratio9x16 => "9:16",
        AspectRatio::Ratio4x3 => "4:3",
        AspectRatio::Ratio1x1 => "1:1",
        AspectRatio::Source => "Source",
    };

    let (frame_w, frame_h) = match project.frame.aspect_ratio {
        AspectRatio::Ratio9x16 => (240.0, 420.0),
        AspectRatio::Ratio1x1 => (380.0, 380.0),
        AspectRatio::Ratio4x3 => (520.0, 390.0),
        _ => (640.0, 360.0),
    };

    let source_label = state
        .selected_source()
        .map(|s| format!("{} ({}×{})", s.name, s.width, s.height))
        .unwrap_or_else(|| "Primary Display (1920×1080)".into());

    let badge_text = match state.mode {
        SessionMode::Recording => "RECORDING LIVE",
        SessionMode::Editing => "PROJECT READY",
        SessionMode::Idle => "GPU STAGE PREVIEW",
    };
    let badge_bg = match state.mode {
        SessionMode::Recording => rgb(0xdc2626),
        SessionMode::Editing => rgb(0x059669),
        SessionMode::Idle => rgb(0x2563eb),
    };

    div()
        .flex_1()
        .h_full()
        .bg(rgb(0x0b0d12))
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap_4()
        .p_6()
        .child(
            div()
                .w(px(frame_w))
                .h(px(frame_h))
                .rounded(px(project.frame.corner_radius_px))
                .bg(rgb(0x161b26))
                .border_1()
                .border_color(rgb(0x2d3548))
                .shadow_lg()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .gap_3()
                .child(
                    div()
                        .px_3()
                        .py_1()
                        .rounded_full()
                        .bg(badge_bg)
                        .text_xs()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(rgb(0xffffff))
                        .child(badge_text),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .text_base()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(rgb(0xf8fafc))
                        .child(IconName::Frame)
                        .child(source_label),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(0x94a3b8))
                        .child(format!(
                            "Aspect {ratio_label} • {} fps • {} zoom regions • {} annotations",
                            project.source.fps,
                            project.zoom_regions.len(),
                            project.annotations.len()
                        )),
                ),
        )
        .child(
            div()
                .text_xs()
                .text_color(rgb(0x94a3b8))
                .child(state.last_status.clone()),
        )
}
