use clap::Parser;
use fovea_app::cli::{CliArgs, CliCommand};
use fovea_app::demo::{build_demo_project, camera_at_ms};
use fovea_app::state::{RightPanelTab, SessionMode, StudioState};
use fovea_capture::{CaptureSource, PermissionReport};
use fovea_core::{AspectRatio, EditCommand};

#[test]
fn cli_parses_default_editor_open_and_mcp_subcommands() {
    let default_cli = CliArgs::try_parse_from(["fovea"]).expect("default");
    assert!(default_cli.command.is_none());
    assert!(!default_cli.editor);

    let editor_cli = CliArgs::try_parse_from(["fovea", "--editor"]).expect("editor");
    assert!(editor_cli.editor);

    let open_cli = CliArgs::try_parse_from(["fovea", "open", "/tmp/demo.fovea"]).expect("open");
    assert!(matches!(
        open_cli.command,
        Some(CliCommand::Open { ref path }) if path.to_string_lossy() == "/tmp/demo.fovea"
    ));

    let mcp_cli = CliArgs::try_parse_from(["fovea", "mcp"]).expect("mcp");
    assert!(matches!(mcp_cli.command, Some(CliCommand::Mcp)));
}

#[test]
fn studio_state_handles_recording_lifecycle_cycles_tabs_and_undo_redo() {
    let mut state = StudioState::new(
        PermissionReport::all_granted_for_test(),
        vec![
            CaptureSource::display(1, "Built-in Display", 1920, 1080),
            CaptureSource::display(2, "External 4K", 3840, 2160),
        ],
    );
    assert_eq!(state.mode, SessionMode::Idle);
    assert_eq!(state.active_tab, RightPanelTab::Inspector);
    assert_eq!(state.selected_source().unwrap().id, 1);

    state.cycle_source();
    assert_eq!(state.selected_source().unwrap().id, 2);
    state.cycle_source();
    assert_eq!(state.selected_source().unwrap().id, 1);

    state.set_tab(RightPanelTab::AiChat);
    assert_eq!(state.active_tab, RightPanelTab::AiChat);

    state.cycle_aspect_ratio().expect("cycle ratio");
    assert_eq!(state.project().frame.aspect_ratio, AspectRatio::Ratio9x16);
    assert!(state.undo());
    assert_eq!(state.project().frame.aspect_ratio, AspectRatio::Ratio16x9);
    assert!(state.redo());
    assert_eq!(state.project().frame.aspect_ratio, AspectRatio::Ratio9x16);

    state
        .apply_edit(EditCommand::SetTitle {
            title: "Product Walkthrough".into(),
        })
        .expect("title");
    assert_eq!(state.project().title, "Product Walkthrough");

    state.mark_recording_started("/tmp/test.fovea".into());
    assert_eq!(state.mode, SessionMode::Recording);
    state.mark_recording_stopped(420, 7_000);
    assert_eq!(state.mode, SessionMode::Editing);
    assert_eq!(state.project().source.duration_ms, 7_000);
}

#[test]
fn demo_camera_ramps_smoothly_across_zoom_regions() {
    let doc = build_demo_project(1920, 1080);
    let (s0, _, _) = camera_at_ms(&doc.zoom_regions, 0);
    let (s_mid, fx, fy) = camera_at_ms(&doc.zoom_regions, 2_000);
    let (s_end, _, _) = camera_at_ms(&doc.zoom_regions, 7_000);
    assert!((s0 - 1.0).abs() < 1e-4);
    assert!((s_mid - 1.5).abs() < 1e-4);
    assert!(fx < 0.45 && fy < 0.48);
    assert!((s_end - 1.0).abs() < 1e-4);
}
