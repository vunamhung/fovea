use gpui_kit::component::IconName;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use fovea_capture::PermissionState;

use crate::ui::studio::StudioView;

pub fn render_voice_tab(_view: &StudioView, _cx: &mut Context<StudioView>) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap_3()
        .child(section_header("VBEE NEURAL VOICEOVER"))
        .child(info_card(
            "Vbee TTS (vbee.vn)",
            "Generates synchronized Vietnamese & English narration with automatic audio ducking and optional freeze-frame video sync. Wired in Milestone 3.",
        ))
        .child(section_header("AUTO-CAPTIONS"))
        .child(info_card(
            "On-Demand Local Whisper (CoreML)",
            "Downloads model only on first use to keep Fovea ~14 MB. Produces word-level timestamps offline.",
        ))
}

pub fn render_permissions_tab(view: &StudioView, cx: &mut Context<StudioView>) -> impl IntoElement {
    let entries = view.state.permissions.entries.clone();
    div()
        .flex()
        .flex_col()
        .gap_2()
        .child(section_header("MACOS PERMISSIONS"))
        .child(
            div()
                .text_xs()
                .text_color(rgb(0x64748b))
                .child("Click a permission to trigger macOS prompt and toggle Fovea ON in Settings."),
        )
        .children(entries.into_iter().enumerate().map(|(i, e)| {
            let granted = e.state == PermissionState::Granted;
            let kind = e.kind;
            div()
                .id(("perm-row", i))
                .px_3()
                .py_2()
                .rounded(px(10.0))
                .bg(rgb(0xf8fafc))
                .border_1()
                .border_color(rgb(0xe2e8f0))
                .flex()
                .items_center()
                .justify_between()
                .cursor_pointer()
                .child(
                    div()
                        .text_xs()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(rgb(0x1e293b))
                        .child(kind.label()),
                )
                .child(
                    div()
                        .px_2()
                        .py_0p5()
                        .rounded(px(6.0))
                        .bg(if granted { rgb(0xdcfce7) } else { rgb(0xeff6ff) })
                        .text_xs()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(if granted { rgb(0x15803d) } else { rgb(0x2563eb) })
                        .child(if granted { "Granted" } else { "Request" }),
                )
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.open_permission_settings(kind, cx);
                }))
        }))
        .child(
            div()
                .id("perm-recheck-card")
                .mt_1()
                .h(px(32.0))
                .rounded(px(9.0))
                .bg(rgb(0x2563eb))
                .flex()
                .items_center()
                .justify_center()
                .gap_1p5()
                .cursor_pointer()
                .text_xs()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(rgb(0xffffff))
                .child(IconName::RefreshCw)
                .child("Recheck Permissions")
                .on_click(cx.listener(|this, _, _, cx| {
                    this.refresh_permissions(cx);
                })),
        )
}

pub fn section_header(title: &'static str) -> impl IntoElement {
    div()
        .text_xs()
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(rgb(0x64748b))
        .child(title)
}

pub fn info_card(title: &'static str, body: &'static str) -> impl IntoElement {
    div()
        .p_3()
        .rounded(px(11.0))
        .bg(rgb(0xf8fafc))
        .border_1()
        .border_color(rgb(0xe2e8f0))
        .flex()
        .flex_col()
        .gap_1()
        .child(
            div()
                .text_xs()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(rgb(0x0f172a))
                .child(title),
        )
        .child(div().text_xs().text_color(rgb(0x64748b)).child(body))
}

pub fn light_toggle(
    id: &'static str,
    label: &'static str,
    enabled: bool,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    div()
        .id(id)
        .h(px(32.0))
        .px_3()
        .rounded(px(9.0))
        .bg(rgb(0xf8fafc))
        .border_1()
        .border_color(rgb(0xe8eaef))
        .flex()
        .items_center()
        .justify_between()
        .cursor_pointer()
        .child(
            div()
                .text_xs()
                .font_weight(FontWeight::MEDIUM)
                .text_color(rgb(0x334155))
                .child(label),
        )
        .child(
            div()
                .w(px(32.0))
                .h(px(18.0))
                .p(px(2.0))
                .rounded_full()
                .bg(if enabled { rgb(0x2563eb) } else { rgb(0xcbd5e1) })
                .flex()
                .items_center()
                .when(enabled, |d| d.justify_end())
                .child(div().w(px(14.0)).h(px(14.0)).rounded_full().bg(rgb(0xffffff))),
        )
        .on_click(on_click)
}
