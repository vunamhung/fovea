use gpui_kit::component::label::Label;
use gpui_kit::*;

struct FoveaBootstrap;

impl Render for FoveaBootstrap {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .size_full()
            .items_center()
            .justify_center()
            .child(Label::new("Fovea Studio"))
    }
}

fn main() {
    gpui_kit::application()
        .with_assets(gpui_kit::assets::Assets)
        .run(|cx| {
            gpui_kit::init(cx);
            gpui_kit::open_window(WindowOptions::default(), cx, |_, cx| {
                cx.new(|_| FoveaBootstrap)
            })
            .expect("failed to open Fovea window");
        });
}
