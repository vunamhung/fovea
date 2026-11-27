use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::IconName;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use crate::state::RightPanelTab;
use crate::ui::ai_chat::render_ai_chat_panel;
use crate::ui::studio::StudioView;

pub fn render_inspector(view: &StudioView, cx: &mut Context<StudioView>) -> impl IntoElement {
    let tab = view.state.active_tab;

    div()
        .w(px(330.0))
        .h_full()
        .bg(rgb(0x12141a))
        .border_l_1()
        .border_color(rgb(0x232734))
        .flex()
        .flex_col()
        .child(
            div()
                .p_3()
                .border_b_1()
                .border_color(rgb(0x232734))
                .flex()
                .gap_2()
                .child(
                    Button::new("tab-inspector")
                        .flex_1()
                        .map(|b| {
                            if tab == RightPanelTab::Inspector {
                                b.primary()
                            } else {
                                b.outline()
                            }
                        })
                        .icon(IconName::Inspector)
                        .label("Inspector")
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.state.set_tab(RightPanelTab::Inspector);
                            cx.notify();
                        })),
                )
                .child(
                    Button::new("tab-ai-chat")
                        .flex_1()
                        .map(|b| {
                            if tab == RightPanelTab::AiChat {
                                b.primary()
                            } else {
                                b.outline()
                            }
                        })
                        .icon(IconName::Bot)
                        .label("AI Chat")
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.state.set_tab(RightPanelTab::AiChat);
                            cx.notify();
                        })),
                ),
        )
        .child(match tab {
            RightPanelTab::Inspector => render_inspector_body(view, cx).into_any_element(),
            RightPanelTab::AiChat => render_ai_chat_panel(view, cx).into_any_element(),
        })
}

fn render_inspector_body(view: &StudioView, cx: &mut Context<StudioView>) -> impl IntoElement {
    let state = &view.state;
    let sources: Vec<(usize, String, bool)> = state
        .sources
        .iter()
        .take(6)
        .enumerate()
        .map(|(i, s)| {
            (
                i,
                format!("{} ({}×{})", s.name, s.width, s.height),
                i == state.selected_source_idx,
            )
        })
        .collect();

    div()
        .flex_1()
        .p_4()
        .flex()
        .flex_col()
        .gap_4()
        .child(section_header("Capture Source"))
        .child(
            div()
                .flex()
                .flex_col()
                .gap_1p5()
                .children(sources.into_iter().map(|(idx, label, selected)| {
                    Button::new(("src-item", idx))
                        .w_full()
                        .map(|b| if selected { b.primary() } else { b.outline() })
                        .label(label)
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.state.select_source(idx);
                            cx.notify();
                        }))
                }))
                .child(
                    Button::new("refresh-sources")
                        .outline()
                        .icon(IconName::RefreshCw)
                        .label("Refresh Sources")
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.refresh_sources(cx);
                        })),
                ),
        )
        .child(section_header("Capture Options"))
        .child(
            div()
                .flex()
                .flex_col()
                .gap_2()
                .child(toggle_row(
                    "toggle-sys-audio",
                    "System Audio (48kHz)",
                    state.capture_system_audio,
                    cx.listener(|this, _, _, cx| {
                        this.state.capture_system_audio = !this.state.capture_system_audio;
                        cx.notify();
                    }),
                ))
                .child(toggle_row(
                    "toggle-hide-cursor",
                    "Hide Native OS Cursor",
                    state.hide_os_cursor,
                    cx.listener(|this, _, _, cx| {
                        this.state.hide_os_cursor = !this.state.hide_os_cursor;
                        cx.notify();
                    }),
                ))
                .child(toggle_row(
                    "toggle-keystrokes",
                    "Keystroke HUD (Secure-Input Safe)",
                    state.capture_keystrokes,
                    cx.listener(|this, _, _, cx| {
                        this.state.capture_keystrokes = !this.state.capture_keystrokes;
                        cx.notify();
                    }),
                )),
        )
        .child(section_header("Voiceover & Captions"))
        .child(
            div()
                .p_3()
                .rounded_md()
                .bg(rgb(0x181c26))
                .border_1()
                .border_color(rgb(0x262c3d))
                .text_xs()
                .text_color(rgb(0x94a3b8))
                .child("Vbee Neural TTS + On-demand Whisper CoreML STT ready for Milestone 3."),
        )
}

fn section_header(text: &'static str) -> impl IntoElement {
    div()
        .text_xs()
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(rgb(0x94a3b8))
        .child(text)
}

fn toggle_row(
    id: &'static str,
    label: &'static str,
    enabled: bool,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    Button::new(id)
        .w_full()
        .map(|b| if enabled { b.primary() } else { b.outline() })
        .icon(if enabled {
            IconName::CircleCheck
        } else {
            IconName::CircleX
        })
        .label(label)
        .on_click(on_click)
}
