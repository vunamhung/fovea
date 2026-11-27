use serde::{Deserialize, Serialize};

use crate::error::{CaptureError, CaptureResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PermissionKind {
    ScreenRecording,
    Accessibility,
    InputMonitoring,
    Microphone,
    Camera,
}

impl PermissionKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::ScreenRecording => "Screen Recording",
            Self::Accessibility => "Accessibility",
            Self::InputMonitoring => "Input Monitoring",
            Self::Microphone => "Microphone",
            Self::Camera => "Camera",
        }
    }

    pub fn is_required(self) -> bool {
        matches!(self, Self::ScreenRecording)
    }

    pub fn macos_settings_url(self) -> &'static str {
        match self {
            Self::ScreenRecording => {
                "x-apple.systempreferences:com.apple.preference.security?Privacy_ScreenCapture"
            }
            Self::Accessibility => {
                "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility"
            }
            Self::InputMonitoring => {
                "x-apple.systempreferences:com.apple.preference.security?Privacy_ListenEvent"
            }
            Self::Microphone => {
                "x-apple.systempreferences:com.apple.preference.security?Privacy_Microphone"
            }
            Self::Camera => {
                "x-apple.systempreferences:com.apple.preference.security?Privacy_Camera"
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PermissionState {
    Granted,
    Denied,
    NotDetermined,
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PermissionEntry {
    pub kind: PermissionKind,
    pub state: PermissionState,
    pub required: bool,
    pub settings_url: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PermissionReport {
    pub entries: Vec<PermissionEntry>,
}

impl PermissionReport {
    pub fn new(entries: Vec<PermissionEntry>) -> Self {
        Self { entries }
    }

    pub fn all_granted_for_test() -> Self {
        let all = [
            PermissionKind::ScreenRecording,
            PermissionKind::Accessibility,
            PermissionKind::InputMonitoring,
            PermissionKind::Microphone,
            PermissionKind::Camera,
        ];
        Self::new(
            all.into_iter()
                .map(|kind| PermissionEntry {
                    kind,
                    state: PermissionState::Granted,
                    required: kind.is_required(),
                    settings_url: kind.macos_settings_url().into(),
                })
                .collect(),
        )
    }

    pub fn get(&self, kind: PermissionKind) -> PermissionState {
        self.entries
            .iter()
            .find(|e| e.kind == kind)
            .map(|e| e.state)
            .unwrap_or(PermissionState::Unsupported)
    }

    pub fn set(&mut self, kind: PermissionKind, state: PermissionState) {
        if let Some(entry) = self.entries.iter_mut().find(|e| e.kind == kind) {
            entry.state = state;
        }
    }

    pub fn all_required_granted(&self) -> bool {
        self.entries
            .iter()
            .filter(|e| e.required)
            .all(|e| e.state == PermissionState::Granted)
    }
}

pub fn open_system_settings_url(url: &str) -> CaptureResult<()> {
    #[cfg(target_os = "macos")]
    {
        let status = std::process::Command::new("open").arg(url).status()?;
        if !status.success() {
            return Err(CaptureError::Backend(format!(
                "failed to open System Settings ({url})"
            )));
        }
        Ok(())
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = url;
        Err(CaptureError::Backend(
            "opening System Settings is only implemented on macOS".into(),
        ))
    }
}
