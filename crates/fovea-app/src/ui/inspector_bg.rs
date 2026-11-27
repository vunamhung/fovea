use gpui_kit::component::IconName;
use gpui_kit::*;

use fovea_core::BackgroundStyle;
use fovea_core::EditCommand;

use crate::state::BackgroundTab;
use crate::ui::pill_slider::pill_slider;
use crate::ui::presets::{SOLID_COLORS, WALLPAPER_PRESETS};
use crate::ui::studio::StudioView;

pub fn render_background_section(
    view: &StudioView,
    cx: &mut Context<StudioView>,
) -> impl IntoElement {
    let bg_tab = view.state.bg_tab;
    let selected_preset = view.state.selected_preset_idx;
    let blur_px = view.state.bg_blur_px;

    div()
        .flex()
        .flex_col()
        .gap_2()
        .child(section_action_row(
            "BACKGROUND",
            "Reset",
            "reset-bg",
            cx.listener(|this, _, _, cx| {
                this.state.bg_blur_px = 0.0;
                this.state.remove_background = false;
                this.apply_preset_index(0, cx);
            }),
        ))
        .child(pill_slider(
            "slider-bg-blur",
            "Blur",
            format!("{blur_px:.1}px"),
            (blur_px / 32.0).clamp(0.0, 1.0),
            cx.listener(|this, v: &f32, _, cx| {
                this.state.bg_blur_px = (*v * 32.0).round();
                cx.notify();
            }),
        ))
        .child(
            div()
                .p(px(3.0))
                .rounded(px(9.0))
                .bg(rgb(0xf1f5f9))
                .flex()
                .gap_1()
                .child(seg_btn("bg-img", "Image", BackgroundTab::Image, bg_tab, cx))
                .child(seg_btn("bg-vid", "Video", BackgroundTab::Video, bg_tab, cx))
                .child(seg_btn("bg-col", "Color", BackgroundTab::Color, bg_tab, cx))
                .child(seg_btn("bg-grd", "Gradient", BackgroundTab::Gradient, bg_tab, cx)),
        )
        .child(
            div()
                .id("upload-custom-bg")
                .h(px(28.0))
                .w_full()
                .rounded(px(8.0))
                .bg(rgb(0xf8fafc))
                .border_1()
                .border_color(rgb(0xe2e8f0))
                .flex()
                .items_center()
                .justify_center()
                .gap_1p5()
                .cursor_pointer()
                .text_xs()
                .font_weight(FontWeight::MEDIUM)
                .text_color(rgb(0x334155))
                .child(IconName::FolderOpen)
                .child("Upload Custom")
                .on_click(cx.listener(|this, _, _, cx| {
                    this.state.last_status = "Custom wallpaper picker arrives in Milestone 2".into();
                    cx.notify();
                })),
        )
        .child(render_swatch_grid(bg_tab, selected_preset, cx))
}

fn render_swatch_grid(
    tab: BackgroundTab,
    selected_idx: usize,
    cx: &mut Context<StudioView>,
) -> impl IntoElement {
    let mut grid = div().flex().flex_col().gap(px(5.0));
    for range in [0..6_usize, 6..12, 12..18] {
        let mut row = div().flex().gap(px(5.0));
        for idx in range {
            let is_sel = idx == selected_idx;
            let fill: Background = if tab == BackgroundTab::Color {
                let (_, color) = SOLID_COLORS[idx % SOLID_COLORS.len()];
                rgb(color).into()
            } else {
                let p = WALLPAPER_PRESETS[idx % WALLPAPER_PRESETS.len()];
                linear_gradient(
                    p.angle_deg,
                    linear_color_stop(rgb(p.from_rgb), 0.0),
                    linear_color_stop(rgb(p.to_rgb), 1.0),
                )
            };
            row = row.child(
                div()
                    .id(("swatch", idx))
                    .flex_1()
                    .h(px(28.0))
                    .rounded(px(7.0))
                    .bg(fill)
                    .border_2()
                    .border_color(if is_sel { rgb(0x2563eb) } else { rgba(0x00000014) })
                    .cursor_pointer()
                    .on_click(cx.listener(move |this, _, _, cx| {
                        if this.state.bg_tab == BackgroundTab::Color {
                            let (hex, _) = SOLID_COLORS[idx % SOLID_COLORS.len()];
                            let mut f = this.state.project().frame.clone();
                            f.background = BackgroundStyle::Solid { hex: hex.into() };
                            this.state.selected_preset_idx = idx;
                            let _ = this.state.apply_edit(EditCommand::UpdateFrameStyle(f));
                            cx.notify();
                        } else {
                            this.apply_preset_index(idx, cx);
                        }
                    })),
            );
        }
        grid = grid.child(row);
    }
    grid
}

pub fn section_action_row(
    title: &'static str,
    action: &'static str,
    action_id: &'static str,
    on_action: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .justify_between()
        .child(
            div()
                .text_xs()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(rgb(0x64748b))
                .child(title),
        )
        .child(
            div()
                .id(action_id)
                .cursor_pointer()
                .text_xs()
                .font_weight(FontWeight::MEDIUM)
                .text_color(rgb(0x2563eb))
                .child(action)
                .on_click(on_action),
        )
}

fn seg_btn(
    id: &'static str,
    label: &'static str,
    target: BackgroundTab,
    current: BackgroundTab,
    cx: &mut Context<StudioView>,
) -> impl IntoElement {
    let active = target == current;
    div()
        .id(id)
        .flex_1()
        .h(px(24.0))
        .rounded(px(7.0))
        .bg(if active { rgb(0x2563eb) } else { rgba(0x00000000) })
        .flex()
        .items_center()
        .justify_center()
        .cursor_pointer()
        .text_xs()
        .font_weight(FontWeight::MEDIUM)
        .text_color(if active { rgb(0xffffff) } else { rgb(0x475569) })
        .child(label)
        .on_click(cx.listener(move |this, _, _, cx| {
            this.state.bg_tab = target;
            cx.notify();
        }))
}
