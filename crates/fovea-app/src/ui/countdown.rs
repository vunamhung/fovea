use gpui_kit::*;

/// Centered translucent 3 -> 2 -> 1 countdown overlay shown before recording begins.
pub struct CountdownOverlay {
    pub count: u8,
}

impl CountdownOverlay {
    pub fn new(initial: u8) -> Self {
        Self { count: initial }
    }
}

impl Render for CountdownOverlay {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            .child(
                div()
                    .w(px(148.0))
                    .h(px(148.0))
                    .rounded(px(28.0))
                    .bg(rgba(0x090b10de))
                    .border_1()
                    .border_color(rgba(0xffffff22))
                    .shadow_lg()
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_color(rgb(0xffffff))
                    .text_size(px(76.0))
                    .font_weight(FontWeight::BOLD)
                    .child(self.count.to_string()),
            )
    }
}
