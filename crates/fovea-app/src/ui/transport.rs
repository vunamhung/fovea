use gpui_kit::component::IconName;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use crate::ui::studio::StudioView;

pub fn render_transport_bar(view: &StudioView, cx: &mut Context<StudioView>) -> impl IntoElement {
    let playhead_ms = view.state.playhead_ms;
    let total_ms = view.state.output_duration_ms().max(1_000);
    let is_playing = view.state.is_playing;
    let volume_pct = view.state.volume_pct;
    let can_del_clip = view.state.project().clips.len() > 1;
    let selected_zoom_scale = view.state.selected_zoom_id.and_then(|id| {
        view.state
            .project()
            .zoom_regions
            .iter()
            .find(|z| z.id == id)
            .map(|z| z.scale)
    });

    div()
        .h(px(44.0))
        .w_full()
        .px_4()
        .border_b_1()
        .border_color(rgb(0xf1f5f9))
        .flex()
        .items_center()
        .justify_between()
        .child(
            div()
                .flex()
                .items_center()
                .gap_1p5()
                .child(transport_btn(
                    "tl-add-zoom",
                    IconName::Plus,
                    Some("Zoom"),
                    cx.listener(|this, _, _, cx| this.add_demo_zoom(cx)),
                ))
                .child(transport_btn(
                    "tl-split",
                    IconName::Minus,
                    Some("Split"),
                    cx.listener(|this, _, _, cx| this.split_clip_at_playhead(cx)),
                ))
                .when(can_del_clip, |d| {
                    d.child(transport_btn(
                        "tl-del-clip",
                        IconName::Close,
                        Some("Delete Clip"),
                        cx.listener(|this, _, _, cx| this.delete_selected_clip(cx)),
                    ))
                })
                .child(transport_btn(
                    "tl-add-blur",
                    IconName::EyeOff,
                    Some("Blur"),
                    cx.listener(|this, _, _, cx| this.add_demo_privacy_blur(cx)),
                ))
                .when_some(selected_zoom_scale, |d, current_scale| {
                    d.child(render_zoom_tuner(current_scale, cx))
                }),
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap_3()
                .child(div().text_xs().text_color(rgb(0x64748b)).child(fmt_time(playhead_ms)))
                .child(
                    div()
                        .id("tl-rewind")
                        .cursor_pointer()
                        .text_color(rgb(0x475569))
                        .child(IconName::ChevronLeft)
                        .on_click(cx.listener(|this, _, _, cx| this.seek_to_ms(0, cx))),
                )
                .child(
                    div()
                        .id("tl-play-pause")
                        .w(px(32.0))
                        .h(px(32.0))
                        .rounded_full()
                        .bg(rgb(0x18181b))
                        .text_color(rgb(0xffffff))
                        .flex()
                        .items_center()
                        .justify_center()
                        .cursor_pointer()
                        .child(if is_playing { IconName::Pause } else { IconName::Play })
                        .on_click(cx.listener(|this, _, _, cx| this.toggle_playback(cx))),
                )
                .child(div().text_xs().text_color(rgb(0x64748b)).child(fmt_time(total_ms))),
        )
        .child(
            div()
                .id("tl-volume")
                .h(px(26.0))
                .px_3()
                .rounded(px(7.0))
                .bg(rgb(0xf1f5f9))
                .border_1()
                .border_color(rgb(0xe5e7eb))
                .cursor_pointer()
                .flex()
                .items_center()
                .text_xs()
                .font_weight(FontWeight::MEDIUM)
                .text_color(rgb(0x475569))
                .child(if volume_pct == 0 {
                    "Muted".into()
                } else {
                    format!("{volume_pct}%")
                })
                .on_click(cx.listener(|this, _, _, cx| {
                    this.state.volume_pct = match this.state.volume_pct {
                        100 => 50,
                        50 => 0,
                        _ => 100,
                    };
                    cx.notify();
                })),
        )
}

fn render_zoom_tuner(current_scale: f32, cx: &mut Context<StudioView>) -> impl IntoElement {
    let mut row = div()
        .ml_2()
        .px_1p5()
        .h(px(28.0))
        .rounded(px(8.0))
        .bg(rgb(0xfaf5ff))
        .border_1()
        .border_color(rgb(0xd8b4fe))
        .flex()
        .items_center()
        .gap_1();
    for (idx, scale) in [1.3_f32, 1.5, 1.8, 2.2].into_iter().enumerate() {
        let active = (current_scale - scale).abs() < 0.08;
        row = row.child(
            div()
                .id(("z-scale", idx))
                .px_1p5()
                .py_0p5()
                .rounded(px(5.0))
                .bg(if active { rgb(0x7c3aed) } else { rgba(0x00000000) })
                .text_xs()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(if active { rgb(0xffffff) } else { rgb(0x6b21a8) })
                .cursor_pointer()
                .child(format!("{scale:.1}x"))
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.set_selected_zoom_scale(scale, cx);
                })),
        );
    }
    row.child(
        div()
            .id("z-delete")
            .px_1()
            .cursor_pointer()
            .text_xs()
            .text_color(rgb(0xdc2626))
            .child(IconName::Close)
            .on_click(cx.listener(|this, _, _, cx| this.delete_selected_zoom(cx))),
    )
}

fn transport_btn(
    id: &'static str,
    icon: IconName,
    label: Option<&'static str>,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    let mut el = div()
        .id(id)
        .h(px(28.0))
        .px_2p5()
        .rounded(px(8.0))
        .bg(rgb(0xf8fafc))
        .border_1()
        .border_color(rgb(0xe5e7eb))
        .flex()
        .items_center()
        .gap_1p5()
        .cursor_pointer()
        .text_xs()
        .font_weight(FontWeight::MEDIUM)
        .text_color(rgb(0x334155))
        .child(icon);
    if let Some(txt) = label {
        el = el.child(txt);
    }
    el.on_click(on_click)
}

pub fn fmt_time(ms: u64) -> String {
    let secs = ms / 1000;
    format!("{}:{:02}", secs / 60, secs % 60)
}
