use fovea_core::{BackgroundStyle, ProjectDoc, ZoomRegion};
use uuid::Uuid;

/// Creates the initial demo project displayed in the Studio Editor before a real
/// `.fovea` recording bundle is loaded, matching a clean 22-second demo timeline.
pub fn build_demo_project(width: u32, height: u32) -> ProjectDoc {
    let mut doc = ProjectDoc::new("recording-demo", width, height, 60);
    doc.source.duration_ms = 22_000;
    doc.clips[0].source_end_ms = 22_000;
    doc.frame.background = BackgroundStyle::Gradient {
        from_hex: "#6366f1".into(),
        to_hex: "#38bdf8".into(),
        angle_deg: 135.0,
    };
    doc.frame.padding_ratio = 0.14;
    doc.frame.corner_radius_px = 14.0;
    doc.frame.shadow_intensity = 0.57;
    doc.zoom_regions = vec![
        ZoomRegion {
            id: Uuid::new_v4(),
            start_ms: 800,
            end_ms: 5_800,
            scale: 1.5,
            focus_x: 0.36,
            focus_y: 0.42,
            auto_generated: true,
        },
        ZoomRegion {
            id: Uuid::new_v4(),
            start_ms: 9_600,
            end_ms: 12_800,
            scale: 1.65,
            focus_x: 0.68,
            focus_y: 0.56,
            auto_generated: true,
        },
        ZoomRegion {
            id: Uuid::new_v4(),
            start_ms: 16_800,
            end_ms: 20_200,
            scale: 1.5,
            focus_x: 0.48,
            focus_y: 0.68,
            auto_generated: true,
        },
    ];
    doc
}

/// Evaluates smooth eased `(scale, focus_x, focus_y)` at `playhead_ms` across zoom regions.
pub fn camera_at_ms(regions: &[ZoomRegion], playhead_ms: u64) -> (f32, f32, f32) {
    const RAMP_MS: f32 = 360.0;
    for z in regions {
        if playhead_ms < z.start_ms || playhead_ms > z.end_ms || z.end_ms <= z.start_ms {
            continue;
        }
        let elapsed = (playhead_ms - z.start_ms) as f32;
        let remain = (z.end_ms - playhead_ms) as f32;
        let dur = (z.end_ms - z.start_ms) as f32;
        let ramp = RAMP_MS.min(dur * 0.45).max(1.0);
        let t_in = (elapsed / ramp).clamp(0.0, 1.0);
        let t_out = (remain / ramp).clamp(0.0, 1.0);
        let ease = smoothstep(t_in.min(t_out));
        let scale = 1.0 + (z.scale - 1.0) * ease;
        let fx = 0.5 + (z.focus_x - 0.5) * ease;
        let fy = 0.5 + (z.focus_y - 0.5) * ease;
        return (scale, fx, fy);
    }
    (1.0, 0.5, 0.5)
}

fn smoothstep(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}
