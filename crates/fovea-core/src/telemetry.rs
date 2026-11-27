use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AxClickContext {
    pub app_name: String,
    pub window_title: Option<String>,
    pub role: Option<String>,
    pub label: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TelemetryEvent {
    CursorMove {
        timestamp_ms: u64,
        x: f32,
        y: f32,
    },
    CursorClick {
        timestamp_ms: u64,
        x: f32,
        y: f32,
        button: MouseButton,
        ax_context: Option<AxClickContext>,
    },
    CursorShape {
        timestamp_ms: u64,
        shape: String,
    },
    Keystroke {
        timestamp_ms: u64,
        display: String,
    },
}

impl TelemetryEvent {
    pub fn cursor_move(timestamp_ms: u64, x: f32, y: f32) -> Self {
        Self::CursorMove { timestamp_ms, x, y }
    }

    pub fn cursor_click(
        timestamp_ms: u64,
        x: f32,
        y: f32,
        button: MouseButton,
        ax_context: Option<AxClickContext>,
    ) -> Self {
        Self::CursorClick {
            timestamp_ms,
            x,
            y,
            button,
            ax_context,
        }
    }

    pub fn keystroke(timestamp_ms: u64, display: impl Into<String>) -> Self {
        Self::Keystroke {
            timestamp_ms,
            display: display.into(),
        }
    }

    pub fn timestamp_ms(&self) -> u64 {
        match self {
            Self::CursorMove { timestamp_ms, .. }
            | Self::CursorClick { timestamp_ms, .. }
            | Self::CursorShape { timestamp_ms, .. }
            | Self::Keystroke { timestamp_ms, .. } => *timestamp_ms,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TelemetryReadReport {
    pub events: Vec<TelemetryEvent>,
    pub skipped_lines: usize,
}
