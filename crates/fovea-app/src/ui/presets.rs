use gpui_kit::*;

use fovea_core::BackgroundStyle;

#[derive(Debug, Clone, Copy)]
pub struct WallpaperPreset {
    pub name: &'static str,
    pub from_hex: &'static str,
    pub to_hex: &'static str,
    pub from_rgb: u32,
    pub to_rgb: u32,
    pub angle_deg: f32,
}

pub const WALLPAPER_PRESETS: &[WallpaperPreset] = &[
    WallpaperPreset { name: "Sonoma Iris", from_hex: "#6366f1", to_hex: "#38bdf8", from_rgb: 0x6366f1, to_rgb: 0x38bdf8, angle_deg: 135.0 },
    WallpaperPreset { name: "Aurora Violet", from_hex: "#4f46e5", to_hex: "#ec4899", from_rgb: 0x4f46e5, to_rgb: 0xec4899, angle_deg: 125.0 },
    WallpaperPreset { name: "Obsidian Glow", from_hex: "#0f172a", to_hex: "#334155", from_rgb: 0x0f172a, to_rgb: 0x334155, angle_deg: 145.0 },
    WallpaperPreset { name: "Coral Sunset", from_hex: "#f97316", to_hex: "#ec4899", from_rgb: 0xf97316, to_rgb: 0xec4899, angle_deg: 130.0 },
    WallpaperPreset { name: "Ocean Breeze", from_hex: "#0284c7", to_hex: "#2dd4bf", from_rgb: 0x0284c7, to_rgb: 0x2dd4bf, angle_deg: 135.0 },
    WallpaperPreset { name: "Peach Silk", from_hex: "#fb7185", to_hex: "#fbbf24", from_rgb: 0xfb7185, to_rgb: 0xfbbf24, angle_deg: 120.0 },
    WallpaperPreset { name: "Emerald Mist", from_hex: "#059669", to_hex: "#34d399", from_rgb: 0x059669, to_rgb: 0x34d399, angle_deg: 140.0 },
    WallpaperPreset { name: "Royal Dusk", from_hex: "#1d4ed8", to_hex: "#7c3aed", from_rgb: 0x1d4ed8, to_rgb: 0x7c3aed, angle_deg: 135.0 },
    WallpaperPreset { name: "Magenta Bloom", from_hex: "#9333ea", to_hex: "#f43f5e", from_rgb: 0x9333ea, to_rgb: 0xf43f5e, angle_deg: 125.0 },
    WallpaperPreset { name: "Arctic Glass", from_hex: "#bae6fd", to_hex: "#c7d2fe", from_rgb: 0xbae6fd, to_rgb: 0xc7d2fe, angle_deg: 135.0 },
    WallpaperPreset { name: "Forest Velvet", from_hex: "#064e3b", to_hex: "#0d9488", from_rgb: 0x064e3b, to_rgb: 0x0d9488, angle_deg: 145.0 },
    WallpaperPreset { name: "Studio Pearl", from_hex: "#e2e8f0", to_hex: "#cbd5e1", from_rgb: 0xe2e8f0, to_rgb: 0xcbd5e1, angle_deg: 150.0 },
    WallpaperPreset { name: "Cobalt Wave", from_hex: "#1e3a8a", to_hex: "#06b6d4", from_rgb: 0x1e3a8a, to_rgb: 0x06b6d4, angle_deg: 130.0 },
    WallpaperPreset { name: "Citrus Mint", from_hex: "#84cc16", to_hex: "#14b8a6", from_rgb: 0x84cc16, to_rgb: 0x14b8a6, angle_deg: 125.0 },
    WallpaperPreset { name: "Crimson Flame", from_hex: "#b91c1c", to_hex: "#f59e0b", from_rgb: 0xb91c1c, to_rgb: 0xf59e0b, angle_deg: 140.0 },
    WallpaperPreset { name: "Lavender Sky", from_hex: "#a855f7", to_hex: "#38bdf8", from_rgb: 0xa855f7, to_rgb: 0x38bdf8, angle_deg: 135.0 },
    WallpaperPreset { name: "Midnight Plum", from_hex: "#312e81", to_hex: "#9d174d", from_rgb: 0x312e81, to_rgb: 0x9d174d, angle_deg: 150.0 },
    WallpaperPreset { name: "Silver Satin", from_hex: "#f8fafc", to_hex: "#93c5fd", from_rgb: 0xf8fafc, to_rgb: 0x93c5fd, angle_deg: 135.0 },
];

pub const SOLID_COLORS: &[(&str, u32)] = &[
    ("#0f172a", 0x0f172a),
    ("#1e293b", 0x1e293b),
    ("#2563eb", 0x2563eb),
    ("#4f46e5", 0x4f46e5),
    ("#7c3aed", 0x7c3aed),
    ("#db2777", 0xdb2777),
    ("#e11d48", 0xe11d48),
    ("#ea580c", 0xea580c),
    ("#d97706", 0xd97706),
    ("#059669", 0x059669),
    ("#0d9488", 0x0d9488),
    ("#0284c7", 0x0284c7),
    ("#e2e8f0", 0xe2e8f0),
    ("#f8fafc", 0xf8fafc),
    ("#fef3c7", 0xfef3c7),
    ("#dbeafe", 0xdbeafe),
    ("#f3e8ff", 0xf3e8ff),
    ("#dcfce7", 0xdcfce7),
];

pub fn parse_hex_rgb(hex: &str, fallback: u32) -> u32 {
    u32::from_str_radix(hex.trim_start_matches('#'), 16).unwrap_or(fallback)
}

pub fn background_to_gpui(bg: &BackgroundStyle) -> Background {
    match bg {
        BackgroundStyle::Solid { hex } => rgb(parse_hex_rgb(hex, 0x1e293b)).into(),
        BackgroundStyle::Gradient {
            from_hex,
            to_hex,
            angle_deg,
        } => {
            let c1 = rgb(parse_hex_rgb(from_hex, 0x6366f1));
            let c2 = rgb(parse_hex_rgb(to_hex, 0x38bdf8));
            linear_gradient(
                *angle_deg,
                linear_color_stop(c1, 0.0),
                linear_color_stop(c2, 1.0),
            )
        }
        BackgroundStyle::Image { .. } | BackgroundStyle::Video { .. } => linear_gradient(
            135.0,
            linear_color_stop(rgb(0x6366f1), 0.0),
            linear_color_stop(rgb(0x38bdf8), 1.0),
        ),
    }
}
