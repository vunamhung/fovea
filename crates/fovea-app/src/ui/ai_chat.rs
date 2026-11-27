use gpui_kit::component::button::Button;
use gpui_kit::component::IconName;
use gpui_kit::*;

use crate::ui::studio::StudioView;

pub fn render_ai_chat_panel(_view: &StudioView, cx: &mut Context<StudioView>) -> impl IntoElement {
    div()
        .flex_1()
        .p_4()
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
                        .text_color(rgb(0x94a3b8))
                        .child("AI Studio Copilot"),
                )
                .child(
                    div()
                        .p_3()
                        .rounded_md()
                        .bg(rgb(0x181c26))
                        .border_1()
                        .border_color(rgb(0x262c3d))
                        .text_xs()
                        .text_color(rgb(0xcbd5e1))
                        .child(
                            "Connects via local claude / agy CLI (sandboxed to fovea mcp) or direct API keys. Every edit runs inside a single Undo transaction.",
                        ),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_1p5()
                        .child(
                            Button::new("ai-quick-zoom")
                                .outline()
                                .icon(IconName::Bot)
                                .label("Demo: Add Smart 2.0x Zoom")
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.add_demo_zoom(cx);
                                })),
                        )
                        .child(
                            Button::new("ai-quick-blur")
                                .outline()
                                .icon(IconName::EyeOff)
                                .label("Demo: Add Privacy Blur Box")
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.add_demo_privacy_blur(cx);
                                })),
                        ),
                ),
        )
        .child(
            div()
                .p_2p5()
                .rounded_md()
                .bg(rgb(0x0f1218))
                .border_1()
                .border_color(rgb(0x262c3d))
                .text_xs()
                .text_color(rgb(0x64748b))
                .child("Provider: claude / agy CLI • Voice: Vbee TTS"),
        )
}
