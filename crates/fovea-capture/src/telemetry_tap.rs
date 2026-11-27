use std::fs::OpenOptions;
use std::io::{BufWriter, Write};
use std::path::PathBuf;

use fovea_core::{FoveaBundle, TelemetryEvent};

use crate::error::CaptureResult;

#[derive(Debug, Clone)]
pub struct TelemetryTapConfig {
    pub capture_keystrokes: bool,
    pub shortcuts_only: bool,
    pub capture_ax_context: bool,
    pub bounds_x: f64,
    pub bounds_y: f64,
    pub bounds_w: f64,
    pub bounds_h: f64,
}

impl Default for TelemetryTapConfig {
    fn default() -> Self {
        Self {
            capture_keystrokes: true,
            shortcuts_only: true,
            capture_ax_context: true,
            bounds_x: 0.0,
            bounds_y: 0.0,
            bounds_w: 1920.0,
            bounds_h: 1080.0,
        }
    }
}

pub struct TelemetrySink {
    path: PathBuf,
    pending: Vec<TelemetryEvent>,
    flush_threshold: usize,
}

impl TelemetrySink {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            pending: Vec::with_capacity(64),
            flush_threshold: 64,
        }
    }

    pub fn push(&mut self, event: TelemetryEvent) {
        self.pending.push(event);
        if self.pending.len() >= self.flush_threshold {
            let _ = self.flush();
        }
    }

    pub fn flush(&mut self) -> CaptureResult<()> {
        if self.pending.is_empty() {
            return Ok(());
        }
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;
        let mut writer = BufWriter::new(file);
        for ev in self.pending.drain(..) {
            let line = serde_json::to_string(&ev)
                .map_err(|e| crate::error::CaptureError::Backend(e.to_string()))?;
            writeln!(writer, "{line}")?;
        }
        writer.flush()?;
        Ok(())
    }
}

impl Drop for TelemetrySink {
    fn drop(&mut self) {
        let _ = self.flush();
    }
}

pub struct TelemetryTapHandle {
    #[cfg(target_os = "macos")]
    inner: Option<crate::macos::event_tap::MacOsEventTap>,
}

impl TelemetryTapHandle {
    pub fn start(config: TelemetryTapConfig, bundle: &FoveaBundle) -> CaptureResult<Self> {
        #[cfg(target_os = "macos")]
        {
            let tap = crate::macos::event_tap::MacOsEventTap::start(
                config,
                bundle.telemetry_path(),
            )?;
            Ok(Self { inner: Some(tap) })
        }
        #[cfg(not(target_os = "macos"))]
        {
            let _ = (config, bundle);
            Ok(Self {})
        }
    }

    pub fn stop(&mut self) -> CaptureResult<()> {
        #[cfg(target_os = "macos")]
        if let Some(tap) = self.inner.take() {
            tap.stop()?;
        }
        Ok(())
    }
}
