use gpui_kit::component::IconName;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use crate::state::SessionMode;
use crate::ui::hud_launcher::HudLauncherView;
use crate::ui::studio::open_studio_editor_window;

pub fn render_hud_pill(view: &HudLauncherView, cx: &mut Context<HudLauncherView>) -> impl IntoElement {
    let recording = view.state.mode == SessionMode::Recording;
    let source_label = view
        .state
        .selected_source()
        .map(|s| {
            let short: String = s.name.chars().take(18).collect();
            if view.state.sources.len() > 1 {
                format!("{short} ({}/{})", view.state.selected_source_idx + 1, view.state.sources.len())
            } else {
                short
            }
        })
        .unwrap_or_else(|| "Display 1".into());
    let timer = format!("{:02}:{:02}", view.elapsed_sec / 60, view.elapsed_sec % 60);
    let show_notice = !recording && view.state.last_status != "Ready";

    div()
        .size_full()
        .p_1()
        .flex()
        .flex_col()
        .items_center()
        .justify_end()
        .gap_1()
        .when(show_notice, |d| {
            d.child(
                div()
                    .px_3()
                    .py_1()
                    .rounded_full()
                    .bg(rgba(0x0f172ae6))
                    .border_1()
                    .border_color(rgba(0xffffff20))
                    .text_xs()
                    .text_color(rgb(0xfef08a))
                    .child(view.state.last_status.clone()),
            )
        })
        .child(
            div()
                .h(px(56.0))
                .w_full()
                .px_3()
                .rounded(px(18.0))
                .bg(rgba(0x14161df2))
                .border_1()
                .border_color(rgba(0xffffff24))
                .shadow_lg()
                .flex()
                .items_center()
                .justify_between()
                .gap_2()
                .child(render_drag_grip())
                .child(if recording {
                    render_recording_controls(&timer, cx).into_any_element()
                } else {
                    render_idle_controls(view, &source_label, cx).into_any_element()
                })
                .child(render_right_actions(cx)),
        )
}

fn render_drag_grip() -> impl IntoElement {
    div()
        .id("hud-drag-grip")
        .w(px(14.0))
        .h(px(28.0))
        .rounded(px(5.0))
        .cursor_grab()
        .on_mouse_down(MouseButton::Left, |_, window, _| window.start_window_move())
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap(px(3.0))
        .child(div().w(px(3.0)).h(px(3.0)).rounded_full().bg(rgba(0xffffff44)))
        .child(div().w(px(3.0)).h(px(3.0)).rounded_full().bg(rgba(0xffffff44)))
        .child(div().w(px(3.0)).h(px(3.0)).rounded_full().bg(rgba(0xffffff44)))
}

fn render_recording_controls(timer: &str, cx: &mut Context<HudLauncherView>) -> impl IntoElement {
    div()
        .flex().items_center().gap_2p5()
        .child(div().w(px(10.0)).h(px(10.0)).rounded_full().bg(rgb(0xef4444)))
        .child(div().text_sm().font_weight(FontWeight::SEMIBOLD).text_color(rgb(0xffffff)).child(timer.to_string()))
        .child(hud_pill_btn("hud-stop", "Stop & Edit", rgb(0xef4444), rgb(0xffffff), cx.listener(|this, _, win, cx| this.stop_and_open_editor(win, cx))))
        .child(hud_pill_btn("hud-cancel", "Cancel", rgb(0x232734), rgb(0xcbd5e1), cx.listener(|this, _, _, cx| this.cancel_recording(cx))))
}

fn render_idle_controls(
    view: &HudLauncherView,
    source_label: &str,
    cx: &mut Context<HudLauncherView>,
) -> impl IntoElement {
    let needs_perm = !view.state.permissions.all_required_granted();
    div()
        .flex()
        .items_center()
        .gap_1p5()
        .child(hud_pill_btn(
            "hud-source",
            source_label,
            rgb(0x232734),
            rgb(0xe2e8f0),
            cx.listener(|this, _, _, cx| {
                this.state.cycle_source();
                cx.notify();
            }),
        ))
        .child(hud_icon_toggle(
            "hud-refresh",
            IconName::RefreshCw,
            false,
            cx.listener(|this, _, _, cx| this.refresh_sources(cx)),
        ))
        .when(needs_perm, |d| {
            d.child(hud_pill_btn(
                "hud-grant-perm",
                "Grant Access",
                rgb(0xd97706),
                rgb(0xffffff),
                cx.listener(|this, _, _, cx| this.request_screen_permission(cx)),
            ))
        })
        .child(hud_icon_toggle(
            "hud-audio",
            IconName::Mic,
            view.state.capture_system_audio,
            cx.listener(|this, _, _, cx| {
                this.state.capture_system_audio = !this.state.capture_system_audio;
                cx.notify();
            }),
        ))
        .child(hud_pill_btn(
            "hud-timer",
            if view.state.countdown_enabled { "3s" } else { "0s" },
            if view.state.countdown_enabled { rgb(0x334155) } else { rgb(0x232734) },
            rgb(0xe2e8f0),
            cx.listener(|this, _, _, cx| {
                this.state.countdown_enabled = !this.state.countdown_enabled;
                cx.notify();
            }),
        ))
        .child(hud_pill_btn(
            "hud-record",
            if view.counting_down { "Starting..." } else { "● Record" },
            rgb(0xef4444),
            rgb(0xffffff),
            cx.listener(|this, _, _, cx| this.start_countdown_and_record(cx)),
        ))
}

fn render_right_actions(cx: &mut Context<HudLauncherView>) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .gap_1p5()
        .child(hud_pill_btn(
            "hud-open-studio",
            "Studio",
            rgb(0x2563eb),
            rgb(0xffffff),
            cx.listener(|_, _, win, cx| {
                open_studio_editor_window(None, cx);
                win.remove_window();
            }),
        ))
        .child(
            div()
                .id("hud-quit")
                .w(px(28.0))
                .h(px(28.0))
                .rounded_full()
                .bg(rgba(0xffffff14))
                .flex()
                .items_center()
                .justify_center()
                .cursor_pointer()
                .text_color(rgb(0x94a3b8))
                .child(IconName::Close)
                .on_click(|_, _, cx| cx.quit()),
        )
}

fn hud_pill_btn(
    id: &'static str,
    label: &str,
    bg_color: Rgba,
    fg_color: Rgba,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    div()
        .id(id)
        .px_2p5()
        .h(px(34.0))
        .rounded(px(10.0))
        .bg(bg_color)
        .flex()
        .items_center()
        .justify_center()
        .cursor_pointer()
        .text_xs()
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(fg_color)
        .child(label.to_string())
        .on_click(on_click)
}

fn hud_icon_toggle(
    id: &'static str,
    icon: IconName,
    active: bool,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    div()
        .id(id)
        .w(px(32.0))
        .h(px(34.0))
        .rounded(px(10.0))
        .bg(if active { rgb(0x2563eb) } else { rgb(0x232734) })
        .text_color(if active { rgb(0xffffff) } else { rgb(0x94a3b8) })
        .flex()
        .items_center()
        .justify_center()
        .cursor_pointer()
        .child(icon)
        .on_click(on_click)
}
