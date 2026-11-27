use gpui_kit::*;

use fovea_core::{EditCommand, FrameStyle};

use crate::state::RightPanelTab;
use crate::ui::ai_chat::render_ai_chat_panel;
use crate::ui::inspector_bg::{render_background_section, section_action_row};
use crate::ui::inspector_system::{light_toggle, render_permissions_tab, render_voice_tab};
use crate::ui::inspector_tabs::{render_cursor_tab, render_webcam_tab};
use crate::ui::pill_slider::pill_slider;
use crate::ui::studio::StudioView;

pub fn render_floating_inspector(
    view: &StudioView,
    cx: &mut Context<StudioView>,
) -> impl IntoElement {
    let tab = view.state.active_tab;

    div()
        .w(px(274.0))
        .h_full()
        .p_3()
        .bg(rgb(0xf8fafc))
        .flex()
        .child(
            div()
                .size_full()
                .p_3p5()
                .rounded(px(16.0))
                .bg(rgb(0xffffff))
                .border_1()
                .border_color(rgb(0xe5e7eb))
                .shadow_md()
                .flex()
                .flex_col()
                .gap_2p5()
                .overflow_hidden()
                .child(match tab {
                    RightPanelTab::Inspector => render_frame_inspector(view, cx).into_any_element(),
                    RightPanelTab::Cursor => render_cursor_tab(view, cx).into_any_element(),
                    RightPanelTab::Webcam => render_webcam_tab(view, cx).into_any_element(),
                    RightPanelTab::VoiceCaptions => render_voice_tab(view, cx).into_any_element(),
                    RightPanelTab::AiChat => render_ai_chat_panel(view, cx).into_any_element(),
                    RightPanelTab::Permissions => {
                        render_permissions_tab(view, cx).into_any_element()
                    }
                }),
        )
}

fn render_frame_inspector(
    view: &StudioView,
    cx: &mut Context<StudioView>,
) -> impl IntoElement {
    let frame = view.state.project().frame.clone();
    let shadow_pct = (frame.shadow_intensity * 100.0).round() as u32;
    let radius_pct = ((frame.corner_radius_px / 36.0).clamp(0.0, 1.0) * 100.0).round() as u32;
    let pad_pct = ((frame.padding_ratio / 0.30).clamp(0.0, 1.0) * 100.0).round() as u32;
    let remove_bg = view.state.remove_background;

    div()
        .flex_1()
        .flex()
        .flex_col()
        .gap_2p5()
        .child(render_background_section(view, cx))
        .child(section_action_row(
            "FRAME",
            "Reset",
            "reset-frame",
            cx.listener(|this, _, _, cx| {
                let f = FrameStyle {
                    background: this.state.project().frame.background.clone(),
                    ..FrameStyle::default()
                };
                let _ = this.state.apply_edit(EditCommand::UpdateFrameStyle(f));
                cx.notify();
            }),
        ))
        .child(pill_slider(
            "slider-shadow",
            "Shadow",
            format!("{shadow_pct}%"),
            frame.shadow_intensity,
            cx.listener(|this, v: &f32, _, cx| {
                let mut f = this.state.project().frame.clone();
                f.shadow_intensity = *v;
                let _ = this.state.apply_edit(EditCommand::UpdateFrameStyle(f));
                cx.notify();
            }),
        ))
        .child(pill_slider(
            "slider-radius",
            "Radius",
            format!("{radius_pct}%"),
            (frame.corner_radius_px / 36.0).clamp(0.0, 1.0),
            cx.listener(|this, v: &f32, _, cx| {
                let mut f = this.state.project().frame.clone();
                f.corner_radius_px = (*v * 36.0).round();
                let _ = this.state.apply_edit(EditCommand::UpdateFrameStyle(f));
                cx.notify();
            }),
        ))
        .child(section_action_row(
            "PADDING",
            "Reset",
            "reset-pad",
            cx.listener(|this, _, _, cx| {
                let mut f = this.state.project().frame.clone();
                f.padding_ratio = 0.14;
                let _ = this.state.apply_edit(EditCommand::UpdateFrameStyle(f));
                cx.notify();
            }),
        ))
        .child(pill_slider(
            "slider-padding",
            "Padding",
            format!("{pad_pct}%"),
            (frame.padding_ratio / 0.30).clamp(0.0, 1.0),
            cx.listener(|this, v: &f32, _, cx| {
                let mut f = this.state.project().frame.clone();
                f.padding_ratio = *v * 0.30;
                let _ = this.state.apply_edit(EditCommand::UpdateFrameStyle(f));
                cx.notify();
            }),
        ))
        .child(light_toggle(
            "toggle-remove-bg",
            "Remove background",
            remove_bg,
            cx.listener(|this, _, _, cx| {
                this.state.remove_background = !this.state.remove_background;
                cx.notify();
            }),
        ))
}
