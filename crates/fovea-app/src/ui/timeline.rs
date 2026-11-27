use gpui_kit::component::IconName;
use gpui_kit::*;
use uuid::Uuid;

use crate::ui::studio::StudioView;
use crate::ui::transport::{fmt_time, render_transport_bar};

struct ZoomPill {
    idx: usize,
    id: Uuid,
    start_ms: u64,
    end_ms: u64,
    left: f32,
    width: f32,
    label: String,
    active: bool,
}

pub fn render_timeline(view: &StudioView, cx: &mut Context<StudioView>) -> impl IntoElement {
    let project = view.state.project();
    let total_ms = view.state.output_duration_ms().max(1_000);
    let playhead_ms = view.state.playhead_ms.min(total_ms);
    let playhead_ratio = (playhead_ms as f32 / total_ms as f32).clamp(0.0, 1.0);
    let selected_clip = view.state.selected_clip_id;
    let selected_zoom = view.state.selected_zoom_id;

    let clip_total: u64 = project
        .clips
        .iter()
        .map(|c| c.source_end_ms.saturating_sub(c.source_start_ms).max(1))
        .sum::<u64>()
        .max(1);
    let clips: Vec<(usize, Uuid, f32, bool)> = project
        .clips
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let dur = c.source_end_ms.saturating_sub(c.source_start_ms).max(1);
            let frac = (dur as f32 / clip_total as f32).clamp(0.05, 1.0);
            (i, c.id, frac, selected_clip == Some(c.id))
        })
        .collect();

    let zooms: Vec<ZoomPill> = project
        .zoom_regions
        .iter()
        .enumerate()
        .map(|(idx, z)| {
            let left = (z.start_ms as f32 / total_ms as f32).clamp(0.0, 0.96);
            let width = ((z.end_ms.saturating_sub(z.start_ms)) as f32 / total_ms as f32)
                .clamp(0.04, 1.0 - left);
            let active = selected_zoom == Some(z.id)
                || (playhead_ms >= z.start_ms && playhead_ms <= z.end_ms);
            ZoomPill {
                idx,
                id: z.id,
                start_ms: z.start_ms,
                end_ms: z.end_ms,
                left,
                width,
                label: format!("{:.1}x", z.scale),
                active,
            }
        })
        .collect();

    div()
        .w_full()
        .bg(rgb(0xffffff))
        .border_t_1()
        .border_color(rgb(0xebedf2))
        .flex()
        .flex_col()
        .child(render_transport_bar(view, cx))
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
                .child(render_clip_track(clips, cx))
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

fn render_clip_track(
    clips: Vec<(usize, Uuid, f32, bool)>,
    cx: &mut Context<StudioView>,
) -> impl IntoElement {
    let mut track = div().h(px(36.0)).w_full().flex().gap_1();
    for (i, id, frac, is_sel) in clips {
        let label = if i == 0 {
            "Clip".to_string()
        } else {
            format!("Clip {}", i + 1)
        };
        track = track.child(
            div()
                .id(("clip-seg", i))
                .w(relative(frac))
                .h_full()
                .rounded(px(9.0))
                .bg(linear_gradient(
                    180.0,
                    linear_color_stop(rgb(0xdbeafe), 0.0),
                    linear_color_stop(rgb(0x93c5fd), 1.0),
                ))
                .border_1()
                .border_color(if is_sel { rgb(0x2563eb) } else { rgb(0x93c5fd) })
                .cursor_pointer()
                .flex()
                .items_center()
                .justify_center()
                .gap_1p5()
                .text_xs()
                .font_weight(FontWeight::MEDIUM)
                .text_color(rgb(0x1e3a8a))
                .child(IconName::Frame)
                .child(label)
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.state.selected_clip_id = Some(id);
                    cx.notify();
                })),
        );
    }
    track
}

fn render_zoom_track(zooms: Vec<ZoomPill>, cx: &mut Context<StudioView>) -> impl IntoElement {
    let mut track = div().relative().h(px(38.0)).w_full();
    for z in zooms {
        let seek_ms = (z.start_ms + 450).min(z.end_ms);
        let id = z.id;
        track = track.child(
            div()
                .id(("zoom-pill", z.idx))
                .absolute()
                .top_0()
                .bottom_0()
                .left(relative(z.left))
                .w(relative(z.width))
                .rounded(px(9.0))
                .bg(linear_gradient(
                    180.0,
                    linear_color_stop(rgb(0xf3e8ff), 0.0),
                    linear_color_stop(rgb(0xe9d5ff), 1.0),
                ))
                .border_1()
                .border_color(if z.active { rgb(0x7c3aed) } else { rgb(0xc084fc) })
                .cursor_pointer()
                .flex()
                .items_center()
                .justify_center()
                .gap_1()
                .text_xs()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(rgb(0x6b21a8))
                .child(IconName::Search)
                .child(z.label)
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.state.selected_zoom_id = Some(id);
                    this.seek_to_ms(seek_ms, cx);
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
    let mut labels = div()
        .relative()
        .size_full()
        .flex()
        .items_center()
        .justify_between();
    for i in 0..=ticks {
        let ms = (total_ms * i as u64) / ticks as u64;
        labels = labels.child(div().text_xs().text_color(rgb(0x94a3b8)).child(fmt_time(ms)));
    }

    let mut scrub = div().absolute().top_0().left_0().size_full().flex();
    for step in 0..=40_usize {
        let ms = (total_ms * step as u64) / 40;
        scrub = scrub.child(
            div()
                .id(("ruler-scrub", step))
                .flex_1()
                .h_full()
                .cursor_pointer()
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |this, _, _, cx| this.seek_to_ms(ms, cx)),
                )
                .on_mouse_move(cx.listener(move |this, ev: &MouseMoveEvent, _, cx| {
                    if ev.pressed_button == Some(MouseButton::Left) {
                        this.seek_to_ms(ms, cx);
                    }
                })),
        );
    }

    div()
        .relative()
        .h(px(26.0))
        .w_full()
        .px_3()
        .child(labels)
        .child(scrub)
        .child(
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
