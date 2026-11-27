use std::ffi::c_void;
use std::ptr;

use block2::RcBlock;
use objc2::runtime::Bool;
use objc2_foundation::NSError;
use objc2_screen_capture_kit::SCShareableContent;

use crate::permissions::{PermissionEntry, PermissionKind, PermissionReport, PermissionState};

type CFTypeRef = *const c_void;
type CFDictionaryRef = *const c_void;

#[link(name = "ApplicationServices", kind = "framework")]
unsafe extern "C" {
    fn AXIsProcessTrusted() -> bool;
    fn AXIsProcessTrustedWithOptions(options: CFDictionaryRef) -> bool;
    static kAXTrustedCheckOptionPrompt: CFTypeRef;
}

#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {
    static kCFBooleanTrue: CFTypeRef;
    fn CFDictionaryCreate(
        allocator: CFTypeRef,
        keys: *const CFTypeRef,
        values: *const CFTypeRef,
        num_values: isize,
        key_callbacks: *const c_void,
        value_callbacks: *const c_void,
    ) -> CFDictionaryRef;
    fn CFRelease(cf: CFTypeRef);
}

#[link(name = "IOKit", kind = "framework")]
unsafe extern "C" {
    fn IOHIDCheckAccess(request_type: u32) -> u32;
    fn IOHIDRequestAccess(request_type: u32) -> bool;
}

const K_IOHID_REQUEST_TYPE_LISTEN_EVENT: u32 = 1;
const K_IOHID_ACCESS_TYPE_GRANTED: u32 = 0;
const K_IOHID_ACCESS_TYPE_DENIED: u32 = 1;

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

/// Triggers the native macOS TCC registration / prompt API for `kind` so macOS
/// adds Fovea to System Settings Privacy lists with an ON/OFF toggle.
pub fn request_macos_permission(kind: PermissionKind) -> bool {
    match kind {
        PermissionKind::ScreenRecording => {
            let granted = objc2_core_graphics::CGRequestScreenCaptureAccess();
            if !granted {
                // ScreenCaptureKit probe ensures macOS 14+ registers the app bundle in
                // System Settings -> Screen & System Audio Recording with an ON/OFF toggle.
                let block = RcBlock::new(|_content: *mut SCShareableContent, _err: *mut NSError| {});
                unsafe { SCShareableContent::getShareableContentWithCompletionHandler(&block) };
            }
            granted
        }
        PermissionKind::Accessibility => request_accessibility_prompt(),
        PermissionKind::InputMonitoring => unsafe {
            IOHIDRequestAccess(K_IOHID_REQUEST_TYPE_LISTEN_EVENT)
        },
        PermissionKind::Microphone => {
            request_av_media_access(unsafe { objc2_av_foundation::AVMediaTypeAudio })
        }
        PermissionKind::Camera => {
            request_av_media_access(unsafe { objc2_av_foundation::AVMediaTypeVideo })
        }
    }
}

fn request_accessibility_prompt() -> bool {
    unsafe {
        let keys = [kAXTrustedCheckOptionPrompt];
        let values = [kCFBooleanTrue];
        let opts = CFDictionaryCreate(
            ptr::null(),
            keys.as_ptr(),
            values.as_ptr(),
            1,
            ptr::null(),
            ptr::null(),
        );
        if opts.is_null() {
            return AXIsProcessTrusted();
        }
        let trusted = AXIsProcessTrustedWithOptions(opts);
        CFRelease(opts);
        trusted
    }
}

fn request_av_media_access(media_type: Option<&objc2_av_foundation::AVMediaType>) -> bool {
    use objc2_av_foundation::AVCaptureDevice;
    let Some(mt) = media_type else {
        return false;
    };
    if av_media_permission(Some(mt)) == PermissionState::Granted {
        return true;
    }
    let block = RcBlock::new(|_granted: Bool| {});
    unsafe {
        AVCaptureDevice::requestAccessForMediaType_completionHandler(mt, &block);
    }
    false
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
