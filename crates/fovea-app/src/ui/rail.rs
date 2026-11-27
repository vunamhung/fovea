use gpui_kit::component::IconName;
use gpui_kit::*;

use crate::state::RightPanelTab;
use crate::ui::studio::StudioView;

pub fn render_left_rail(view: &StudioView, cx: &mut Context<StudioView>) -> impl IntoElement {
    let active = view.state.active_tab;
    let items = [
        ("rail-bg", IconName::Palette, RightPanelTab::Inspector),
        ("rail-cursor", IconName::Frame, RightPanelTab::Cursor),
        ("rail-webcam", IconName::Eye, RightPanelTab::Webcam),
        ("rail-voice", IconName::Mic, RightPanelTab::VoiceCaptions),
        ("rail-ai", IconName::Bot, RightPanelTab::AiChat),
        ("rail-perms", IconName::Settings, RightPanelTab::Permissions),
    ];

    div()
        .w(px(50.0))
        .h_full()
        .py_3()
        .bg(rgb(0xffffff))
        .border_r_1()
        .border_color(rgb(0xebedf2))
        .flex()
        .flex_col()
        .items_center()
        .justify_between()
        .child(
            div()
                .flex()
                .flex_col()
                .items_center()
                .gap_2()
                .children(items.into_iter().map(|(id, icon, tab)| {
                    let is_active = active == tab;
                    div()
                        .id(id)
                        .w(px(34.0))
                        .h(px(34.0))
                        .rounded(px(10.0))
                        .bg(if is_active { rgb(0xeff6ff) } else { rgb(0xffffff) })
                        .border_1()
                        .border_color(if is_active { rgb(0xbfdbfe) } else { rgb(0xffffff) })
                        .text_color(if is_active { rgb(0x2563eb) } else { rgb(0x64748b) })
                        .flex()
                        .items_center()
                        .justify_center()
                        .cursor_pointer()
                        .child(icon)
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.state.set_tab(tab);
                            cx.notify();
                        }))
                })),
        )
        .child(
            div()
                .w(px(28.0))
                .h(px(28.0))
                .rounded_full()
                .bg(rgb(0xf1f5f9))
                .text_color(rgb(0x64748b))
                .flex()
                .items_center()
                .justify_center()
                .child(IconName::CircleUser),
        )
}
