use fovea_core::{
    AnnotationKind, BackgroundStyle, ProjectDoc, TelemetryEvent, camera_at_ms, sample_cursor_at_ms,
};

use crate::frame_decoder::DecodedFrame;

/// Composites a single output video frame into a tightly packed BGRA buffer (`out_w * out_h * 4`).
/// Matches the live Studio Stage look: gradient/solid/image backdrop, padding, corner radius,
/// drop shadow, crop, smooth camera zoom/pan, rendered cursor, and privacy blur boxes.
#[allow(clippy::too_many_arguments)]
pub fn composite_frame(
    project: &ProjectDoc,
    source_bgra: Option<&DecodedFrame>,
    custom_bg: Option<&DecodedFrame>,
    telemetry: &[TelemetryEvent],
    out_ms: u64,
    out_w: u32,
    out_h: u32,
    remove_bg: bool,
) -> Vec<u8> {
    let w = out_w.max(2) as usize;
    let h = out_h.max(2) as usize;
    let mut buf = vec![0_u8; w * h * 4];

    fill_background(&mut buf, w, h, &project.frame.background, custom_bg, remove_bg);

    let pad = if remove_bg {
        0.0_f32
    } else {
        (h as f32 * project.frame.padding_ratio.clamp(0.0, 0.30)).round()
    };
    let rx0 = pad;
    let ry0 = pad;
    let rw = (w as f32 - pad * 2.0).max(16.0);
    let rh = (h as f32 - pad * 2.0).max(16.0);
    let radius = (project.frame.corner_radius_px * (h as f32 / 440.0)).min(rw * 0.45).min(rh * 0.45);

    if !remove_bg && project.frame.shadow_intensity > 0.02 {
        draw_soft_shadow(&mut buf, (w, h), (rx0, ry0, rw, rh), radius, project.frame.shadow_intensity);
    }

    let (scale, fx, fy) = camera_at_ms(&project.zoom_regions, out_ms);
    let crop = &project.frame.crop;
    let cw = crop.width.clamp(0.1, 1.0);
    let ch = crop.height.clamp(0.1, 1.0);
    let cx0 = crop.x.clamp(0.0, 1.0 - cw);
    let cy0 = crop.y.clamp(0.0, 1.0 - ch);

    let x_start = (rx0.floor() as isize).max(0) as usize;
    let y_start = (ry0.floor() as isize).max(0) as usize;
    let x_end = ((rx0 + rw).ceil() as usize).min(w);
    let y_end = ((ry0 + rh).ceil() as usize).min(h);

    for py in y_start..y_end {
        let local_y = py as f32 + 0.5 - ry0;
        let ny = (local_y / rh).clamp(0.0, 1.0);
        for px in x_start..x_end {
            let local_x = px as f32 + 0.5 - rx0;
            let cov = rounded_rect_coverage(local_x, local_y, rw, rh, radius);
            if cov <= 0.0 {
                continue;
            }
            let nx = (local_x / rw).clamp(0.0, 1.0);
            // Apply smooth camera zoom + pan centered on (fx, fy), clamped inside [0, 1].
            let z_x = (fx + (nx - 0.5) / scale).clamp(0.0, 1.0);
            let z_y = (fy + (ny - 0.5) / scale).clamp(0.0, 1.0);
            let src_u = (cx0 + z_x * cw).clamp(0.0, 1.0);
            let src_v = (cy0 + z_y * ch).clamp(0.0, 1.0);

            let fg = if let Some(src) = source_bgra {
                sample_bgra(src, src_u, src_v)
            } else {
                sample_demo_card(src_u, src_v)
            };
            blend_pixel(&mut buf, (py * w + px) * 4, fg, cov);
        }
    }

    for ann in &project.annotations {
        if out_ms >= ann.start_ms
            && out_ms <= ann.end_ms
            && matches!(ann.payload, AnnotationKind::PrivacyBlur { .. })
        {
            draw_privacy_box(&mut buf, (w, h), (rx0, ry0, rw, rh), (ann.x, ann.y, ann.width, ann.height));
        }
    }

    if project.cursor.visible {
        let (cur_x, cur_y, clicked) =
            sample_cursor_at_ms(telemetry, out_ms).unwrap_or((fx, fy, scale > 1.15));
        let screen_x = rx0 + ((cur_x - fx) * scale + 0.5).clamp(0.0, 1.0) * rw;
        let screen_y = ry0 + ((cur_y - fy) * scale + 0.5).clamp(0.0, 1.0) * rh;
        let rad = (9.0 * project.cursor.scale * (h as f32 / 720.0)).clamp(5.0, 26.0);
        draw_cursor_dot(&mut buf, w, h, screen_x, screen_y, rad, clicked);
    }

    buf
}

fn fill_background(buf: &mut [u8], w: usize, h: usize, bg: &BackgroundStyle, custom_bg: Option<&DecodedFrame>, remove_bg: bool) {
    if remove_bg {
        for px in buf.as_chunks_mut::<4>().0 { px.copy_from_slice(&[249, 245, 241, 255]); }
        return;
    }
    if let Some(img) = custom_bg {
        for y in 0..h {
            let v = y as f32 / h as f32;
            for x in 0..w {
                let c = sample_bgra(img, x as f32 / w as f32, v);
                let i = (y * w + x) * 4;
                buf[i..i + 4].copy_from_slice(&c);
            }
        }
        return;
    }
    let (c1, c2) = match bg {
        BackgroundStyle::Solid { hex } => { let c = parse_hex_bgra(hex, [0x3b, 0x29, 0x1e, 255]); (c, c) }
        BackgroundStyle::Gradient { from_hex, to_hex, .. } => (
            parse_hex_bgra(from_hex, [0xf1, 0x66, 0x63, 255]),
            parse_hex_bgra(to_hex, [0xf8, 0xbd, 0x38, 255]),
        ),
        _ => ([0xf1, 0x66, 0x63, 255], [0xf8, 0xbd, 0x38, 255]),
    };
    let inv = 1.0 / (w + h).max(1) as f32;
    for y in 0..h {
        for x in 0..w {
            let t = ((x + y) as f32 * inv).clamp(0.0, 1.0);
            let i = (y * w + x) * 4;
            buf[i] = lerp_u8(c1[0], c2[0], t);
            buf[i + 1] = lerp_u8(c1[1], c2[1], t);
            buf[i + 2] = lerp_u8(c1[2], c2[2], t);
            buf[i + 3] = 255;
        }
    }
}

fn draw_soft_shadow(
    buf: &mut [u8],
    dims: (usize, usize),
    rect: (f32, f32, f32, f32),
    radius: f32,
    intensity: f32,
) {
    let (w, h) = dims;
    let (rx0, ry0, rw, rh) = rect;
    let spread = (18.0 + intensity * 34.0) * (h as f32 / 720.0);
    let y_off = (6.0 + intensity * 14.0) * (h as f32 / 720.0);
    let max_alpha = (intensity * 0.36).clamp(0.0, 0.42);
    let x0 = ((rx0 - spread).floor() as isize).max(0) as usize;
    let y0 = ((ry0 - spread + y_off).floor() as isize).max(0) as usize;
    let x1 = ((rx0 + rw + spread).ceil() as usize).min(w);
    let y1 = ((ry0 + rh + spread + y_off).ceil() as usize).min(h);
    for py in y0..y1 {
        let ly = py as f32 + 0.5 - (ry0 + y_off);
        for px in x0..x1 {
            let lx = px as f32 + 0.5 - rx0;
            let d = rounded_rect_sdf(lx, ly, rw, rh, radius);
            if d > 0.0 && d < spread {
                let falloff = (1.0 - d / spread).powi(2) * max_alpha;
                blend_pixel(buf, (py * w + px) * 4, [15, 20, 30, 255], falloff);
            }
        }
    }
}

fn draw_privacy_box(
    buf: &mut [u8],
    dims: (usize, usize),
    rect: (f32, f32, f32, f32),
    norm: (f32, f32, f32, f32),
) {
    let (w, h) = dims;
    let (rx0, ry0, rw, rh) = rect;
    let (ax, ay, aw, ah) = norm;
    let x0 = ((rx0 + rw * ax).max(0.0) as usize).min(w);
    let y0 = ((ry0 + rh * ay).max(0.0) as usize).min(h);
    let x1 = ((rx0 + rw * (ax + aw)).min(w as f32) as usize).min(w);
    let y1 = ((ry0 + rh * (ay + ah)).min(h as f32) as usize).min(h);
    for py in y0..y1 {
        for px in x0..x1 {
            blend_pixel(buf, (py * w + px) * 4, [30, 23, 15, 255], 0.88);
        }
    }
}

fn draw_cursor_dot(buf: &mut [u8], w: usize, h: usize, cx: f32, cy: f32, r: f32, clicked: bool) {
    let ring = if clicked { r * 1.45 } else { r + 2.0 };
    let x0 = ((cx - ring - 1.0).max(0.0) as usize).min(w);
    let y0 = ((cy - ring - 1.0).max(0.0) as usize).min(h);
    let x1 = ((cx + ring + 1.0).ceil() as usize).min(w);
    let y1 = ((cy + ring + 1.0).ceil() as usize).min(h);
    for py in y0..y1 {
        for px in x0..x1 {
            let d = ((px as f32 - cx).hypot(py as f32 - cy)).max(0.0);
            if d <= r - 1.5 {
                blend_pixel(buf, (py * w + px) * 4, [39, 24, 17, 255], 1.0);
            } else if d <= r + 1.2 {
                let border = if clicked { [250, 165, 96, 255] } else { [255, 255, 255, 255] };
                blend_pixel(buf, (py * w + px) * 4, border, 0.95);
            } else if clicked && d <= ring {
                blend_pixel(buf, (py * w + px) * 4, [250, 165, 96, 255], 0.35);
            }
        }
    }
}

fn sample_bgra(frame: &DecodedFrame, u: f32, v: f32) -> [u8; 4] {
    let x = ((u * frame.width.saturating_sub(1) as f32).round() as usize).min(frame.width as usize - 1);
    let y = ((v * frame.height.saturating_sub(1) as f32).round() as usize).min(frame.height as usize - 1);
    let i = (y * frame.width as usize + x) * 4;
    [frame.bgra[i], frame.bgra[i + 1], frame.bgra[i + 2], 255]
}

fn sample_demo_card(u: f32, v: f32) -> [u8; 4] {
    if v < 0.08 { [249, 247, 246, 255] }
    else if v > 0.14 && v < 0.24 && u > 0.05 && u < 0.95 { [235, 99, 37, 255] }
    else { [255, 255, 255, 255] }
}

fn rounded_rect_sdf(x: f32, y: f32, w: f32, h: f32, r: f32) -> f32 {
    let (qx, qy) = ((x - w * 0.5).abs() - (w * 0.5 - r), (y - h * 0.5).abs() - (h * 0.5 - r));
    qx.max(0.0).hypot(qy.max(0.0)) + qx.max(qy).min(0.0) - r
}

fn rounded_rect_coverage(x: f32, y: f32, w: f32, h: f32, r: f32) -> f32 {
    (0.5 - rounded_rect_sdf(x, y, w, h, r)).clamp(0.0, 1.0)
}

fn blend_pixel(buf: &mut [u8], i: usize, fg: [u8; 4], alpha: f32) {
    let (a, inv) = (alpha.clamp(0.0, 1.0), 1.0 - alpha.clamp(0.0, 1.0));
    buf[i] = (fg[0] as f32 * a + buf[i] as f32 * inv) as u8;
    buf[i + 1] = (fg[1] as f32 * a + buf[i + 1] as f32 * inv) as u8;
    buf[i + 2] = (fg[2] as f32 * a + buf[i + 2] as f32 * inv) as u8;
    buf[i + 3] = 255;
}

fn lerp_u8(a: u8, b: u8, t: f32) -> u8 {
    (a as f32 + (b as f32 - a as f32) * t).round().clamp(0.0, 255.0) as u8
}

fn parse_hex_bgra(hex: &str, fallback: [u8; 4]) -> [u8; 4] {
    let Ok(v) = u32::from_str_radix(hex.trim_start_matches('#'), 16) else { return fallback; };
    [(v & 0xff) as u8, ((v >> 8) & 0xff) as u8, ((v >> 16) & 0xff) as u8, 255]
}
