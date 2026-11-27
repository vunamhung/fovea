use gpui_kit::*;

/// Tactile macOS horizontal pill scrubber styled after Recordly / Screen Studio sliders.
/// Supports both instant click and drag-to-scrub across 20 smooth detents in `[0.0, 1.0]`.
pub fn pill_slider(
    id: &'static str,
    label: &'static str,
    value_text: String,
    ratio: f32,
    on_step: impl Fn(&f32, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    let clamped = ratio.clamp(0.0, 1.0);
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
                .bg(rgb(0xe2e8f0)),
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
            div()
                .absolute()
                .top_0()
                .left_0()
                .size_full()
                .flex()
                .children((0..=20_usize).map(move |step| {
                    let target = step as f32 / 20.0;
                    let cb_down = std::rc::Rc::clone(&on_step);
                    let cb_drag = std::rc::Rc::clone(&on_step);
                    div()
                        .id((id, step))
                        .flex_1()
                        .h_full()
                        .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                            cb_down(&target, window, cx);
                        })
                        .on_mouse_move(move |ev, window, cx| {
                            if ev.pressed_button == Some(MouseButton::Left) {
                                cb_drag(&target, window, cx);
                            }
                        })
                })),
        )
}
