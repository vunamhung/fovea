use gpui_kit::component::IconName;
use gpui_kit::*;

use crate::ui::studio::StudioView;
use crate::ui::transport::{fmt_time, render_transport_bar};

pub fn render_timeline(view: &StudioView, cx: &mut Context<StudioView>) -> impl IntoElement {
    let project = view.state.project();
    let total_ms = view.state.output_duration_ms().max(1_000);
    let playhead_ms = view.state.playhead_ms.min(total_ms);
    let playhead_ratio = (playhead_ms as f32 / total_ms as f32).clamp(0.0, 1.0);
    let is_playing = view.state.is_playing;

    let zooms: Vec<(usize, u64, f32, f32, String, bool)> = project
        .zoom_regions
        .iter()
        .enumerate()
        .map(|(i, z)| {
            let left = (z.start_ms as f32 / total_ms as f32).clamp(0.0, 0.96);
            let width = ((z.end_ms.saturating_sub(z.start_ms)) as f32 / total_ms as f32)
                .clamp(0.04, 1.0 - left);
            let active = playhead_ms >= z.start_ms && playhead_ms <= z.end_ms;
            (i, z.start_ms, left, width, format!("{:.1}x", z.scale), active)
        })
        .collect();

    div()
        .w_full()
        .bg(rgb(0xffffff))
        .border_t_1()
        .border_color(rgb(0xebedf2))
        .flex()
        .flex_col()
        .child(render_transport_bar(playhead_ms, total_ms, is_playing, cx))
        .child(render_ruler(total_ms, playhead_ratio, cx))
        .child(
            div()
                .relative()
                .w_full()
                .px_3()
                .pb_3()
                .flex()
                .flex_col()
                .gap_2()
                .child(render_clip_track())
                .child(render_zoom_track(zooms, cx))
                .child(
                    div()
                        .absolute()
                        .top_0()
                        .bottom_0()
                        .left(relative(playhead_ratio))
                        .w(px(2.0))
                        .bg(rgb(0x2563eb)),
                ),
        )
}

fn render_clip_track() -> impl IntoElement {
    div()
        .h(px(36.0))
        .w_full()
        .rounded(px(9.0))
        .bg(linear_gradient(
            180.0,
            linear_color_stop(rgb(0xdbeafe), 0.0),
            linear_color_stop(rgb(0x93c5fd), 1.0),
        ))
        .border_1()
        .border_color(rgb(0x93c5fd))
        .flex()
        .items_center()
        .justify_center()
        .gap_1p5()
        .text_xs()
        .font_weight(FontWeight::MEDIUM)
        .text_color(rgb(0x1e3a8a))
        .child(IconName::Frame)
        .child("Clip")
}

fn render_zoom_track(
    zooms: Vec<(usize, u64, f32, f32, String, bool)>,
    cx: &mut Context<StudioView>,
) -> impl IntoElement {
    let mut track = div().relative().h(px(38.0)).w_full();
    for (i, start_ms, left, width, label, active) in zooms {
        track = track.child(
            div()
                .id(("zoom-pill", i))
                .absolute()
                .top_0()
                .bottom_0()
                .left(relative(left))
                .w(relative(width))
                .rounded(px(9.0))
                .bg(linear_gradient(
                    180.0,
                    linear_color_stop(rgb(0xf3e8ff), 0.0),
                    linear_color_stop(rgb(0xe9d5ff), 1.0),
                ))
                .border_1()
                .border_color(if active { rgb(0x7c3aed) } else { rgb(0xc084fc) })
                .cursor_pointer()
                .flex()
                .items_center()
                .justify_center()
                .gap_1()
                .text_xs()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(rgb(0x6b21a8))
                .child(IconName::Search)
                .child(label)
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.seek_to_ms(start_ms, cx);
                })),
        );
    }
    track
}

fn render_ruler(
    total_ms: u64,
    playhead_ratio: f32,
    cx: &mut Context<StudioView>,
) -> impl IntoElement {
    let ticks = 10_usize;
    let mut row = div()
        .relative()
        .h(px(26.0))
        .w_full()
        .px_3()
        .flex()
        .items_center()
        .justify_between();
    for i in 0..=ticks {
        let ms = (total_ms * i as u64) / ticks as u64;
        row = row.child(
            div()
                .id(("ruler-tick", i))
                .cursor_pointer()
                .text_xs()
                .text_color(rgb(0x94a3b8))
                .child(fmt_time(ms))
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.seek_to_ms(ms, cx);
                })),
        );
    }
    row.child(
        div()
            .absolute()
            .top(px(2.0))
            .left(relative(playhead_ratio))
            .w(px(8.0))
            .h(px(10.0))
            .rounded_sm()
            .bg(rgb(0x2563eb)),
    )
}
