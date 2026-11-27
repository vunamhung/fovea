use gpui_kit::*;

use fovea_core::EditCommand;

use crate::ui::inspector_system::{light_toggle, section_header};
use crate::ui::pill_slider::pill_slider;
use crate::ui::studio::StudioView;

pub fn render_cursor_tab(view: &StudioView, cx: &mut Context<StudioView>) -> impl IntoElement {
    let cursor = view.state.project().cursor.clone();
    let hud = view.state.project().keystroke_hud.clone();

    div()
        .flex()
        .flex_col()
        .gap_3()
        .child(section_header("CURSOR & MOTION"))
        .child(light_toggle(
            "cur-visible",
            "Show Rendered Cursor",
            cursor.visible,
            cx.listener(|this, _, _, cx| {
                let mut c = this.state.project().cursor.clone();
                c.visible = !c.visible;
                let _ = this.state.apply_edit(EditCommand::UpdateCursorStyle(c));
                cx.notify();
            }),
        ))
        .child(pill_slider(
            "cur-scale",
            "Cursor Size",
            format!("{:.1}x", cursor.scale),
            ((cursor.scale - 0.5) / 2.0).clamp(0.0, 1.0),
            cx.listener(|this, v: &f32, _, cx| {
                let mut c = this.state.project().cursor.clone();
                c.scale = 0.5 + *v * 2.0;
                let _ = this.state.apply_edit(EditCommand::UpdateCursorStyle(c));
                cx.notify();
            }),
        ))
        .child(pill_slider(
            "cur-smooth",
            "Smoothing",
            format!("{}%", (cursor.smoothing * 100.0).round() as u32),
            cursor.smoothing,
            cx.listener(|this, v: &f32, _, cx| {
                let mut c = this.state.project().cursor.clone();
                c.smoothing = *v * 0.95;
                let _ = this.state.apply_edit(EditCommand::UpdateCursorStyle(c));
                cx.notify();
            }),
        ))
        .child(pill_slider(
            "cur-blur",
            "Motion Blur",
            format!("{}%", (cursor.motion_blur * 100.0).round() as u32),
            cursor.motion_blur,
            cx.listener(|this, v: &f32, _, cx| {
                let mut c = this.state.project().cursor.clone();
                c.motion_blur = *v;
                let _ = this.state.apply_edit(EditCommand::UpdateCursorStyle(c));
                cx.notify();
            }),
        ))
        .child(light_toggle(
            "cur-bounce",
            "Click Bounce Effect",
            cursor.click_bounce,
            cx.listener(|this, _, _, cx| {
                let mut c = this.state.project().cursor.clone();
                c.click_bounce = !c.click_bounce;
                let _ = this.state.apply_edit(EditCommand::UpdateCursorStyle(c));
                cx.notify();
            }),
        ))
        .child(section_header("KEYSTROKE HUD"))
        .child(light_toggle(
            "hud-enabled",
            "Show Keystrokes (Secure-Input Safe)",
            hud.enabled,
            cx.listener(|this, _, _, cx| {
                let mut k = this.state.project().keystroke_hud.clone();
                k.enabled = !k.enabled;
                let _ = this.state.apply_edit(EditCommand::UpdateKeystrokeHud(k));
                cx.notify();
            }),
        ))
}

pub fn render_webcam_tab(view: &StudioView, cx: &mut Context<StudioView>) -> impl IntoElement {
    let cam = view.state.project().webcam.clone();

    div()
        .flex()
        .flex_col()
        .gap_3()
        .child(section_header("WEBCAM OVERLAY"))
        .child(light_toggle(
            "cam-enable",
            "Enable Webcam Bubble",
            cam.enabled,
            cx.listener(|this, _, _, cx| {
                let mut w = this.state.project().webcam.clone();
                w.enabled = !w.enabled;
                let _ = this.state.apply_edit(EditCommand::UpdateWebcam(w));
                cx.notify();
            }),
        ))
        .child(light_toggle(
            "cam-mirror",
            "Mirror Video",
            cam.mirror,
            cx.listener(|this, _, _, cx| {
                let mut w = this.state.project().webcam.clone();
                w.mirror = !w.mirror;
                let _ = this.state.apply_edit(EditCommand::UpdateWebcam(w));
                cx.notify();
            }),
        ))
        .child(light_toggle(
            "cam-zoom-react",
            "React to Auto-Zoom",
            cam.zoom_reactive,
            cx.listener(|this, _, _, cx| {
                let mut w = this.state.project().webcam.clone();
                w.zoom_reactive = !w.zoom_reactive;
                let _ = this.state.apply_edit(EditCommand::UpdateWebcam(w));
                cx.notify();
            }),
        ))
        .child(pill_slider(
            "cam-size",
            "Bubble Size",
            format!("{}%", (cam.size_ratio * 100.0).round() as u32),
            (cam.size_ratio / 0.45).clamp(0.0, 1.0),
            cx.listener(|this, v: &f32, _, cx| {
                let mut w = this.state.project().webcam.clone();
                w.size_ratio = (0.10 + *v * 0.35).clamp(0.10, 0.45);
                let _ = this.state.apply_edit(EditCommand::UpdateWebcam(w));
                cx.notify();
            }),
        ))
}
