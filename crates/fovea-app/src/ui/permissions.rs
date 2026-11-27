use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::IconName;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use fovea_capture::PermissionState;

use crate::ui::studio::StudioView;

pub fn render_permissions_banner(
    view: &StudioView,
    cx: &mut Context<StudioView>,
) -> impl IntoElement {
    let entries = view.state.permissions.entries.clone();

    div()
        .w_full()
        .px_4()
        .py_3()
        .bg(rgb(0x1a1d28))
        .border_b_1()
        .border_color(rgb(0x31384d))
        .flex()
        .flex_col()
        .gap_2()
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(rgb(0xf8fafc))
                        .child("macOS Permissions Setup"),
                )
                .child(
                    div()
                        .flex()
                        .gap_2()
                        .child(
                            Button::new("perm-refresh")
                                .outline()
                                .icon(IconName::RefreshCw)
                                .label("Recheck")
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.refresh_permissions(cx);
                                })),
                        )
                        .child(
                            Button::new("perm-close")
                                .outline()
                                .icon(IconName::Close)
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.state.show_permissions = false;
                                    cx.notify();
                                })),
                        ),
                ),
        )
        .child(
            div()
                .flex()
                .flex_wrap()
                .gap_2()
                .children(entries.into_iter().enumerate().map(|(i, entry)| {
                    let granted = entry.state == PermissionState::Granted;
                    let kind = entry.kind;
                    let suffix = if granted {
                        "Granted"
                    } else if entry.required {
                        "Required"
                    } else {
                        "Optional"
                    };
                    Button::new(("perm-open", i))
                        .map(|b| if granted { b.outline() } else { b.primary() })
                        .icon(if granted {
                            IconName::CircleCheck
                        } else {
                            IconName::ExternalLink
                        })
                        .label(format!("{} ({suffix})", kind.label()))
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.open_permission_settings(kind, cx);
                        }))
                })),
        )
}
