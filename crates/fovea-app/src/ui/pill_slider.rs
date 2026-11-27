use gpui_kit::*;

/// Tactile macOS horizontal pill scrubber styled after Recordly / Screen Studio sliders.
/// Clicking anywhere along the pill track sets the normalized value in `[0.0, 1.0]`.
pub fn pill_slider(
    id: &'static str,
    label: &'static str,
    value_text: String,
    ratio: f32,
    on_step: impl Fn(&f32, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    let clamped = ratio.clamp(0.0, 1.0);
    let fill_pct = clamped * 100.0;
    let on_step = std::rc::Rc::new(on_step);

    div()
        .id(id)
        .relative()
        .h(px(30.0))
        .w_full()
        .rounded(px(9.0))
        .bg(rgb(0xf4f5f7))
        .border_1()
        .border_color(rgb(0xe8eaef))
        .overflow_hidden()
        .cursor_pointer()
        .child(
            div()
                .absolute()
                .top_0()
                .bottom_0()
                .left_0()
                .w(relative(clamped))
                .bg(rgb(0xe5e7eb)),
        )
        .child(
            div()
                .absolute()
                .top(px(6.0))
                .bottom(px(6.0))
                .left(relative(clamped.clamp(0.015, 0.985)))
                .w(px(2.0))
                .rounded_full()
                .bg(rgb(0x334155)),
        )
        .child(
            div()
                .relative()
                .size_full()
                .px_3()
                .flex()
                .items_center()
                .justify_between()
                .child(
                    div()
                        .text_xs()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(rgb(0x475569))
                        .child(label),
                )
                .child(
                    div()
                        .text_xs()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(rgb(0x0f172a))
                        .child(value_text),
                ),
        )
        .child(
            // Interactive step zones across the track (20 detents = 5% steps).
            div()
                .absolute()
                .inset_0()
                .flex()
                .children((0..=20_usize).map(move |step| {
                    let target = step as f32 / 20.0;
                    let cb = std::rc::Rc::clone(&on_step);
                    div()
                        .id((id, step))
                        .flex_1()
                        .h_full()
                        .on_click(move |_, window, cx| {
                            cb(&target, window, cx);
                        })
                })),
        )
        .tooltip(move |_, cx| {
            cx.new(|_| SliderTip(format!("{label}: {fill_pct:.0}%"))).into()
        })
}

struct SliderTip(String);

impl Render for SliderTip {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .px_2()
            .py_1()
            .rounded_md()
            .bg(rgb(0x0f172a))
            .text_xs()
            .text_color(rgb(0xffffff))
            .child(self.0.clone())
    }
}
