use gpui_kit::component::IconName;
use gpui_kit::*;

use crate::ui::studio::StudioView;

pub fn render_transport_bar(
    playhead_ms: u64,
    total_ms: u64,
    is_playing: bool,
    volume_pct: u8,
    cx: &mut Context<StudioView>,
) -> impl IntoElement {
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
                .gap_2()
                .child(transport_btn(
                    "tl-add-layer",
                    IconName::Plus,
                    Some("Add Layer"),
                    cx.listener(|this, _, _, cx| this.add_demo_zoom(cx)),
                ))
                .child(transport_btn(
                    "tl-zoom-add",
                    IconName::Search,
                    None,
                    cx.listener(|this, _, _, cx| this.add_demo_zoom(cx)),
                ))
                .child(transport_btn(
                    "tl-add-blur",
                    IconName::EyeOff,
                    None,
                    cx.listener(|this, _, _, cx| this.add_demo_privacy_blur(cx)),
                )),
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap_3()
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(0x64748b))
                        .child(fmt_time(playhead_ms)),
                )
                .child(
                    div()
                        .id("tl-rewind")
                        .cursor_pointer()
                        .text_color(rgb(0x475569))
                        .child(IconName::ChevronLeft)
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.seek_to_ms(0, cx);
                        })),
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
                        .child(if is_playing {
                            IconName::Pause
                        } else {
                            IconName::Play
                        })
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.toggle_playback(cx);
                        })),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(0x64748b))
                        .child(fmt_time(total_ms)),
                ),
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
