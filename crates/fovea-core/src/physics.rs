use std::f32::consts::PI;

use crate::schema::ProjectDoc;
use crate::style::CursorStyle;
use crate::telemetry::TelemetryEvent;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpringState {
    pub value: f32,
    pub velocity: f32,
}

impl SpringState {
    pub fn new(initial: f32) -> Self {
        Self {
            value: initial,
            velocity: 0.0,
        }
    }

    pub fn step(&mut self, target: f32, stiffness: f32, damping: f32, dt_sec: f32) -> f32 {
        let dt = dt_sec.clamp(0.0, 0.05);
        let force = -stiffness * (self.value - target) - damping * self.velocity;
        self.velocity += force * dt;
        self.value += self.velocity * dt;
        self.value
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CameraPose {
    pub scale: f32,
    pub focus_x: f32,
    pub focus_y: f32,
    pub velocity_x: f32,
    pub velocity_y: f32,
}

impl Default for CameraPose {
    fn default() -> Self {
        Self {
            scale: 1.0,
            focus_x: 0.5,
            focus_y: 0.5,
            velocity_x: 0.0,
            velocity_y: 0.0,
        }
    }
}

pub fn evaluate_camera_pose(project: &ProjectDoc, output_ms: u64) -> CameraPose {
    if project.zoom_regions.is_empty() {
        return CameraPose::default();
    }
    const STEP_MS: u64 = 16;
    const DT: f32 = 1.0 / 60.0;
    const STIFFNESS: f32 = 170.0;
    const DAMPING: f32 = 26.0;

    let mut scale = SpringState::new(1.0);
    let mut fx = SpringState::new(0.5);
    let mut fy = SpringState::new(0.5);

    let mut t: u64 = 0;
    while t <= output_ms {
        let (target_scale, target_x, target_y) = project
            .zoom_regions
            .iter()
            .rfind(|z| t >= z.start_ms && t <= z.end_ms)
            .map(|z| (z.scale.max(1.0), z.focus_x.clamp(0.0, 1.0), z.focus_y.clamp(0.0, 1.0)))
            .unwrap_or((1.0, 0.5, 0.5));

        scale.step(target_scale, STIFFNESS, DAMPING, DT);
        fx.step(target_x, STIFFNESS, DAMPING, DT);
        fy.step(target_y, STIFFNESS, DAMPING, DT);
        t += STEP_MS;
    }

    CameraPose {
        scale: scale.value.max(1.0),
        focus_x: fx.value.clamp(0.0, 1.0),
        focus_y: fy.value.clamp(0.0, 1.0),
        velocity_x: fx.velocity,
        velocity_y: fy.velocity,
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CursorPose {
    pub x: f32,
    pub y: f32,
    pub scale: f32,
    pub opacity: f32,
    pub vx: f32,
    pub vy: f32,
    pub sway_rad: f32,
}

pub fn evaluate_cursor_pose(
    events: &[TelemetryEvent],
    style: &CursorStyle,
    source_ms: u64,
) -> Option<CursorPose> {
    if !style.visible {
        return None;
    }
    let alpha = (1.0 - style.smoothing.clamp(0.0, 0.95)).max(0.08);
    let mut cur_x: Option<f32> = None;
    let mut cur_y: Option<f32> = None;
    let mut vx = 0.0_f32;
    let mut vy = 0.0_f32;
    let mut last_move_ms: u64 = 0;
    let mut last_click_ms: Option<u64> = None;

    for ev in events {
        let ts = ev.timestamp_ms();
        if ts > source_ms {
            break;
        }
        match *ev {
            TelemetryEvent::CursorMove { x, y, .. } | TelemetryEvent::CursorClick { x, y, .. } => {
                if let (Some(px), Some(py)) = (cur_x, cur_y) {
                    let dt = (ts.saturating_sub(last_move_ms).max(1) as f32) / 1000.0;
                    let nx = px + (x - px) * alpha;
                    let ny = py + (y - py) * alpha;
                    vx = (nx - px) / dt;
                    vy = (ny - py) / dt;
                    if (x - px).hypot(y - py) > 0.002 {
                        last_move_ms = ts;
                    }
                    cur_x = Some(nx);
                    cur_y = Some(ny);
                } else {
                    cur_x = Some(x);
                    cur_y = Some(y);
                    last_move_ms = ts;
                }
                if matches!(ev, TelemetryEvent::CursorClick { .. }) {
                    last_click_ms = Some(ts);
                    last_move_ms = ts;
                }
            }
            _ => {}
        }
    }

    let (x, y) = (cur_x?, cur_y?);
    let mut scale = style.scale;
    if style.click_bounce
        && let Some(click_ms) = last_click_ms
    {
        let elapsed = source_ms.saturating_sub(click_ms);
        if elapsed <= 180 {
            let phase = (elapsed as f32 / 180.0) * PI;
            scale *= 1.0 - 0.22 * phase.sin();
        }
    }

    let opacity = match style.idle_hide_ms {
        Some(threshold) if source_ms.saturating_sub(last_move_ms) > threshold => {
            let over = source_ms.saturating_sub(last_move_ms) - threshold;
            (1.0 - (over as f32 / 300.0)).clamp(0.0, 1.0)
        }
        _ => 1.0,
    };

    let sway_rad = (vx * 0.12 * style.sway).clamp(-0.35, 0.35);
    Some(CursorPose {
        x,
        y,
        scale,
        opacity,
        vx,
        vy,
        sway_rad,
    })
}
