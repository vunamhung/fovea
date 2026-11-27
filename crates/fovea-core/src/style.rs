use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CropRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Default for CropRect {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            width: 1.0,
            height: 1.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AspectRatio {
    Ratio16x9,
    Ratio9x16,
    Ratio4x3,
    Ratio1x1,
    Source,
}

impl AspectRatio {
    pub fn value(self, source_width: u32, source_height: u32) -> f32 {
        match self {
            Self::Ratio16x9 => 16.0 / 9.0,
            Self::Ratio9x16 => 9.0 / 16.0,
            Self::Ratio4x3 => 4.0 / 3.0,
            Self::Ratio1x1 => 1.0,
            Self::Source => source_width.max(1) as f32 / source_height.max(1) as f32,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum BackgroundStyle {
    Solid { hex: String },
    Gradient { from_hex: String, to_hex: String, angle_deg: f32 },
    Image { path: String, blur_radius: f32 },
    Video { path: String, blur_radius: f32 },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FrameStyle {
    pub background: BackgroundStyle,
    pub aspect_ratio: AspectRatio,
    pub padding_ratio: f32,
    pub corner_radius_px: f32,
    pub shadow_intensity: f32,
    pub crop: CropRect,
}

impl Default for FrameStyle {
    fn default() -> Self {
        Self {
            background: BackgroundStyle::Gradient {
                from_hex: "#111827".into(),
                to_hex: "#1e293b".into(),
                angle_deg: 135.0,
            },
            aspect_ratio: AspectRatio::Ratio16x9,
            padding_ratio: 0.08,
            corner_radius_px: 18.0,
            shadow_intensity: 0.55,
            crop: CropRect::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CursorStyle {
    pub visible: bool,
    pub scale: f32,
    pub smoothing: f32,
    pub click_bounce: bool,
    pub motion_blur: f32,
    pub sway: f32,
    pub idle_hide_ms: Option<u64>,
}

impl Default for CursorStyle {
    fn default() -> Self {
        Self {
            visible: true,
            scale: 1.25,
            smoothing: 0.65,
            click_bounce: true,
            motion_blur: 0.35,
            sway: 0.15,
            idle_hide_ms: Some(3000),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KeystrokeHudStyle {
    pub enabled: bool,
    pub shortcuts_only: bool,
    pub duration_ms: u64,
    pub anchor_x: f32,
    pub anchor_y: f32,
}

impl Default for KeystrokeHudStyle {
    fn default() -> Self {
        Self {
            enabled: true,
            shortcuts_only: true,
            duration_ms: 1400,
            anchor_x: 0.5,
            anchor_y: 0.88,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WebcamConfig {
    pub enabled: bool,
    pub mirror: bool,
    pub zoom_reactive: bool,
    pub x: f32,
    pub y: f32,
    pub size_ratio: f32,
    pub corner_radius_ratio: f32,
    pub shadow_intensity: f32,
}

impl Default for WebcamConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            mirror: true,
            zoom_reactive: true,
            x: 0.86,
            y: 0.82,
            size_ratio: 0.22,
            corner_radius_ratio: 0.5,
            shadow_intensity: 0.5,
        }
    }
}
