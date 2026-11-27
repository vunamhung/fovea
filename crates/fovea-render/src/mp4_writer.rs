use std::path::Path;
use std::ptr::{self, NonNull};
use std::sync::mpsc;
use std::time::Duration;

use block2::RcBlock;
use objc2::AnyThread;
use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2_av_foundation::{
    AVAssetWriter, AVAssetWriterInput, AVAssetWriterInputPixelBufferAdaptor, AVAssetWriterStatus,
    AVFileTypeMPEG4, AVMediaTypeVideo, AVVideoCodecKey, AVVideoCodecTypeH264, AVVideoHeightKey,
    AVVideoWidthKey,
};
use objc2_core_media::{CMTime, CMTimeFlags};
use objc2_core_video::{
    CVPixelBuffer, CVPixelBufferCreate, CVPixelBufferGetBaseAddress, CVPixelBufferGetBytesPerRow,
    CVPixelBufferLockBaseAddress, CVPixelBufferLockFlags, CVPixelBufferUnlockBaseAddress,
};
use objc2_foundation::{NSDictionary, NSNumber, NSString, NSURL};

use crate::error::{RenderError, RenderResult};

const PIXEL_FORMAT_32BGRA: u32 = 0x4247_5241;

/// Hardware VideoToolbox H.264 MP4 encoder backed by macOS AVAssetWriter.
pub struct Mp4ExportWriter {
    writer: Retained<AVAssetWriter>,
    input: Retained<AVAssetWriterInput>,
    adaptor: Retained<AVAssetWriterInputPixelBufferAdaptor>,
    width: u32,
    height: u32,
    fps: u32,
    frame_index: i64,
}

impl Mp4ExportWriter {
    pub fn new(output_path: &Path, width: u32, height: u32, fps: u32) -> RenderResult<Self> {
        let w = width.max(2) & !1;
        let h = height.max(2) & !1;
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        if output_path.exists() {
            let _ = std::fs::remove_file(output_path);
        }
        let path_str = NSString::from_str(&output_path.to_string_lossy());
        let url = NSURL::fileURLWithPath(&path_str);
        let file_type = unsafe { AVFileTypeMPEG4 }
            .ok_or_else(|| RenderError::Encode("AVFileTypeMPEG4 unavailable".into()))?;
        let writer = unsafe { AVAssetWriter::assetWriterWithURL_fileType_error(&url, file_type) }
            .map_err(|e| RenderError::Encode(format!("AVAssetWriter init failed: {e}")))?;

        let media_type = unsafe { AVMediaTypeVideo }
            .ok_or_else(|| RenderError::Encode("AVMediaTypeVideo unavailable".into()))?;
        let codec_key = unsafe { AVVideoCodecKey }
            .ok_or_else(|| RenderError::Encode("AVVideoCodecKey unavailable".into()))?;
        let codec_val = unsafe { AVVideoCodecTypeH264 }
            .ok_or_else(|| RenderError::Encode("AVVideoCodecTypeH264 unavailable".into()))?;
        let w_key = unsafe { AVVideoWidthKey }
            .ok_or_else(|| RenderError::Encode("AVVideoWidthKey unavailable".into()))?;
        let h_key = unsafe { AVVideoHeightKey }
            .ok_or_else(|| RenderError::Encode("AVVideoHeightKey unavailable".into()))?;

        let w_num = NSNumber::new_u32(w);
        let h_num = NSNumber::new_u32(h);
        let keys: [&NSString; 3] = [codec_key, w_key, h_key];
        let vals: [&AnyObject; 3] = [codec_val.as_ref(), w_num.as_ref(), h_num.as_ref()];
        let settings = NSDictionary::from_slices(&keys, &vals);

        let input = unsafe {
            AVAssetWriterInput::initWithMediaType_outputSettings(
                AVAssetWriterInput::alloc(),
                media_type,
                Some(&settings),
            )
        };
        unsafe { input.setExpectsMediaDataInRealTime(false) };
        let adaptor = unsafe {
            AVAssetWriterInputPixelBufferAdaptor::assetWriterInputPixelBufferAdaptorWithAssetWriterInput_sourcePixelBufferAttributes(
                &input,
                None,
            )
        };
        unsafe {
            if !writer.canAddInput(&input) {
                return Err(RenderError::Encode("cannot add video input".into()));
            }
            writer.addInput(&input);
            if !writer.startWriting() {
                return Err(RenderError::Encode("startWriting failed".into()));
            }
            writer.startSessionAtSourceTime(CMTime {
                value: 0,
                timescale: fps.max(1) as i32,
                flags: CMTimeFlags::Valid,
                epoch: 0,
            });
        }
        Ok(Self {
            writer,
            input,
            adaptor,
            width: w,
            height: h,
            fps: fps.max(1),
            frame_index: 0,
        })
    }

    pub fn append_bgra(&mut self, bgra: &[u8]) -> RenderResult<()> {
        let mut waited = 0_u32;
        while !unsafe { self.input.isReadyForMoreMediaData() } {
            if unsafe { self.writer.status() } != AVAssetWriterStatus::Writing {
                return Err(RenderError::Encode("writer aborted while waiting".into()));
            }
            std::thread::sleep(Duration::from_millis(2));
            waited += 1;
            if waited > 2_000 {
                return Err(RenderError::Encode("timed out waiting for encoder".into()));
            }
        }

        let mut pb_ptr: *mut CVPixelBuffer = ptr::null_mut();
        let status = unsafe {
            CVPixelBufferCreate(
                None,
                self.width as usize,
                self.height as usize,
                PIXEL_FORMAT_32BGRA,
                None,
                NonNull::from(&mut pb_ptr),
            )
        };
        if status != 0 || pb_ptr.is_null() {
            return Err(RenderError::Encode(format!("CVPixelBufferCreate failed ({status})")));
        }

        let pb_ref = unsafe { &*pb_ptr };
        unsafe { CVPixelBufferLockBaseAddress(pb_ref, CVPixelBufferLockFlags(0)) };
        let base = CVPixelBufferGetBaseAddress(pb_ref) as *mut u8;
        let stride = CVPixelBufferGetBytesPerRow(pb_ref);
        let row_bytes = (self.width as usize) * 4;
        if !base.is_null() && bgra.len() >= row_bytes * (self.height as usize) {
            for y in 0..self.height as usize {
                unsafe {
                    ptr::copy_nonoverlapping(
                        bgra.as_ptr().add(y * row_bytes),
                        base.add(y * stride),
                        row_bytes,
                    );
                }
            }
        }
        unsafe { CVPixelBufferUnlockBaseAddress(pb_ref, CVPixelBufferLockFlags(0)) };

        let pts = CMTime {
            value: self.frame_index,
            timescale: self.fps as i32,
            flags: CMTimeFlags::Valid,
            epoch: 0,
        };
        let ok = unsafe {
            self.adaptor
                .appendPixelBuffer_withPresentationTime(pb_ref, pts)
        };
        unsafe { CFRelease(pb_ptr.cast()) };
        if !ok {
            return Err(RenderError::Encode("appendPixelBuffer failed".into()));
        }
        self.frame_index += 1;
        Ok(())
    }

    pub fn finish(self) -> RenderResult<()> {
        unsafe { self.input.markAsFinished() };
        let (tx, rx) = mpsc::channel::<()>();
        let handler = RcBlock::new(move || {
            let _ = tx.send(());
        });
        unsafe { self.writer.finishWritingWithCompletionHandler(&handler) };
        rx.recv_timeout(Duration::from_secs(30))
            .map_err(|_| RenderError::Encode("timed out finishing MP4 export".into()))?;
        if unsafe { self.writer.status() } == AVAssetWriterStatus::Failed {
            let detail = unsafe { self.writer.error() }
                .map(|e| e.localizedDescription().to_string())
                .unwrap_or_else(|| "unknown error".into());
            return Err(RenderError::Encode(detail));
        }
        Ok(())
    }
}

#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {
    fn CFRelease(cf: *const std::ffi::c_void);
}
