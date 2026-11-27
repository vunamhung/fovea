use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::{Disableable, IconName};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use fovea_core::AspectRatio;

use crate::state::SessionMode;
use crate::ui::studio::StudioView;

pub fn render_topbar(view: &StudioView, cx: &mut Context<StudioView>) -> impl IntoElement {
    let state = &view.state;
    let is_recording = state.mode == SessionMode::Recording;
    let perms_ok = state.permissions.all_required_granted();
    let ratio = state.project().frame.aspect_ratio;
    let title = state.project().title.clone();
    let can_undo = state.can_undo();
    let can_redo = state.can_redo();

    div()
        .h(px(52.0))
        .w_full()
        .px_4()
        .flex()
        .items_center()
        .justify_between()
        .bg(rgb(0x12141a))
        .border_b_1()
        .border_color(rgb(0x232734))
        .child(
            div()
                .flex()
                .items_center()
                .gap_3()
                .child(
                    div()
                        .px_2p5()
                        .py_1()
                        .rounded_md()
                        .bg(rgb(0x1f2433))
                        .text_sm()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(rgb(0xf3f4f6))
                        .child("FOVEA"),
                )
                .child(
                    Button::new("record-toggle")
                        .map(|b| if is_recording { b.danger() } else { b.primary() })
                        .icon(if is_recording {
                            IconName::Square
                        } else {
                            IconName::Play
                        })
                        .label(if is_recording { "Stop" } else { "Record" })
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.toggle_recording(cx);
                        })),
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(rgb(0xcbd5e1))
                        .child(title),
                ),
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap_1p5()
                .child(aspect_btn("ratio-16-9", "16:9", AspectRatio::Ratio16x9, ratio, cx))
                .child(aspect_btn("ratio-9-16", "9:16", AspectRatio::Ratio9x16, ratio, cx))
                .child(aspect_btn("ratio-4-3", "4:3", AspectRatio::Ratio4x3, ratio, cx))
                .child(aspect_btn("ratio-1-1", "1:1", AspectRatio::Ratio1x1, ratio, cx)),
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap_2()
                .child(
                    Button::new("perms-btn")
                        .outline()
                        .icon(if perms_ok {
                            IconName::CircleCheck
                        } else {
                            IconName::TriangleAlert
                        })
                        .label(if perms_ok { "Permissions" } else { "Setup Access" })
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.state.show_permissions = !this.state.show_permissions;
                            cx.notify();
                        })),
                )
                .child(
                    Button::new("undo-btn")
                        .outline()
                        .icon(IconName::Undo2)
                        .disabled(!can_undo)
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.state.undo();
                            cx.notify();
                        })),
                )
                .child(
                    Button::new("redo-btn")
                        .outline()
                        .icon(IconName::Redo2)
                        .disabled(!can_redo)
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.state.redo();
                            cx.notify();
                        })),
                )
                .child(
                    Button::new("export-btn")
                        .primary()
                        .icon(IconName::ExternalLink)
                        .label("Export")
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.state.last_status =
                                "Hardware VideoToolbox export arrives in Milestone 2".into();
                            cx.notify();
                        })),
                ),
        )
}

fn aspect_btn(
    id: &'static str,
    label: &'static str,
    target: AspectRatio,
    current: AspectRatio,
    cx: &mut Context<StudioView>,
) -> impl IntoElement {
    Button::new(id)
        .map(|b| if target == current { b.primary() } else { b.outline() })
        .label(label)
        .on_click(cx.listener(move |this, _, _, cx| {
            let _ = this.state.set_aspect_ratio(target);
            cx.notify();
        }))
}
