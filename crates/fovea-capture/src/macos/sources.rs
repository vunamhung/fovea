use std::sync::mpsc;
use std::time::Duration;

use block2::RcBlock;
use objc2::AnyThread;
use objc2::rc::Retained;
use objc2_core_foundation::{CGPoint, CGRect, CGSize};
use objc2_foundation::{NSArray, NSError};
use objc2_screen_capture_kit::{SCContentFilter, SCShareableContent, SCWindow};

use crate::backend::{CaptureSource, CaptureTarget};
use crate::error::{CaptureError, CaptureResult};
use crate::permissions::{PermissionEntry, PermissionKind, PermissionReport, PermissionState};

#[link(name = "ApplicationServices", kind = "framework")]
unsafe extern "C" {
    fn AXIsProcessTrusted() -> bool;
}

#[link(name = "IOKit", kind = "framework")]
unsafe extern "C" {
    fn IOHIDCheckAccess(request_type: u32) -> u32;
}

const K_IOHID_REQUEST_TYPE_LISTEN_EVENT: u32 = 1;
const K_IOHID_ACCESS_TYPE_GRANTED: u32 = 0;
const K_IOHID_ACCESS_TYPE_DENIED: u32 = 1;

pub struct SendFilter(pub Retained<SCContentFilter>);
unsafe impl Send for SendFilter {}

pub fn check_macos_permissions() -> PermissionReport {
    let screen = if objc2_core_graphics::CGPreflightScreenCaptureAccess() {
        PermissionState::Granted
    } else {
        PermissionState::Denied
    };
    let accessibility = if unsafe { AXIsProcessTrusted() } {
        PermissionState::Granted
    } else {
        PermissionState::Denied
    };
    let input_monitoring = match unsafe { IOHIDCheckAccess(K_IOHID_REQUEST_TYPE_LISTEN_EVENT) } {
        K_IOHID_ACCESS_TYPE_GRANTED => PermissionState::Granted,
        K_IOHID_ACCESS_TYPE_DENIED => PermissionState::Denied,
        _ => PermissionState::NotDetermined,
    };
    let mic = av_media_permission(unsafe { objc2_av_foundation::AVMediaTypeAudio });
    let cam = av_media_permission(unsafe { objc2_av_foundation::AVMediaTypeVideo });

    let items = [
        (PermissionKind::ScreenRecording, screen),
        (PermissionKind::Accessibility, accessibility),
        (PermissionKind::InputMonitoring, input_monitoring),
        (PermissionKind::Microphone, mic),
        (PermissionKind::Camera, cam),
    ];
    PermissionReport::new(
        items
            .into_iter()
            .map(|(kind, state)| PermissionEntry {
                kind,
                state,
                required: kind.is_required(),
                settings_url: kind.macos_settings_url().into(),
            })
            .collect(),
    )
}

fn av_media_permission(
    media_type: Option<&objc2_av_foundation::AVMediaType>,
) -> PermissionState {
    use objc2_av_foundation::{AVAuthorizationStatus, AVCaptureDevice};
    let Some(mt) = media_type else {
        return PermissionState::Unsupported;
    };
    match unsafe { AVCaptureDevice::authorizationStatusForMediaType(mt) } {
        AVAuthorizationStatus::Authorized => PermissionState::Granted,
        AVAuthorizationStatus::NotDetermined => PermissionState::NotDetermined,
        AVAuthorizationStatus::Denied | AVAuthorizationStatus::Restricted => {
            PermissionState::Denied
        }
        _ => PermissionState::Unsupported,
    }
}

pub fn list_macos_sources() -> CaptureResult<Vec<CaptureSource>> {
    let (tx, rx) = mpsc::channel::<Result<Vec<CaptureSource>, String>>();
    let block = RcBlock::new(move |content: *mut SCShareableContent, err: *mut NSError| {
        if !err.is_null() {
            let msg = unsafe { &*err }.localizedDescription().to_string();
            let _ = tx.send(Err(msg));
            return;
        }
        if content.is_null() {
            let _ = tx.send(Err("SCShareableContent returned null".into()));
            return;
        }
        let content = unsafe { &*content };
        let mut sources = Vec::new();
        let displays = unsafe { content.displays() };
        for (idx, display) in displays.iter().enumerate() {
            sources.push(CaptureSource::display(
                unsafe { display.displayID() },
                format!("Display {}", idx + 1),
                unsafe { display.width() }.max(0) as u32,
                unsafe { display.height() }.max(0) as u32,
            ));
        }
        let windows = unsafe { content.windows() };
        for win in windows.iter() {
            let frame = unsafe { win.frame() };
            if !unsafe { win.isOnScreen() } || frame.size.width < 80.0 || frame.size.height < 80.0 {
                continue;
            }
            let title = unsafe { win.title() }
                .map(|s| s.to_string())
                .unwrap_or_default();
            let app_name = unsafe { win.owningApplication() }
                .map(|a| unsafe { a.applicationName() }.to_string());
            if title.trim().is_empty() && app_name.is_none() {
                continue;
            }
            let label = if title.trim().is_empty() {
                app_name.clone().unwrap_or_else(|| "Window".into())
            } else {
                title
            };
            sources.push(CaptureSource::window(
                unsafe { win.windowID() },
                label,
                app_name,
                frame.size.width as u32,
                frame.size.height as u32,
            ));
        }
        let _ = tx.send(Ok(sources));
    });
    unsafe { SCShareableContent::getShareableContentWithCompletionHandler(&block) };
    rx.recv_timeout(Duration::from_secs(5))
        .map_err(|_| CaptureError::Backend("timed out enumerating macOS capture sources".into()))?
        .map_err(CaptureError::Backend)
}

pub fn resolve_macos_filter(
    target: CaptureTarget,
) -> CaptureResult<(SendFilter, Option<CGRect>)> {
    let (tx, rx) = mpsc::channel::<Result<(SendFilter, Option<CGRect>), CaptureError>>();
    let block = RcBlock::new(move |content: *mut SCShareableContent, err: *mut NSError| {
        if !err.is_null() {
            let msg = unsafe { &*err }.localizedDescription().to_string();
            let _ = tx.send(Err(CaptureError::Backend(msg)));
            return;
        }
        if content.is_null() {
            let _ = tx.send(Err(CaptureError::Backend(
                "SCShareableContent returned null".into(),
            )));
            return;
        }
        let content = unsafe { &*content };
        let res = build_filter(content, &target);
        let _ = tx.send(res);
    });
    unsafe { SCShareableContent::getShareableContentWithCompletionHandler(&block) };
    rx.recv_timeout(Duration::from_secs(5))
        .map_err(|_| CaptureError::Backend("timed out resolving capture filter".into()))?
}

fn build_filter(
    content: &SCShareableContent,
    target: &CaptureTarget,
) -> CaptureResult<(SendFilter, Option<CGRect>)> {
    let empty_windows: Retained<NSArray<SCWindow>> = NSArray::new();
    match *target {
        CaptureTarget::Display { display_id } => {
            let displays = unsafe { content.displays() };
            let display = displays
                .iter()
                .find(|d| unsafe { d.displayID() } == display_id)
                .or_else(|| displays.iter().next())
                .ok_or_else(|| CaptureError::SourceNotFound(format!("display {display_id}")))?;
            let filter = unsafe {
                SCContentFilter::initWithDisplay_excludingWindows(
                    SCContentFilter::alloc(),
                    &display,
                    &empty_windows,
                )
            };
            Ok((SendFilter(filter), None))
        }
        CaptureTarget::Window { window_id } => {
            let windows = unsafe { content.windows() };
            let win = windows
                .iter()
                .find(|w| unsafe { w.windowID() } == window_id)
                .ok_or_else(|| CaptureError::SourceNotFound(format!("window {window_id}")))?;
            let filter = unsafe {
                SCContentFilter::initWithDesktopIndependentWindow(SCContentFilter::alloc(), &win)
            };
            Ok((SendFilter(filter), None))
        }
        CaptureTarget::Region {
            display_id,
            x,
            y,
            width,
            height,
        } => {
            let displays = unsafe { content.displays() };
            let display = displays
                .iter()
                .find(|d| unsafe { d.displayID() } == display_id)
                .or_else(|| displays.iter().next())
                .ok_or_else(|| CaptureError::SourceNotFound(format!("display {display_id}")))?;
            let filter = unsafe {
                SCContentFilter::initWithDisplay_excludingWindows(
                    SCContentFilter::alloc(),
                    &display,
                    &empty_windows,
                )
            };
            let rect = CGRect::new(
                CGPoint::new(x as f64, y as f64),
                CGSize::new(width as f64, height as f64),
            );
            Ok((SendFilter(filter), Some(rect)))
        }
    }
}
