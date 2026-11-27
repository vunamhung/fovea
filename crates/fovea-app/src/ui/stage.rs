use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use fovea_core::{AnnotationKind, AspectRatio};

use crate::demo::camera_at_ms;
use crate::ui::presets::background_to_gpui;
use crate::ui::stage_parts::{render_sample_macos_window, render_stage_toolbar, render_webcam_bubble};
use crate::ui::studio::StudioView;

pub fn render_stage(view: &StudioView, cx: &mut Context<StudioView>) -> impl IntoElement {
    let project = view.state.project();
    let frame = &project.frame;
    let playhead_ms = view.state.playhead_ms;
    let (zoom_scale, focus_x, focus_y) = camera_at_ms(&project.zoom_regions, playhead_ms);
    let active_zoom = (zoom_scale > 1.02).then_some(zoom_scale);

    let ratio_label = match frame.aspect_ratio {
        AspectRatio::Ratio16x9 => "16:9",
        AspectRatio::Ratio9x16 => "9:16",
        AspectRatio::Ratio4x3 => "4:3",
        AspectRatio::Ratio1x1 => "1:1",
        AspectRatio::Source => "Source",
    };
    let (canvas_w, canvas_h) = match frame.aspect_ratio {
        AspectRatio::Ratio9x16 => (280.0_f32, 460.0_f32),
        AspectRatio::Ratio1x1 => (440.0, 440.0),
        AspectRatio::Ratio4x3 => (600.0, 450.0),
        _ => (780.0, 438.0),
    };

    let pad_px = if view.state.remove_background {
        0.0
    } else {
        (canvas_h * frame.padding_ratio.clamp(0.0, 0.30)).round()
    };
    let inner_w = (canvas_w - pad_px * 2.0).max(80.0);
    let inner_h = (canvas_h - pad_px * 2.0).max(60.0);
    let zoomed_w = inner_w * zoom_scale;
    let zoomed_h = inner_h * zoom_scale;
    let offset_x = (inner_w * 0.5 - zoomed_w * focus_x).clamp(inner_w - zoomed_w, 0.0);
    let offset_y = (inner_h * 0.5 - zoomed_h * focus_y).clamp(inner_h - zoomed_h, 0.0);

    let shadow_alpha = (frame.shadow_intensity.clamp(0.0, 1.0) * 0.42).max(0.02);
    let shadow_blur = px(12.0 + frame.shadow_intensity * 36.0);
    let shadow_y = px(4.0 + frame.shadow_intensity * 16.0);
    let bg_fill = if view.state.remove_background {
        rgb(0xf1f5f9).into()
    } else {
        background_to_gpui(&frame.background)
    };
    let blur_alpha = (view.state.bg_blur_px / 32.0).clamp(0.0, 1.0) * 0.45;
    let cursor = &project.cursor;
    let webcam = &project.webcam;
    let blurs: Vec<(f32, f32, f32, f32)> = project
        .annotations
        .iter()
        .filter(|a| {
            playhead_ms >= a.start_ms
                && playhead_ms <= a.end_ms
                && matches!(a.payload, AnnotationKind::PrivacyBlur { .. })
        })
        .map(|a| (a.x, a.y, a.width, a.height))
        .collect();
    let show_toast = view.state.last_status != "Ready";
    let toast_text = view.state.last_status.clone();

    div()
        .relative()
        .flex_1()
        .h_full()
        .bg(rgb(0xf8fafc))
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap_3()
        .px_5()
        .py_3()
        .child(render_stage_toolbar(ratio_label, view.state.crop_mode, active_zoom, cx))
        .child(
            div()
                .relative()
                .w(px(canvas_w))
                .h(px(canvas_h))
                .rounded(px(12.0))
                .bg(bg_fill)
                .overflow_hidden()
                .when_some(view.custom_bg_image.clone(), |d, bg_img| {
                    d.when(!view.state.remove_background, |d| {
                        d.child(
                            img(bg_img)
                                .absolute()
                                .inset_0()
                                .size_full()
                                .object_fit(ObjectFit::Cover),
                        )
                    })
                })
                .when(blur_alpha > 0.01 && !view.state.remove_background, |d| {
                    d.child(div().absolute().inset_0().bg(hsla(0.0, 0.0, 1.0, blur_alpha)))
                })
                .child(
                    div()
                        .relative()
                        .size_full()
                        .p(px(pad_px))
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(
                            div()
                                .relative()
                                .w(px(inner_w))
                                .h(px(inner_h))
                                .rounded(px(frame.corner_radius_px))
                                .overflow_hidden()
                                .bg(rgb(0xffffff))
                                .shadow(vec![BoxShadow {
                                    color: hsla(0.0, 0.0, 0.0, shadow_alpha),
                                    offset: point(px(0.0), shadow_y),
                                    blur_radius: shadow_blur,
                                    spread_radius: px(0.0),
                                    inset: false,
                                }])
                                .child(
                                    div()
                                        .absolute()
                                        .left(px(offset_x))
                                        .top(px(offset_y))
                                        .w(px(zoomed_w))
                                        .h(px(zoomed_h))
                                        .child(if let Some(ref frame_img) = view.preview_frame {
                                            img(frame_img.clone())
                                                .size_full()
                                                .object_fit(ObjectFit::Cover)
                                                .into_any_element()
                                        } else {
                                            render_sample_macos_window(zoom_scale).into_any_element()
                                        })
                                        .when(cursor.visible, |d| {
                                            let sz = (14.0 * cursor.scale).clamp(10.0, 34.0);
                                            d.child(
                                                div()
                                                    .absolute()
                                                    .left(px(zoomed_w * focus_x - sz * 0.5))
                                                    .top(px(zoomed_h * focus_y - sz * 0.5))
                                                    .w(px(sz))
                                                    .h(px(sz))
                                                    .rounded_full()
                                                    .bg(rgb(0x111827))
                                                    .border_2()
                                                    .border_color(if zoom_scale > 1.15 {
                                                        rgb(0x60a5fa)
                                                    } else {
                                                        rgb(0xffffff)
                                                    }),
                                            )
                                        })
                                        .children(blurs.into_iter().map(|(bx, by, bw, bh)| {
                                            div()
                                                .absolute()
                                                .left(px(zoomed_w * bx))
                                                .top(px(zoomed_h * by))
                                                .w(px(zoomed_w * bw))
                                                .h(px(zoomed_h * bh))
                                                .rounded(px(8.0))
                                                .bg(rgba(0x0f172ad8))
                                                .border_1()
                                                .border_color(rgba(0xffffff33))
                                                .flex()
                                                .items_center()
                                                .justify_center()
                                                .text_xs()
                                                .text_color(rgb(0xe2e8f0))
                                                .child("REDACTED")
                                        })),
                                )
                                .when(view.state.crop_mode, |d| {
                                    d.child(
                                        div()
                                            .absolute()
                                            .inset_3()
                                            .rounded(px(8.0))
                                            .border_2()
                                            .border_color(rgb(0x2563eb)),
                                    )
                                }),
                        ),
                )
                .when(webcam.enabled, |d| {
                    let react = if webcam.zoom_reactive && zoom_scale > 1.05 { 0.72 } else { 1.0 };
                    let cam_sz = (canvas_h * webcam.size_ratio * react).clamp(44.0, 150.0);
                    d.child(render_webcam_bubble(cam_sz))
                }),
        )
        .when(show_toast, |d| {
            d.child(
                div()
                    .id("stage-toast")
                    .px_3()
                    .py_1()
                    .rounded_full()
                    .bg(rgb(0x0f172a))
                    .text_xs()
                    .text_color(rgb(0xffffff))
                    .cursor_pointer()
                    .child(toast_text)
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.state.last_status = "Ready".into();
                        cx.notify();
                    })),
            )
        })
}
