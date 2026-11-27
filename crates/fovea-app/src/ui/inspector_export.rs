use gpui_kit::component::IconName;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use fovea_render::ExportFormat;

use crate::ui::inspector_system::section_header;
use crate::ui::studio::StudioView;

pub fn render_export_tab(view: &StudioView, cx: &mut Context<StudioView>) -> impl IntoElement {
    let fmt = view.state.export_format;
    let height = view.state.export_height;
    let fps = view.state.export_fps;
    let progress = view.state.export_progress;
    let exporting = progress.is_some();
    let has_last = view.state.last_export_path.is_some();

    div()
        .flex()
        .flex_col()
        .gap_3()
        .child(section_header("EXPORT VIDEO"))
        .child(
            div()
                .flex()
                .gap_1p5()
                .child(opt_btn("exp-mp4", "MP4 (H.264)", fmt == ExportFormat::Mp4, cx.listener(|this, _, _, cx| {
                    this.state.export_format = ExportFormat::Mp4;
                    cx.notify();
                })))
                .child(opt_btn("exp-gif", "Animated GIF", fmt == ExportFormat::Gif, cx.listener(|this, _, _, cx| {
                    this.state.export_format = ExportFormat::Gif;
                    cx.notify();
                }))),
        )
        .child(section_header("RESOLUTION"))
        .child(
            div()
                .flex()
                .gap_1p5()
                .child(opt_btn("exp-1080", "1080p HD", height == 1080, cx.listener(|this, _, _, cx| {
                    this.state.export_height = 1080;
                    cx.notify();
                })))
                .child(opt_btn("exp-720", "720p Fast", height == 720, cx.listener(|this, _, _, cx| {
                    this.state.export_height = 720;
                    cx.notify();
                }))),
        )
        .child(section_header("FRAME RATE"))
        .child(
            div()
                .flex()
                .gap_1p5()
                .child(opt_btn("exp-30", "30 fps", fps == 30, cx.listener(|this, _, _, cx| {
                    this.state.export_fps = 30;
                    cx.notify();
                })))
                .child(opt_btn("exp-60", "60 fps", fps == 60, cx.listener(|this, _, _, cx| {
                    this.state.export_fps = 60;
                    cx.notify();
                }))),
        )
        .when_some(progress, |d, p| {
            let pct = (p * 100.0).round() as u32;
            d.child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(rgb(0x2563eb))
                            .child(format!("Rendering video... {pct}%")),
                    )
                    .child(
                        div()
                            .h(px(8.0))
                            .w_full()
                            .rounded_full()
                            .bg(rgb(0xe2e8f0))
                            .overflow_hidden()
                            .child(div().h_full().w(relative(p.clamp(0.02, 1.0))).bg(rgb(0x2563eb))),
                    ),
            )
        })
        .child(
            div()
                .id("exp-start")
                .h(px(36.0))
                .w_full()
                .rounded(px(10.0))
                .bg(if exporting { rgb(0x94a3b8) } else { rgb(0x2563eb) })
                .flex()
                .items_center()
                .justify_center()
                .gap_1p5()
                .cursor_pointer()
                .text_xs()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(rgb(0xffffff))
                .child(IconName::ExternalLink)
                .child(if exporting {
                    "Exporting..."
                } else if fmt == ExportFormat::Mp4 {
                    "Export MP4 (VideoToolbox)"
                } else {
                    "Export GIF"
                })
                .on_click(cx.listener(|this, _, _, cx| {
                    this.start_video_export(cx);
                })),
        )
        .when(has_last, |d| {
            d.child(
                div()
                    .id("exp-reveal")
                    .h(px(32.0))
                    .w_full()
                    .rounded(px(9.0))
                    .bg(rgb(0xf0fdf4))
                    .border_1()
                    .border_color(rgb(0x86efac))
                    .flex()
                    .items_center()
                    .justify_center()
                    .gap_1p5()
                    .cursor_pointer()
                    .text_xs()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(rgb(0x15803d))
                    .child(IconName::FolderOpen)
                    .child("Reveal in Finder")
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.reveal_last_export(cx);
                    })),
            )
        })
        .child(
            div()
                .id("exp-save-proj")
                .h(px(30.0))
                .w_full()
                .rounded(px(8.0))
                .bg(rgb(0xf8fafc))
                .border_1()
                .border_color(rgb(0xe2e8f0))
                .flex()
                .items_center()
                .justify_center()
                .cursor_pointer()
                .text_xs()
                .font_weight(FontWeight::MEDIUM)
                .text_color(rgb(0x334155))
                .child("Save .fovea Project")
                .on_click(cx.listener(|this, _, _, cx| {
                    this.export_project_bundle(cx);
                })),
        )
}

fn opt_btn(
    id: &'static str,
    label: &'static str,
    active: bool,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    div()
        .id(id)
        .flex_1()
        .h(px(30.0))
        .rounded(px(8.0))
        .bg(if active { rgb(0x2563eb) } else { rgb(0xf1f5f9) })
        .flex()
        .items_center()
        .justify_center()
        .cursor_pointer()
        .text_xs()
        .font_weight(FontWeight::MEDIUM)
        .text_color(if active { rgb(0xffffff) } else { rgb(0x475569) })
        .child(label)
        .on_click(on_click)
}
