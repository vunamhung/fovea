use gpui_kit::component::button::Button;
use gpui_kit::component::IconName;
use gpui_kit::*;

use crate::ui::studio::StudioView;

pub fn render_timeline(view: &StudioView, cx: &mut Context<StudioView>) -> impl IntoElement {
    let project = view.state.project();
    let duration_sec = view.state.output_duration_ms() as f64 / 1000.0;
    let clips_count = project.clips.len();
    let zoom_count = project.zoom_regions.len();
    let overlay_count = project.annotations.len() + project.captions.len();
    let voice_count = project.voice_clips.len();

    div()
        .h(px(195.0))
        .w_full()
        .bg(rgb(0x101218))
        .border_t_1()
        .border_color(rgb(0x232734))
        .flex()
        .flex_col()
        .child(
            div()
                .h(px(40.0))
                .px_4()
                .border_b_1()
                .border_color(rgb(0x1e2230))
                .flex()
                .items_center()
                .justify_between()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_3()
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(rgb(0xe2e8f0))
                                .child(format!("00:00.0 / {duration_sec:05.1}s")),
                        )
                        .child(
                            Button::new("tl-add-zoom")
                                .outline()
                                .icon(IconName::Plus)
                                .label("Zoom")
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.add_demo_zoom(cx);
                                })),
                        )
                        .child(
                            Button::new("tl-add-blur")
                                .outline()
                                .icon(IconName::EyeOff)
                                .label("Privacy Blur")
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.add_demo_privacy_blur(cx);
                                })),
                        ),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(0x64748b))
                        .child("Single-take non-destructive timeline"),
                ),
        )
        .child(
            div()
                .flex_1()
                .px_4()
                .py_2()
                .flex()
                .flex_col()
                .gap_1p5()
                .child(track_row("Video", format!("{clips_count} clip segment(s)"), rgb(0x1d4ed8)))
                .child(track_row("Zoom", format!("{zoom_count} spring zoom region(s)"), rgb(0x7c3aed)))
                .child(track_row("Overlays", format!("{overlay_count} blur / text / caption item(s)"), rgb(0x0f766e)))
                .child(track_row("Audio / Vbee", format!("{voice_count} voiceover track(s) • auto-duck on"), rgb(0xb45309))),
        )
}

fn track_row(label: &'static str, summary: String, accent: Rgba) -> impl IntoElement {
    div()
        .h(px(30.0))
        .w_full()
        .flex()
        .items_center()
        .gap_3()
        .child(
            div()
                .w(px(96.0))
                .text_xs()
                .font_weight(FontWeight::MEDIUM)
                .text_color(rgb(0x94a3b8))
                .child(label),
        )
        .child(
            div()
                .flex_1()
                .h_full()
                .px_3()
                .rounded_md()
                .bg(rgb(0x171b24))
                .border_l_4()
                .border_color(accent)
                .flex()
                .items_center()
                .text_xs()
                .text_color(rgb(0xcbd5e1))
                .child(summary),
        )
}
