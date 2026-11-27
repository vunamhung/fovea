use fovea_core::{MouseButton, ProjectDoc, TelemetryEvent, generate_auto_zooms};
use fovea_render::{ExportFormat, ExportOptions, composite_frame, export_project};
use tempfile::tempdir;

#[test]
fn compositor_and_hardware_mp4_plus_gif_export_produce_valid_files() {
    let mut doc = ProjectDoc::new("export-smoke", 1280, 720, 30);
    doc.source.duration_ms = 900;
    doc.clips[0].source_end_ms = 900;
    let events = vec![
        TelemetryEvent::cursor_move(100, 0.4, 0.4),
        TelemetryEvent::cursor_click(300, 0.4, 0.4, MouseButton::Left, None),
    ];
    doc.zoom_regions = generate_auto_zooms(&events, 900);
    assert_eq!(doc.zoom_regions.len(), 1);

    let bgra = composite_frame(&doc, None, None, &events, 400, 320, 180, false);
    assert_eq!(bgra.len(), 320 * 180 * 4);

    let dir = tempdir().expect("tempdir");
    let gif_path = dir.path().join("demo.gif");
    let gif_opts = ExportOptions {
        format: ExportFormat::Gif,
        output_path: gif_path.clone(),
        height: 360,
        fps: 8,
        remove_background: false,
    };
    let mut last_prog = 0.0_f32;
    export_project(None, None, &doc, &events, &gif_opts, |p| last_prog = p).expect("gif export");
    assert!(gif_path.metadata().expect("gif meta").len() > 200);
    assert!((last_prog - 1.0).abs() < 1e-4);

    #[cfg(target_os = "macos")]
    {
        let mp4_path = dir.path().join("demo.mp4");
        let mp4_opts = ExportOptions {
            format: ExportFormat::Mp4,
            output_path: mp4_path.clone(),
            height: 360,
            fps: 15,
            remove_background: false,
        };
        export_project(None, None, &doc, &events, &mp4_opts, |_| {}).expect("mp4 export");
        assert!(mp4_path.metadata().expect("mp4 meta").len() > 1024);
    }
}
