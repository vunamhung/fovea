use std::sync::{Arc, Mutex, mpsc};
use std::time::Duration;

use block2::RcBlock;
use dispatch2::{DispatchQueue, DispatchQueueAttr};
use fovea_core::FoveaBundle;
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2::{AnyThread, DefinedClass, define_class};
use objc2_core_media::{CMSampleBuffer, CMTime, CMTimeFlags};
use objc2_foundation::{NSError, NSObject, NSObjectProtocol};
use objc2_screen_capture_kit::{
    SCStream, SCStreamConfiguration, SCStreamOutput, SCStreamOutputType,
};

use crate::backend::{CaptureBackend, CaptureConfig, CaptureSource, CaptureStats};
use crate::error::{CaptureError, CaptureResult};
use crate::macos::permissions::{check_macos_permissions, request_macos_permission};
use crate::macos::sources::{list_macos_sources, resolve_macos_filter};
use crate::macos::writer::NativeMovieWriter;
use crate::permissions::{PermissionKind, PermissionReport, open_system_settings_url};

const PIXEL_FORMAT_BGRA: u32 = u32::from_be_bytes(*b"BGRA");

struct StreamOutputIvars {
    writer: Arc<Mutex<Option<NativeMovieWriter>>>,
}

define_class!(
    #[unsafe(super(NSObject))]
    #[ivars = StreamOutputIvars]
    struct FoveaStreamOutput;

    unsafe impl NSObjectProtocol for FoveaStreamOutput {}

    unsafe impl SCStreamOutput for FoveaStreamOutput {
        #[allow(non_snake_case)]
        #[unsafe(method(stream:didOutputSampleBuffer:ofType:))]
        unsafe fn stream_didOutputSampleBuffer_ofType(
            &self,
            _stream: &SCStream,
            sample_buffer: &CMSampleBuffer,
            r#type: SCStreamOutputType,
        ) {
            let Ok(mut guard) = self.ivars().writer.lock() else {
                return;
            };
            let Some(ref mut writer) = *guard else {
                return;
            };
            if r#type == SCStreamOutputType::Screen {
                writer.append_video_sample(sample_buffer);
            } else if r#type == SCStreamOutputType::Audio {
                writer.append_audio_sample(sample_buffer);
            }
        }
    }
);

impl FoveaStreamOutput {
    fn new(writer: Arc<Mutex<Option<NativeMovieWriter>>>) -> Retained<Self> {
        let this = Self::alloc().set_ivars(StreamOutputIvars { writer });
        unsafe { objc2::msg_send![super(this), init] }
    }
}

struct ActiveSession {
    stream: Retained<SCStream>,
    _output: Retained<FoveaStreamOutput>,
    writer: Arc<Mutex<Option<NativeMovieWriter>>>,
}

pub struct MacOsCaptureBackend {
    active: Option<ActiveSession>,
}

impl MacOsCaptureBackend {
    pub fn new() -> Self {
        Self { active: None }
    }
}

impl Default for MacOsCaptureBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl CaptureBackend for MacOsCaptureBackend {
    fn check_permissions(&self) -> PermissionReport {
        check_macos_permissions()
    }

    fn open_permission_settings(&self, kind: PermissionKind) -> CaptureResult<()> {
        let already_granted = request_macos_permission(kind);
        if already_granted {
            return Ok(());
        }
        open_system_settings_url(kind.macos_settings_url())
    }

    fn list_sources(&self) -> CaptureResult<Vec<CaptureSource>> {
        list_macos_sources()
    }

    fn start(&mut self, config: CaptureConfig, bundle: &FoveaBundle) -> CaptureResult<()> {
        config.validate()?;
        if self.active.is_some() {
            return Err(CaptureError::AlreadyRecording);
        }
        if !self.check_permissions().all_required_granted() {
            return Err(CaptureError::PermissionDenied(
                "Screen Recording permission is required".into(),
            ));
        }

        let (filter, region_rect) = resolve_macos_filter(config.target.clone())?;
        let sc_cfg = unsafe { SCStreamConfiguration::new() };
        unsafe {
            sc_cfg.setWidth(config.width as usize);
            sc_cfg.setHeight(config.height as usize);
            sc_cfg.setPixelFormat(PIXEL_FORMAT_BGRA);
            sc_cfg.setQueueDepth(5);
            sc_cfg.setShowsCursor(!config.hide_os_cursor);
            sc_cfg.setMinimumFrameInterval(CMTime {
                value: 1,
                timescale: config.fps as i32,
                flags: CMTimeFlags::Valid,
                epoch: 0,
            });
            if let Some(rect) = region_rect {
                sc_cfg.setSourceRect(rect);
            }
            if config.capture_system_audio {
                sc_cfg.setCapturesAudio(true);
                sc_cfg.setSampleRate(48_000);
                sc_cfg.setChannelCount(2);
            }
        }

        let writer = NativeMovieWriter::new(
            bundle.screen_video_path(),
            config.width,
            config.height,
            config.capture_system_audio,
        )?;
        let shared_writer = Arc::new(Mutex::new(Some(writer)));
        let output = FoveaStreamOutput::new(Arc::clone(&shared_writer));
        let proto_output = ProtocolObject::from_ref(&*output);

        let stream = unsafe {
            SCStream::initWithFilter_configuration_delegate(
                SCStream::alloc(),
                &filter.0,
                &sc_cfg,
                None,
            )
        };
        let queue = DispatchQueue::new("dev.fovea.capture.sample", DispatchQueueAttr::SERIAL);
        unsafe {
            stream
                .addStreamOutput_type_sampleHandlerQueue_error(
                    proto_output,
                    SCStreamOutputType::Screen,
                    Some(&queue),
                )
                .map_err(|e| CaptureError::Backend(format!("add video output failed: {e}")))?;
            if config.capture_system_audio {
                stream
                    .addStreamOutput_type_sampleHandlerQueue_error(
                        proto_output,
                        SCStreamOutputType::Audio,
                        Some(&queue),
                    )
                    .map_err(|e| CaptureError::Backend(format!("add audio output failed: {e}")))?;
            }
        }

        let (tx, rx) = mpsc::channel::<Result<(), String>>();
        let block = RcBlock::new(move |err: *mut NSError| {
            if err.is_null() {
                let _ = tx.send(Ok(()));
            } else {
                let msg = unsafe { &*err }.localizedDescription().to_string();
                let _ = tx.send(Err(msg));
            }
        });
        unsafe { stream.startCaptureWithCompletionHandler(Some(&block)) };
        rx.recv_timeout(Duration::from_secs(5))
            .map_err(|_| CaptureError::Backend("timed out starting ScreenCaptureKit".into()))?
            .map_err(CaptureError::Backend)?;

        self.active = Some(ActiveSession {
            stream,
            _output: output,
            writer: shared_writer,
        });
        tracing::info!(
            width = config.width,
            height = config.height,
            fps = config.fps,
            "started macOS ScreenCaptureKit recording"
        );
        Ok(())
    }

    fn stop(&mut self) -> CaptureResult<CaptureStats> {
        let Some(session) = self.active.take() else {
            return Ok(CaptureStats::default());
        };
        let (tx, rx) = mpsc::channel::<()>();
        let block = RcBlock::new(move |_err: *mut NSError| {
            let _ = tx.send(());
        });
        unsafe { session.stream.stopCaptureWithCompletionHandler(Some(&block)) };
        let _ = rx.recv_timeout(Duration::from_secs(5));

        let writer = session
            .writer
            .lock()
            .map_err(|_| CaptureError::Backend("writer mutex poisoned".into()))?
            .take();
        match writer {
            Some(w) => w.finish(),
            None => Ok(CaptureStats::default()),
        }
    }
}
