use std::ffi::{CStr, c_char, c_void};
use std::ptr;

use fovea_core::AxClickContext;

type CFTypeRef = *const c_void;
type CFStringRef = *const c_void;
type AXUIElementRef = *const c_void;

const K_CF_STRING_ENCODING_UTF8: u32 = 0x0800_0100;

#[link(name = "ApplicationServices", kind = "framework")]
unsafe extern "C" {
    fn AXIsProcessTrusted() -> bool;
    fn AXUIElementCreateSystemWide() -> AXUIElementRef;
    fn AXUIElementCopyElementAtPosition(
        application: AXUIElementRef,
        x: f32,
        y: f32,
        element: *mut AXUIElementRef,
    ) -> i32;
    fn AXUIElementCopyAttributeValue(
        element: AXUIElementRef,
        attribute: CFStringRef,
        value: *mut CFTypeRef,
    ) -> i32;
}

#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {
    fn CFRelease(cf: CFTypeRef);
    fn CFGetTypeID(cf: CFTypeRef) -> usize;
    fn CFStringGetTypeID() -> usize;
    fn CFStringCreateWithCString(
        alloc: CFTypeRef,
        c_str: *const c_char,
        encoding: u32,
    ) -> CFStringRef;
    fn CFStringGetCString(
        the_string: CFStringRef,
        buffer: *mut c_char,
        buffer_size: isize,
        encoding: u32,
    ) -> bool;
}

pub fn query_ax_click_context(screen_x: f32, screen_y: f32) -> Option<AxClickContext> {
    if !unsafe { AXIsProcessTrusted() } {
        return None;
    }
    let system_wide = unsafe { AXUIElementCreateSystemWide() };
    if system_wide.is_null() {
        return None;
    }
    let mut element: AXUIElementRef = ptr::null();
    let status = unsafe {
        AXUIElementCopyElementAtPosition(system_wide, screen_x, screen_y, &mut element)
    };
    unsafe { CFRelease(system_wide) };
    if status != 0 || element.is_null() {
        return None;
    }

    let role = copy_ax_string(element, c"AXRole");
    let label = copy_ax_string(element, c"AXTitle")
        .or_else(|| copy_ax_string(element, c"AXDescription"))
        .or_else(|| copy_ax_string(element, c"AXHelp"));
    let window_title = copy_ax_string(element, c"AXWindow");

    unsafe { CFRelease(element) };
    if role.is_none() && label.is_none() {
        return None;
    }

    Some(AxClickContext {
        app_name: String::from("macOS App"),
        window_title,
        role,
        label,
    })
}

fn copy_ax_string(element: AXUIElementRef, attr: &CStr) -> Option<String> {
    let attr_ref =
        unsafe { CFStringCreateWithCString(ptr::null(), attr.as_ptr(), K_CF_STRING_ENCODING_UTF8) };
    if attr_ref.is_null() {
        return None;
    }
    let mut value: CFTypeRef = ptr::null();
    let err = unsafe { AXUIElementCopyAttributeValue(element, attr_ref, &mut value) };
    unsafe { CFRelease(attr_ref) };
    if err != 0 || value.is_null() {
        return None;
    }
    let out = if unsafe { CFGetTypeID(value) == CFStringGetTypeID() } {
        let mut buf = [0_i8; 256];
        let ok = unsafe {
            CFStringGetCString(
                value,
                buf.as_mut_ptr(),
                buf.len() as isize,
                K_CF_STRING_ENCODING_UTF8,
            )
        };
        if ok {
            let s = unsafe { CStr::from_ptr(buf.as_ptr()) }
                .to_string_lossy()
                .trim()
                .to_string();
            if s.is_empty() { None } else { Some(s) }
        } else {
            None
        }
    } else {
        None
    };
    unsafe { CFRelease(value) };
    out
}
