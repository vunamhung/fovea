use std::ffi::c_void;
use std::path::PathBuf;
use std::ptr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Instant;

use fovea_core::{MouseButton, TelemetryEvent};
use objc2_core_foundation::CGPoint;

use crate::error::CaptureResult;
use crate::keystroke_filter::{KeyModifiers, RawKeyEvent, filter_keystroke, macos_keycode_label};
use crate::macos::ax_context::query_ax_click_context;
use crate::telemetry_tap::{TelemetrySink, TelemetryTapConfig};

type CFMachPortRef = *const c_void;
type CFRunLoopSourceRef = *const c_void;
type CFRunLoopRef = *const c_void;
type CGEventRef = *const c_void;
type CGEventTapProxy = *const c_void;
type CGEventCallback = unsafe extern "C" fn(
    proxy: CGEventTapProxy,
    event_type: u32,
    event: CGEventRef,
    user_info: *mut c_void,
) -> CGEventRef;

const K_CG_SESSION_EVENT_TAP: u32 = 1;
const K_CG_HEAD_INSERT_EVENT_TAP: u32 = 0;
const K_CG_EVENT_TAP_OPTION_LISTEN_ONLY: u32 = 1;

const EV_LEFT_MOUSE_DOWN: u32 = 1;
const EV_RIGHT_MOUSE_DOWN: u32 = 3;
const EV_MOUSE_MOVED: u32 = 5;
const EV_LEFT_MOUSE_DRAGGED: u32 = 6;
const EV_KEY_DOWN: u32 = 10;
const EV_OTHER_MOUSE_DOWN: u32 = 25;

const K_CG_KEYBOARD_EVENT_KEYCODE: u32 = 9;
const FLAG_SHIFT: u64 = 1 << 17;
const FLAG_CONTROL: u64 = 1 << 18;
const FLAG_OPTION: u64 = 1 << 19;
const FLAG_COMMAND: u64 = 1 << 20;

#[link(name = "Carbon", kind = "framework")]
unsafe extern "C" {
    fn IsSecureEventInputEnabled() -> bool;
}

#[link(name = "CoreGraphics", kind = "framework")]
unsafe extern "C" {
    fn CGEventTapCreate(
        tap: u32,
        place: u32,
        options: u32,
        events_of_interest: u64,
        callback: CGEventCallback,
        user_info: *mut c_void,
    ) -> CFMachPortRef;
    fn CGEventTapEnable(tap: CFMachPortRef, enable: bool);
    fn CGEventGetLocation(event: CGEventRef) -> CGPoint;
    fn CGEventGetFlags(event: CGEventRef) -> u64;
    fn CGEventGetIntegerValueField(event: CGEventRef, field: u32) -> i64;
}

#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {
    static kCFRunLoopCommonModes: *const c_void;
    fn CFMachPortCreateRunLoopSource(
        allocator: *const c_void,
        port: CFMachPortRef,
        order: isize,
    ) -> CFRunLoopSourceRef;
    fn CFRunLoopGetCurrent() -> CFRunLoopRef;
    fn CFRunLoopAddSource(rl: CFRunLoopRef, source: CFRunLoopSourceRef, mode: *const c_void);
    fn CFRunLoopRun();
    fn CFRunLoopStop(rl: CFRunLoopRef);
    fn CFRelease(cf: *const c_void);
}

struct TapContext {
    config: TelemetryTapConfig,
    started_at: Instant,
    sink: TelemetrySink,
    stopped: Arc<AtomicBool>,
}

struct SendRunLoop(CFRunLoopRef);
unsafe impl Send for SendRunLoop {}

pub struct MacOsEventTap {
    stopped: Arc<AtomicBool>,
    run_loop: Arc<Mutex<Option<SendRunLoop>>>,
    thread: Option<JoinHandle<()>>,
}

impl MacOsEventTap {
    pub fn start(config: TelemetryTapConfig, telemetry_path: PathBuf) -> CaptureResult<Self> {
        let stopped = Arc::new(AtomicBool::new(false));
        let run_loop = Arc::new(Mutex::new(None));
        let thread_stopped = Arc::clone(&stopped);
        let thread_rl = Arc::clone(&run_loop);

        let handle = thread::Builder::new()
            .name("fovea-telemetry-tap".into())
            .spawn(move || {
                run_tap_thread(config, telemetry_path, thread_stopped, thread_rl);
            })?;

        Ok(Self {
            stopped,
            run_loop,
            thread: Some(handle),
        })
    }

    pub fn stop(mut self) -> CaptureResult<()> {
        self.stopped.store(true, Ordering::SeqCst);
        if let Ok(mut guard) = self.run_loop.lock()
            && let Some(rl) = guard.take()
        {
            unsafe { CFRunLoopStop(rl.0) };
        }
        if let Some(handle) = self.thread.take() {
            let _ = handle.join();
        }
        Ok(())
    }
}

fn run_tap_thread(
    config: TelemetryTapConfig,
    telemetry_path: PathBuf,
    stopped: Arc<AtomicBool>,
    run_loop_slot: Arc<Mutex<Option<SendRunLoop>>>,
) {
    let mut mask: u64 = (1 << EV_MOUSE_MOVED)
        | (1 << EV_LEFT_MOUSE_DRAGGED)
        | (1 << EV_LEFT_MOUSE_DOWN)
        | (1 << EV_RIGHT_MOUSE_DOWN)
        | (1 << EV_OTHER_MOUSE_DOWN);
    if config.capture_keystrokes {
        mask |= 1 << EV_KEY_DOWN;
    }

    let ctx = Box::new(TapContext {
        config,
        started_at: Instant::now(),
        sink: TelemetrySink::new(telemetry_path),
        stopped: Arc::clone(&stopped),
    });
    let ctx_ptr = Box::into_raw(ctx);

    let tap = unsafe {
        CGEventTapCreate(
            K_CG_SESSION_EVENT_TAP,
            K_CG_HEAD_INSERT_EVENT_TAP,
            K_CG_EVENT_TAP_OPTION_LISTEN_ONLY,
            mask,
            tap_callback,
            ctx_ptr.cast(),
        )
    };
    if tap.is_null() {
        tracing::warn!("CGEventTapCreate returned null (Accessibility/Input Monitoring missing)");
        unsafe { drop(Box::from_raw(ctx_ptr)) };
        return;
    }

    let source = unsafe { CFMachPortCreateRunLoopSource(ptr::null(), tap, 0) };
    let rl = unsafe { CFRunLoopGetCurrent() };
    if let Ok(mut slot) = run_loop_slot.lock() {
        *slot = Some(SendRunLoop(rl));
    }
    unsafe {
        CFRunLoopAddSource(rl, source, kCFRunLoopCommonModes);
        CGEventTapEnable(tap, true);
        if !stopped.load(Ordering::SeqCst) {
            CFRunLoopRun();
        }
        CFRelease(source);
        CFRelease(tap);
        drop(Box::from_raw(ctx_ptr));
    }
}

unsafe extern "C" fn tap_callback(
    _proxy: CGEventTapProxy,
    event_type: u32,
    event: CGEventRef,
    user_info: *mut c_void,
) -> CGEventRef {
    if user_info.is_null() || event.is_null() {
        return event;
    }
    let ctx = unsafe { &mut *user_info.cast::<TapContext>() };
    if ctx.stopped.load(Ordering::Relaxed) {
        return event;
    }
    let ts_ms = ctx.started_at.elapsed().as_millis() as u64;
    let loc = unsafe { CGEventGetLocation(event) };
    let nx = ((loc.x - ctx.config.bounds_x) / ctx.config.bounds_w.max(1.0)).clamp(0.0, 1.0) as f32;
    let ny = ((loc.y - ctx.config.bounds_y) / ctx.config.bounds_h.max(1.0)).clamp(0.0, 1.0) as f32;

    match event_type {
        EV_MOUSE_MOVED | EV_LEFT_MOUSE_DRAGGED => {
            ctx.sink.push(TelemetryEvent::cursor_move(ts_ms, nx, ny));
        }
        EV_LEFT_MOUSE_DOWN | EV_RIGHT_MOUSE_DOWN | EV_OTHER_MOUSE_DOWN => {
            let button = match event_type {
                EV_LEFT_MOUSE_DOWN => MouseButton::Left,
                EV_RIGHT_MOUSE_DOWN => MouseButton::Right,
                _ => MouseButton::Middle,
            };
            let ax = if ctx.config.capture_ax_context {
                query_ax_click_context(loc.x as f32, loc.y as f32)
            } else {
                None
            };
            ctx.sink
                .push(TelemetryEvent::cursor_click(ts_ms, nx, ny, button, ax));
        }
        EV_KEY_DOWN => {
            let secure = unsafe { IsSecureEventInputEnabled() };
            let keycode = unsafe { CGEventGetIntegerValueField(event, K_CG_KEYBOARD_EVENT_KEYCODE) }
                as u16;
            if let Some(label) = macos_keycode_label(keycode) {
                let flags = unsafe { CGEventGetFlags(event) };
                let raw = RawKeyEvent {
                    key_label: label.into(),
                    modifiers: KeyModifiers {
                        control: (flags & FLAG_CONTROL) != 0,
                        option: (flags & FLAG_OPTION) != 0,
                        cmd: (flags & FLAG_COMMAND) != 0,
                        shift: (flags & FLAG_SHIFT) != 0,
                    },
                };
                if let Some(display) =
                    filter_keystroke(&raw, secure, ctx.config.shortcuts_only)
                {
                    ctx.sink.push(TelemetryEvent::keystroke(ts_ms, display));
                }
            }
        }
        _ => {}
    }
    event
}
