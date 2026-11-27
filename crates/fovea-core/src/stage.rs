use crate::physics::CameraPose;
use crate::style::{CropRect, FrameStyle};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RectF {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StageTransform {
    pub viewport_w: f32,
    pub viewport_h: f32,
    pub canvas_rect: RectF,
    pub content_rect: RectF,
    pub crop: CropRect,
    pub camera_scale: f32,
    pub focus_x: f32,
    pub focus_y: f32,
}

impl StageTransform {
    pub fn compute(
        viewport_w: f32,
        viewport_h: f32,
        source_w: u32,
        source_h: u32,
        frame: &FrameStyle,
        camera: &CameraPose,
    ) -> Self {
        let vw = viewport_w.max(1.0);
        let vh = viewport_h.max(1.0);
        let target_aspect = frame.aspect_ratio.value(source_w, source_h).max(0.1);
        let viewport_aspect = vw / vh;

        let (cw, ch) = if viewport_aspect > target_aspect {
            (vh * target_aspect, vh)
        } else {
            (vw, vw / target_aspect)
        };
        let canvas_rect = RectF {
            x: (vw - cw) * 0.5,
            y: (vh - ch) * 0.5,
            width: cw,
            height: ch,
        };

        let pad = frame.padding_ratio.clamp(0.0, 0.4);
        let inner_w = cw * (1.0 - 2.0 * pad);
        let inner_h = ch * (1.0 - 2.0 * pad);

        let crop_w = frame.crop.width.clamp(0.05, 1.0);
        let crop_h = frame.crop.height.clamp(0.05, 1.0);
        let cropped_aspect =
            (source_w.max(1) as f32 * crop_w) / (source_h.max(1) as f32 * crop_h);
        let inner_aspect = inner_w / inner_h.max(1.0);

        let (content_w, content_h) = if inner_aspect > cropped_aspect {
            (inner_h * cropped_aspect, inner_h)
        } else {
            (inner_w, inner_w / cropped_aspect)
        };
        let content_rect = RectF {
            x: canvas_rect.x + (cw - content_w) * 0.5,
            y: canvas_rect.y + (ch - content_h) * 0.5,
            width: content_w,
            height: content_h,
        };

        Self {
            viewport_w: vw,
            viewport_h: vh,
            canvas_rect,
            content_rect,
            crop: frame.crop.clone(),
            camera_scale: camera.scale.max(1.0),
            focus_x: camera.focus_x.clamp(0.0, 1.0),
            focus_y: camera.focus_y.clamp(0.0, 1.0),
        }
    }

    pub fn source_to_stage_point(&self, norm_x: f32, norm_y: f32) -> (f32, f32) {
        let crop_w = self.crop.width.max(1e-4);
        let crop_h = self.crop.height.max(1e-4);
        let cx = (norm_x - self.crop.x) / crop_w;
        let cy = (norm_y - self.crop.y) / crop_h;

        let zx = 0.5 + (cx - self.focus_x) * self.camera_scale;
        let zy = 0.5 + (cy - self.focus_y) * self.camera_scale;

        let sx = self.content_rect.x + zx * self.content_rect.width;
        let sy = self.content_rect.y + zy * self.content_rect.height;
        (sx, sy)
    }

    pub fn stage_to_source_point(&self, stage_x: f32, stage_y: f32) -> (f32, f32) {
        let zx = (stage_x - self.content_rect.x) / self.content_rect.width.max(1e-4);
        let zy = (stage_y - self.content_rect.y) / self.content_rect.height.max(1e-4);

        let cx = self.focus_x + (zx - 0.5) / self.camera_scale;
        let cy = self.focus_y + (zy - 0.5) / self.camera_scale;

        let nx = self.crop.x + cx * self.crop.width;
        let ny = self.crop.y + cy * self.crop.height;
        (nx, ny)
    }
}
