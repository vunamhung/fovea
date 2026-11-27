use gpui_kit::component::IconName;
use gpui_kit::*;

use crate::ui::hud_launcher::open_hud_launcher_window;
use crate::ui::studio::StudioView;

pub fn render_topbar(view: &StudioView, cx: &mut Context<StudioView>) -> impl IntoElement {
    let state = &view.state;
    let title = state.project().title.clone();
    let can_undo = state.can_undo();
    let can_redo = state.can_redo();

    div()
        .h(px(48.0))
        .w_full()
        .pl(px(82.0))
        .pr_4()
        .bg(rgb(0xffffff))
        .border_b_1()
        .border_color(rgb(0xebedf2))
        .flex()
        .items_center()
        .justify_between()
        .child(
            div()
                .flex()
                .items_center()
                .gap_1p5()
                .child(top_icon_btn(
                    "top-new-rec",
                    IconName::Play,
                    true,
                    cx.listener(|_, _, _, cx| open_hud_launcher_window(cx)),
                ))
                .child(
                    div()
                        .w(px(1.0))
                        .h(px(18.0))
                        .mx_1()
                        .bg(rgb(0xe5e7eb)),
                )
                .child(top_icon_btn(
                    "top-undo",
                    IconName::Undo2,
                    can_undo,
                    cx.listener(|this, _, _, cx| {
                        this.state.undo();
                        cx.notify();
                    }),
                ))
                .child(top_icon_btn(
                    "top-redo",
                    IconName::Redo2,
                    can_redo,
                    cx.listener(|this, _, _, cx| {
                        this.state.redo();
                        cx.notify();
                    }),
                )),
        )
        .child(
            div()
                .id("top-drag-title")
                .flex_1()
                .h_full()
                .on_mouse_down(MouseButton::Left, |_, window, _| {
                    window.start_window_move();
                })
                .flex()
                .items_center()
                .justify_center()
                .gap_2()
                .child(div().w(px(7.0)).h(px(7.0)).rounded_full().bg(rgb(0x2563eb)))
                .child(
                    div()
                        .text_xs()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(rgb(0x111827))
                        .child(title),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(0x9ca3af))
                        .child(".fovea"),
                ),
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap_2()
                .child(
                    div()
                        .id("top-presets")
                        .h(px(30.0))
                        .px_3()
                        .rounded(px(8.0))
                        .bg(rgb(0xffffff))
                        .border_1()
                        .border_color(rgb(0xe5e7eb))
                        .flex()
                        .items_center()
                        .gap_1p5()
                        .cursor_pointer()
                        .text_xs()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(rgb(0x374151))
                        .child(IconName::Star)
                        .child("Presets")
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.apply_next_preset(cx);
                        })),
                )
                .child(
                    div()
                        .id("top-export")
                        .h(px(32.0))
                        .px_4()
                        .rounded(px(9.0))
                        .bg(rgb(0x2563eb))
                        .flex()
                        .items_center()
                        .gap_1p5()
                        .cursor_pointer()
                        .text_xs()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(rgb(0xffffff))
                        .child(IconName::ExternalLink)
                        .child("Export")
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.export_project_bundle(cx);
                        })),
                ),
        )
}

fn top_icon_btn(
    id: &'static str,
    icon: IconName,
    enabled: bool,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    div()
        .id(id)
        .w(px(28.0))
        .h(px(28.0))
        .rounded(px(7.0))
        .bg(if enabled { rgb(0xf8fafc) } else { rgb(0xffffff) })
        .border_1()
        .border_color(rgb(0xe5e7eb))
        .flex()
        .items_center()
        .justify_center()
        .cursor_pointer()
        .text_color(if enabled { rgb(0x334155) } else { rgb(0xcbd5e1) })
        .child(icon)
        .on_click(on_click)
}
