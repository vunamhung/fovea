use fovea_capture::{
    CaptureBackend, CaptureConfig, CaptureError, CaptureSource, CaptureStats, CaptureTarget,
    PermissionKind, PermissionReport, PermissionState, WriterGuard,
};
use fovea_core::FoveaBundle;
use tempfile::tempdir;

struct DummyBackend {
    running: bool,
    frames: u64,
}

impl CaptureBackend for DummyBackend {
    fn check_permissions(&self) -> PermissionReport {
        PermissionReport::all_granted_for_test()
    }

    fn open_permission_settings(&self, _kind: PermissionKind) -> Result<(), CaptureError> {
        Ok(())
    }

    fn list_sources(&self) -> Result<Vec<CaptureSource>, CaptureError> {
        Ok(vec![CaptureSource::display(1, "Built-in Display", 1920, 1080)])
    }

    fn start(&mut self, config: CaptureConfig, _bundle: &FoveaBundle) -> Result<(), CaptureError> {
        config.validate()?;
        if self.running {
            return Err(CaptureError::AlreadyRecording);
        }
        self.running = true;
        Ok(())
    }

    fn stop(&mut self) -> Result<CaptureStats, CaptureError> {
        if !self.running {
            return Ok(CaptureStats::default());
        }
        self.running = false;
        Ok(CaptureStats {
            video_frames: self.frames,
            audio_buffers: 0,
            duration_ms: 0,
        })
    }
}

#[test]
fn config_validation_rejects_zero_or_odd_dimensions() {
    let bad_zero = CaptureConfig {
        target: CaptureTarget::Display { display_id: 1 },
        width: 0,
        height: 1080,
        fps: 60,
        capture_system_audio: true,
        capture_microphone: false,
        capture_webcam: false,
        hide_os_cursor: true,
    };
    assert!(matches!(
        bad_zero.validate(),
        Err(CaptureError::InvalidConfig(_))
    ));

    let bad_odd = CaptureConfig {
        width: 1921,
        ..bad_zero
    };
    assert!(matches!(
        bad_odd.validate(),
        Err(CaptureError::InvalidConfig(_))
    ));
}

#[test]
fn idempotent_double_stop_and_zero_frame_writer_guard() {
    let dir = tempdir().unwrap();
    let bundle = FoveaBundle::create(dir.path().join("early.fovea"), "Early", 1920, 1080, 60)
        .expect("bundle");

    let mut backend = DummyBackend {
        running: false,
        frames: 0,
    };
    let cfg = CaptureConfig::default_for_display(1, 1920, 1080);
    backend.start(cfg, &bundle).expect("start");
    let first = backend.stop().expect("first stop");
    let second = backend.stop().expect("double stop must be idempotent");
    assert_eq!(first.video_frames, 0);
    assert_eq!(second.video_frames, 0);

    // Guard prevents creating corrupt 0-byte mov when stopped before frame 1.
    let movie = bundle.screen_video_path();
    std::fs::write(&movie, b"").unwrap();
    let mut guard = WriterGuard::new(movie.clone());
    let stats = guard.finish_if_empty().expect("cleanup empty");
    assert_eq!(stats.video_frames, 0);
    assert!(!movie.exists());
}

#[test]
fn permission_report_checks_required_screen_recording() {
    let mut report = PermissionReport::all_granted_for_test();
    assert!(report.all_required_granted());
    report.set(PermissionKind::ScreenRecording, PermissionState::Denied);
    assert!(!report.all_required_granted());
}
