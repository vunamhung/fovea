use fovea_capture::{KeyModifiers, RawKeyEvent, TelemetrySink, filter_keystroke};
use fovea_core::{FoveaBundle, TelemetryEvent};
use tempfile::tempdir;

#[test]
fn secure_input_drops_every_keystroke_including_shortcuts() {
    let secret_char = RawKeyEvent {
        key_label: "P".into(),
        modifiers: KeyModifiers::default(),
    };
    let secret_shortcut = RawKeyEvent {
        key_label: "V".into(),
        modifiers: KeyModifiers {
            cmd: true,
            ..Default::default()
        },
    };
    assert_eq!(filter_keystroke(&secret_char, true, false), None);
    assert_eq!(filter_keystroke(&secret_shortcut, true, true), None);
}

#[test]
fn formats_shortcuts_and_respects_shortcuts_only_mode() {
    let cmd_shift_4 = RawKeyEvent {
        key_label: "4".into(),
        modifiers: KeyModifiers {
            cmd: true,
            shift: true,
            ..Default::default()
        },
    };
    assert_eq!(
        filter_keystroke(&cmd_shift_4, false, true).as_deref(),
        Some("⌘⇧4")
    );

    let ctrl_opt_z = RawKeyEvent {
        key_label: "z".into(),
        modifiers: KeyModifiers {
            control: true,
            option: true,
            ..Default::default()
        },
    };
    assert_eq!(
        filter_keystroke(&ctrl_opt_z, false, true).as_deref(),
        Some("⌃⌥Z")
    );

    let plain_typing = RawKeyEvent {
        key_label: "a".into(),
        modifiers: KeyModifiers::default(),
    };
    assert_eq!(filter_keystroke(&plain_typing, false, true), None);
    assert_eq!(
        filter_keystroke(&plain_typing, false, false).as_deref(),
        Some("A")
    );
}

#[test]
fn telemetry_sink_flushes_buffered_events_on_stop() {
    let dir = tempdir().unwrap();
    let bundle =
        FoveaBundle::create(dir.path().join("tap.fovea"), "Tap", 1920, 1080, 60).expect("bundle");

    let mut sink = TelemetrySink::new(bundle.telemetry_path());
    sink.push(TelemetryEvent::cursor_move(10, 0.1, 0.2));
    sink.push(TelemetryEvent::keystroke(50, "⌘⇧4"));
    sink.flush().expect("flush");

    let report = bundle.read_telemetry_lenient().expect("read");
    assert_eq!(report.skipped_lines, 0);
    assert_eq!(report.events.len(), 2);
}
