use std::path::Path;

use crate::error::{RenderError, RenderResult};

/// A single decoded video frame in 8-bit premultiplied BGRA layout (native macOS / GPUI order).
#[derive(Debug, Clone)]
pub struct DecodedFrame {
    pub width: u32,
    pub height: u32,
    pub bgra: Vec<u8>,
}

impl DecodedFrame {
    pub fn byte_len_valid(&self) -> bool {
        self.bgra.len() == (self.width as usize) * (self.height as usize) * 4
    }
}

pub fn extract_video_frame(
    video_path: &Path,
    timestamp_ms: u64,
    max_dim: u32,
) -> RenderResult<DecodedFrame> {
    if !video_path.exists() {
        return Err(RenderError::NotFound(video_path.display().to_string()));
    }
    #[cfg(target_os = "macos")]
    {
        macos_decode::extract_macos_frame(video_path, timestamp_ms, max_dim)
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (timestamp_ms, max_dim);
        Err(RenderError::Decode(
            "native frame extraction is currently implemented on macOS".into(),
        ))
    }
}

#[cfg(target_os = "macos")]
mod macos_decode {
    use std::ffi::c_void;
    use std::path::Path;
    use std::ptr;

    use objc2_av_foundation::{AVAssetImageGenerator, AVURLAsset};
    use objc2_core_foundation::{CGPoint, CGRect, CGSize};
    use objc2_core_media::{CMTime, CMTimeFlags};
    use objc2_foundation::{NSString, NSURL};

    use super::DecodedFrame;
    use crate::error::{RenderError, RenderResult};

    type CGColorSpaceRef = *const c_void;
    type CGContextRef = *const c_void;
    type CGImageRef = *const c_void;

    const K_CG_BITMAP_BYTE_ORDER_32_LITTLE: u32 = 2 << 12;
    const K_CG_IMAGE_ALPHA_PREMULTIPLIED_FIRST: u32 = 2;

    #[link(name = "CoreGraphics", kind = "framework")]
    unsafe extern "C" {
        fn CGImageGetWidth(image: CGImageRef) -> usize;
        fn CGImageGetHeight(image: CGImageRef) -> usize;
        fn CGColorSpaceCreateDeviceRGB() -> CGColorSpaceRef;
        fn CGColorSpaceRelease(space: CGColorSpaceRef);
        fn CGBitmapContextCreate(
            data: *mut c_void,
            width: usize,
            height: usize,
            bits_per_component: usize,
            bytes_per_row: usize,
            space: CGColorSpaceRef,
            bitmap_info: u32,
        ) -> CGContextRef;
        fn CGContextDrawImage(c: CGContextRef, rect: CGRect, image: CGImageRef);
        fn CGContextRelease(c: CGContextRef);
    }

    pub fn extract_macos_frame(
        video_path: &Path,
        timestamp_ms: u64,
        max_dim: u32,
    ) -> RenderResult<DecodedFrame> {
        let path_str = NSString::from_str(&video_path.to_string_lossy());
        let url = NSURL::fileURLWithPath(&path_str);
        let asset = unsafe { AVURLAsset::URLAssetWithURL_options(&url, None) };
        let generator = unsafe { AVAssetImageGenerator::assetImageGeneratorWithAsset(&asset) };

        let zero_time = CMTime {
            value: 0,
            timescale: 1000,
            flags: CMTimeFlags::Valid,
            epoch: 0,
        };
        let target_time = CMTime {
            value: timestamp_ms as i64,
            timescale: 1000,
            flags: CMTimeFlags::Valid,
            epoch: 0,
        };

        unsafe {
            generator.setAppliesPreferredTrackTransform(true);
            generator.setMaximumSize(CGSize::new(max_dim as f64, max_dim as f64));
            generator.setRequestedTimeToleranceBefore(zero_time);
            generator.setRequestedTimeToleranceAfter(zero_time);
        }

        #[allow(deprecated)]
        let cg_image = unsafe {
            generator.copyCGImageAtTime_actualTime_error(target_time, ptr::null_mut())
        }
        .map_err(|e| RenderError::Decode(format!("AVAssetImageGenerator failed: {e}")))?;

        let raw_img: CGImageRef = (&*cg_image as *const objc2_core_graphics::CGImage).cast();
        let w = unsafe { CGImageGetWidth(raw_img) };
        let h = unsafe { CGImageGetHeight(raw_img) };
        if w == 0 || h == 0 {
            return Err(RenderError::Decode("decoded empty image".into()));
        }

        let bytes_per_row = w * 4;
        let mut bgra = vec![0_u8; bytes_per_row * h];
        let color_space = unsafe { CGColorSpaceCreateDeviceRGB() };
        let bitmap_info = K_CG_BITMAP_BYTE_ORDER_32_LITTLE | K_CG_IMAGE_ALPHA_PREMULTIPLIED_FIRST;
        let ctx = unsafe {
            CGBitmapContextCreate(
                bgra.as_mut_ptr().cast(),
                w,
                h,
                8,
                bytes_per_row,
                color_space,
                bitmap_info,
            )
        };
        if ctx.is_null() {
            unsafe { CGColorSpaceRelease(color_space) };
            return Err(RenderError::Decode("CGBitmapContextCreate failed".into()));
        }

        let rect = CGRect::new(CGPoint::new(0.0, 0.0), CGSize::new(w as f64, h as f64));
        unsafe {
            CGContextDrawImage(ctx, rect, raw_img);
            CGContextRelease(ctx);
            CGColorSpaceRelease(color_space);
        }

        Ok(DecodedFrame {
            width: w as u32,
            height: h as u32,
            bgra,
        })
    }
}
