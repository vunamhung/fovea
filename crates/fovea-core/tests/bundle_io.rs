use std::fs::OpenOptions;
use std::io::Write;

use fovea_core::{
    AxClickContext, CoreError, FoveaBundle, MouseButton, ProjectDoc, SCHEMA_VERSION,
    TelemetryEvent,
};
use tempfile::tempdir;

#[test]
fn create_save_and_open_bundle_round_trip() {
    let dir = tempdir().expect("tempdir");
    let bundle_path = dir.path().join("demo.fovea");

    let bundle = FoveaBundle::create(&bundle_path, "Launch Demo", 1920, 1080, 60).expect("create");
    assert_eq!(bundle.project().schema_version, SCHEMA_VERSION);
    assert_eq!(bundle.project().title, "Launch Demo");
    assert_eq!(bundle.project().source.width, 1920);
    assert!(bundle.voices_dir().is_dir());

    let reopened = FoveaBundle::open(&bundle_path).expect("reopen");
    assert_eq!(reopened.project(), bundle.project());
}

#[test]
fn open_rejects_unsupported_schema_version() {
    let dir = tempdir().expect("tempdir");
    let bundle_path = dir.path().join("future.fovea");
    let bundle = FoveaBundle::create(&bundle_path, "Future", 1280, 720, 60).expect("create");

    let mut raw: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(bundle.project_json_path()).unwrap())
            .unwrap();
    raw["schema_version"] = serde_json::json!(99);
    std::fs::write(
        bundle.project_json_path(),
        serde_json::to_string_pretty(&raw).unwrap(),
    )
    .unwrap();

    let err = FoveaBundle::open(&bundle_path).expect_err("must reject v99");
    assert!(matches!(err, CoreError::UnsupportedSchemaVersion(99)));
}

#[test]
fn telemetry_append_and_lenient_recovery_from_corrupt_tail() {
    let dir = tempdir().expect("tempdir");
    let bundle_path = dir.path().join("telemetry.fovea");
    let bundle = FoveaBundle::create(&bundle_path, "Telemetry", 1920, 1080, 60).expect("create");

    let ev1 = TelemetryEvent::cursor_move(100, 0.25, 0.50);
    let ev2 = TelemetryEvent::cursor_click(
        250,
        0.40,
        0.60,
        MouseButton::Left,
        Some(AxClickContext {
            app_name: "Safari".into(),
            window_title: Some("Fovea".into()),
            role: Some("AXButton".into()),
            label: Some("Sign In".into()),
        }),
    );
    let ev3 = TelemetryEvent::keystroke(400, "⌘⇧4");

    bundle.append_telemetry(&ev1).expect("ev1");
    bundle.append_telemetry(&ev2).expect("ev2");
    bundle.append_telemetry(&ev3).expect("ev3");

    // Simulate crash mid-write leaving a truncated JSON line at EOF.
    let mut f = OpenOptions::new()
        .append(true)
        .open(bundle.telemetry_path())
        .unwrap();
    writeln!(f, "{{\"kind\":\"cursor_move\",\"timestamp_ms\":999").unwrap();

    let report = bundle.read_telemetry_lenient().expect("lenient read");
    assert_eq!(report.events, vec![ev1, ev2, ev3]);
    assert_eq!(report.skipped_lines, 1);

    // Ensure ProjectDoc defaults stay valid after serialization.
    let doc = ProjectDoc::new("Default", 1920, 1080, 60);
    assert_eq!(doc.clips.len(), 1);
}
