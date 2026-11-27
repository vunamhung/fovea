use gpui_kit::component::IconName;
use gpui_kit::*;

use crate::ui::studio::StudioView;

pub fn render_ai_chat_panel(_view: &StudioView, cx: &mut Context<StudioView>) -> impl IntoElement {
    div()
        .flex_1()
        .flex()
        .flex_col()
        .justify_between()
        .gap_3()
        .child(
            div()
                .flex()
                .flex_col()
                .gap_3()
                .child(
                    div()
                        .text_xs()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(rgb(0x64748b))
                        .child("AI STUDIO COPILOT"),
                )
                .child(
                    div()
                        .p_3()
                        .rounded(px(12.0))
                        .bg(rgb(0xf8fafc))
                        .border_1()
                        .border_color(rgb(0xe2e8f0))
                        .text_xs()
                        .text_color(rgb(0x334155))
                        .child(
                            "Edit your video conversationally with Claude / AGY CLI (sandboxed to fovea mcp) or direct API keys. Every AI turn supports one-click Undo.",
                        ),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .child(action_pill(
                            "ai-zoom",
                            IconName::Bot,
                            "Add Smart 1.8x Zoom",
                            cx.listener(|this, _, _, cx| this.add_demo_zoom(cx)),
                        ))
                        .child(action_pill(
                            "ai-blur",
                            IconName::EyeOff,
                            "Add Privacy Blur Box",
                            cx.listener(|this, _, _, cx| this.add_demo_privacy_blur(cx)),
                        )),
                ),
        )
        .child(
            div()
                .px_3()
                .py_2p5()
                .rounded(px(10.0))
                .bg(rgb(0xf1f5f9))
                .border_1()
                .border_color(rgb(0xe2e8f0))
                .text_xs()
                .text_color(rgb(0x64748b))
                .child("Ask Claude / AGY • Vbee TTS ready (M3)"),
        )
}

fn action_pill(
    id: &'static str,
    icon: IconName,
    label: &'static str,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    div()
        .id(id)
        .h(px(34.0))
        .px_3()
        .rounded(px(9.0))
        .bg(rgb(0xeff6ff))
        .border_1()
        .border_color(rgb(0xbfdbfe))
        .flex()
        .items_center()
        .gap_2()
        .cursor_pointer()
        .text_xs()
        .font_weight(FontWeight::MEDIUM)
        .text_color(rgb(0x1d4ed8))
        .child(icon)
        .child(label)
        .on_click(on_click)
}
