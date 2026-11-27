use std::path::PathBuf;
use std::sync::mpsc;
use std::time::Duration;

use block2::RcBlock;
use objc2::AnyThread;
use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2_av_foundation::{
    AVAssetWriter, AVAssetWriterInput, AVAssetWriterInputPixelBufferAdaptor, AVAssetWriterStatus,
    AVFileTypeQuickTimeMovie, AVMediaTypeAudio, AVMediaTypeVideo, AVVideoCodecKey,
    AVVideoCodecTypeH264, AVVideoHeightKey, AVVideoWidthKey,
};
use objc2_core_media::{CMSampleBuffer, CMTime, CMTimeFlags};
use objc2_foundation::{NSDictionary, NSNumber, NSString, NSURL};

use crate::backend::{CaptureStats, WriterGuard};
use crate::error::{CaptureError, CaptureResult};

pub struct NativeMovieWriter {
    writer: Retained<AVAssetWriter>,
    video_input: Retained<AVAssetWriterInput>,
    pixel_adaptor: Retained<AVAssetWriterInputPixelBufferAdaptor>,
    audio_input: Option<Retained<AVAssetWriterInput>>,
    session_started: bool,
    first_pts_ms: Option<u64>,
    logged_failure: bool,
    guard: WriterGuard,
}

// Safety: NativeMovieWriter is exclusively accessed behind an Arc<Mutex<_>>.
unsafe impl Send for NativeMovieWriter {}

impl NativeMovieWriter {
    pub fn new(
        output_path: PathBuf,
        width: u32,
        height: u32,
        with_audio: bool,
    ) -> CaptureResult<Self> {
        if output_path.exists() {
            let _ = std::fs::remove_file(&output_path);
        }
        let path_str = NSString::from_str(&output_path.to_string_lossy());
        let url = NSURL::fileURLWithPath(&path_str);
        let file_type = unsafe { AVFileTypeQuickTimeMovie }
            .ok_or_else(|| CaptureError::Backend("AVFileTypeQuickTimeMovie unavailable".into()))?;

        let writer = unsafe { AVAssetWriter::assetWriterWithURL_fileType_error(&url, file_type) }
            .map_err(|e| CaptureError::Backend(format!("AVAssetWriter init failed: {e}")))?;

        let video_input = build_video_input(width, height)?;
        let pixel_adaptor = unsafe {
            AVAssetWriterInputPixelBufferAdaptor::assetWriterInputPixelBufferAdaptorWithAssetWriterInput_sourcePixelBufferAttributes(
                &video_input,
                None,
            )
        };
        unsafe {
            if !writer.canAddInput(&video_input) {
                return Err(CaptureError::Backend("cannot add video input".into()));
            }
            writer.addInput(&video_input);
        }

        let audio_input = if with_audio {
            let input = build_audio_input()?;
            unsafe {
                if writer.canAddInput(&input) {
                    writer.addInput(&input);
                    Some(input)
                } else {
                    None
                }
            }
        } else {
            None
        };

        if !unsafe { writer.startWriting() } {
            let detail = unsafe { writer.error() }
                .map(|e| e.localizedDescription().to_string())
                .unwrap_or_else(|| "unknown error".into());
            return Err(CaptureError::Backend(format!(
                "AVAssetWriter startWriting failed: {detail}"
            )));
        }

        Ok(Self {
            writer,
            video_input,
            pixel_adaptor,
            audio_input,
            session_started: false,
            first_pts_ms: None,
            logged_failure: false,
            guard: WriterGuard::new(output_path),
        })
    }

    pub fn append_video_sample(&mut self, sample: &CMSampleBuffer) {
        if !unsafe { sample.is_valid() } || !unsafe { sample.data_is_ready() } {
            return;
        }
        // ScreenCaptureKit delivers idle/blank status buffers with no CVImageBuffer;
        // skipping them avoids poisoning AVAssetWriter into AVAssetWriterStatusFailed.
        let Some(pixel_buf) = (unsafe { sample.image_buffer() }) else {
            return;
        };
        let pts: CMTime = unsafe { sample.presentation_time_stamp() };
        if !pts.flags.contains(CMTimeFlags::Valid) || pts.timescale <= 0 {
            return;
        }
        if unsafe { self.writer.status() } != AVAssetWriterStatus::Writing {
            self.log_writer_error();
            return;
        }
        let pts_ms = cm_time_to_ms(pts);
        if !self.session_started {
            unsafe { self.writer.startSessionAtSourceTime(pts) };
            self.session_started = true;
            self.first_pts_ms = Some(pts_ms);
        }
        if !unsafe { self.video_input.isReadyForMoreMediaData() } {
            return;
        }
        let ok = unsafe {
            self.pixel_adaptor
                .appendPixelBuffer_withPresentationTime(&pixel_buf, pts)
        };
        if ok {
            let elapsed = pts_ms.saturating_sub(self.first_pts_ms.unwrap_or(pts_ms));
            self.guard.record_video_frame(elapsed);
        } else {
            self.log_writer_error();
        }
    }

    pub fn append_audio_sample(&mut self, sample: &CMSampleBuffer) {
        if !self.session_started
            || unsafe { self.writer.status() } != AVAssetWriterStatus::Writing
            || !unsafe { sample.is_valid() }
            || !unsafe { sample.data_is_ready() }
            || unsafe { sample.num_samples() } <= 0
        {
            return;
        }
        let pts = unsafe { sample.presentation_time_stamp() };
        if !pts.flags.contains(CMTimeFlags::Valid)
            || cm_time_to_ms(pts) < self.first_pts_ms.unwrap_or(0)
        {
            return;
        }
        let Some(ref audio) = self.audio_input else {
            return;
        };
        if unsafe { audio.isReadyForMoreMediaData() }
            && unsafe { audio.appendSampleBuffer(sample) }
        {
            self.guard.record_audio_buffer();
        }
    }

    pub fn finish(mut self) -> CaptureResult<CaptureStats> {
        if self.guard.video_frames() == 0 {
            self.log_writer_error();
            unsafe { self.writer.cancelWriting() };
            return self.guard.finish_if_empty();
        }

        unsafe {
            self.video_input.markAsFinished();
            if let Some(ref audio) = self.audio_input {
                audio.markAsFinished();
            }
        }

        let (tx, rx) = mpsc::channel::<()>();
        let handler = RcBlock::new(move || {
            let _ = tx.send(());
        });
        unsafe { self.writer.finishWritingWithCompletionHandler(&handler) };
        let _ = rx.recv_timeout(Duration::from_secs(10));
        if unsafe { self.writer.status() } == AVAssetWriterStatus::Failed {
            self.log_writer_error();
        }
        Ok(self.guard.stats())
    }

    fn log_writer_error(&mut self) {
        if self.logged_failure {
            return;
        }
        self.logged_failure = true;
        let err = unsafe { self.writer.error() }
            .map(|e| e.localizedDescription().to_string())
            .unwrap_or_else(|| "none".into());
        tracing::error!(status = ?unsafe { self.writer.status() }, error = %err, "AVAssetWriter failed");
    }
}

fn cm_time_to_ms(t: CMTime) -> u64 {
    if t.timescale <= 0 || t.value <= 0 {
        return 0;
    }
    ((t.value as f64 / t.timescale as f64) * 1000.0).round() as u64
}

fn build_video_input(width: u32, height: u32) -> CaptureResult<Retained<AVAssetWriterInput>> {
    let media_type = unsafe { AVMediaTypeVideo }.ok_or_else(|| CaptureError::Backend("AVMediaTypeVideo unavailable".into()))?;
    let codec_key = unsafe { AVVideoCodecKey }.ok_or_else(|| CaptureError::Backend("AVVideoCodecKey unavailable".into()))?;
    let codec_val = unsafe { AVVideoCodecTypeH264 }.ok_or_else(|| CaptureError::Backend("AVVideoCodecTypeH264 unavailable".into()))?;
    let width_key = unsafe { AVVideoWidthKey }.ok_or_else(|| CaptureError::Backend("AVVideoWidthKey unavailable".into()))?;
    let height_key = unsafe { AVVideoHeightKey }.ok_or_else(|| CaptureError::Backend("AVVideoHeightKey unavailable".into()))?;
    let (w_num, h_num) = (NSNumber::new_u32(width), NSNumber::new_u32(height));
    let keys: [&NSString; 3] = [codec_key, width_key, height_key];
    let vals: [&AnyObject; 3] = [codec_val.as_ref(), w_num.as_ref(), h_num.as_ref()];
    let settings = NSDictionary::from_slices(&keys, &vals);
    let input = unsafe { AVAssetWriterInput::initWithMediaType_outputSettings(AVAssetWriterInput::alloc(), media_type, Some(&settings)) };
    unsafe { input.setExpectsMediaDataInRealTime(true) };
    Ok(input)
}

fn build_audio_input() -> CaptureResult<Retained<AVAssetWriterInput>> {
    let media_type = unsafe { AVMediaTypeAudio }.ok_or_else(|| CaptureError::Backend("AVMediaTypeAudio unavailable".into()))?;
    let format_key = NSString::from_str("AVFormatIDKey");
    let rate_key = NSString::from_str("AVSampleRateKey");
    let ch_key = NSString::from_str("AVNumberOfChannelsKey");
    let bps_key = NSString::from_str("AVEncoderBitRateKey");
    let (aac_id, rate, channels, bitrate) = (
        NSNumber::new_u32(u32::from_be_bytes(*b"aac ")),
        NSNumber::new_f64(48_000.0),
        NSNumber::new_u32(2),
        NSNumber::new_u32(192_000),
    );
    let keys: [&NSString; 4] = [&format_key, &rate_key, &ch_key, &bps_key];
    let vals: [&AnyObject; 4] = [aac_id.as_ref(), rate.as_ref(), channels.as_ref(), bitrate.as_ref()];
    let settings = NSDictionary::from_slices(&keys, &vals);
    let input = unsafe { AVAssetWriterInput::initWithMediaType_outputSettings(AVAssetWriterInput::alloc(), media_type, Some(&settings)) };
    unsafe { input.setExpectsMediaDataInRealTime(true) };
    Ok(input)
}
