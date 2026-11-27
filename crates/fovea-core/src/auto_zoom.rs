use uuid::Uuid;

use crate::schema::ZoomRegion;
use crate::telemetry::TelemetryEvent;

/// Clusters recorded mouse clicks into smooth auto-zoom regions, skipping HUD clicks
/// near the very bottom edge (`y > 0.92`) and final stop clicks.
pub fn generate_auto_zooms(events: &[TelemetryEvent], duration_ms: u64) -> Vec<ZoomRegion> {
    if duration_ms < 600 {
        return Vec::new();
    }
    let clicks: Vec<(u64, f32, f32)> = events
        .iter()
        .filter_map(|ev| match *ev {
            TelemetryEvent::CursorClick {
                timestamp_ms, x, y, ..
            } if y <= 0.92 && timestamp_ms + 350 < duration_ms => {
                Some((timestamp_ms, x.clamp(0.08, 0.92), y.clamp(0.08, 0.92)))
            }
            _ => None,
        })
        .collect();

    let mut regions: Vec<ZoomRegion> = Vec::new();
    for (ts, x, y) in clicks {
        if let Some(last) = regions.last_mut()
            && ts <= last.end_ms + 400
        {
            last.end_ms = (ts + 1_600).min(duration_ms);
            last.focus_x = (last.focus_x + x) * 0.5;
            last.focus_y = (last.focus_y + y) * 0.5;
            continue;
        }
        let start_ms = ts.saturating_sub(250);
        let end_ms = (ts + 1_600).min(duration_ms);
        if end_ms > start_ms + 250 {
            regions.push(ZoomRegion {
                id: Uuid::new_v4(),
                start_ms,
                end_ms,
                scale: 1.6,
                focus_x: x,
                focus_y: y,
                auto_generated: true,
            });
        }
    }
    regions
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
        let ease = smoothstep((elapsed / ramp).min(remain / ramp));
        let scale = 1.0 + (z.scale - 1.0) * ease;
        let fx = 0.5 + (z.focus_x - 0.5) * ease;
        let fy = 0.5 + (z.focus_y - 0.5) * ease;
        return (scale, fx, fy);
    }
    (1.0, 0.5, 0.5)
}

/// Interpolates recorded cursor `(x, y, clicked_recently)` at `ms` from telemetry.
pub fn sample_cursor_at_ms(events: &[TelemetryEvent], ms: u64) -> Option<(f32, f32, bool)> {
    let mut prev: Option<(u64, f32, f32)> = None;
    let mut next: Option<(u64, f32, f32)> = None;
    let mut clicked = false;

    for ev in events {
        match *ev {
            TelemetryEvent::CursorMove { timestamp_ms, x, y } => {
                if timestamp_ms <= ms {
                    prev = Some((timestamp_ms, x, y));
                } else if next.is_none() {
                    next = Some((timestamp_ms, x, y));
                }
            }
            TelemetryEvent::CursorClick { timestamp_ms, x, y, .. } => {
                if ms >= timestamp_ms && ms.saturating_sub(timestamp_ms) <= 220 {
                    clicked = true;
                }
                if timestamp_ms <= ms {
                    prev = Some((timestamp_ms, x, y));
                } else if next.is_none() {
                    next = Some((timestamp_ms, x, y));
                }
            }
            TelemetryEvent::Keystroke { .. } | TelemetryEvent::CursorShape { .. } => {}
        }
    }

    match (prev, next) {
        (Some((t0, x0, y0)), Some((t1, x1, y1))) if t1 > t0 => {
            let alpha = ((ms - t0) as f32 / (t1 - t0) as f32).clamp(0.0, 1.0);
            Some((x0 + (x1 - x0) * alpha, y0 + (y1 - y0) * alpha, clicked))
        }
        (Some((_, x, y)), _) | (_, Some((_, x, y))) => Some((x, y, clicked)),
        (None, None) => None,
    }
}

/// Returns the most recent keystroke label still visible within `duration_ms` of `ms`.
pub fn active_keystroke_at_ms(
    events: &[TelemetryEvent],
    ms: u64,
    duration_ms: u64,
) -> Option<&str> {
    events.iter().rev().find_map(|ev| match ev {
        TelemetryEvent::Keystroke {
            timestamp_ms,
            display,
        } if ms >= *timestamp_ms && ms.saturating_sub(*timestamp_ms) <= duration_ms => {
            Some(display.as_str())
        }
        _ => None,
    })
}

fn smoothstep(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}
