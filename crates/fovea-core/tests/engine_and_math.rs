use fovea_core::{
    CameraPose, CommandEngine, CoreError, CursorStyle, EditCommand, FrameStyle, FreezeSegment,
    MouseButton, ProjectDoc, SpeedSegment, SpringState, StageTransform, TelemetryEvent,
    TimelineMap, ZoomRegion, evaluate_camera_pose, evaluate_cursor_pose,
};
use uuid::Uuid;

fn sample_project() -> ProjectDoc {
    let mut doc = ProjectDoc::new("Timeline Test", 1920, 1080, 60);
    doc.source.duration_ms = 10_000;
    doc.clips[0].source_start_ms = 0;
    doc.clips[0].source_end_ms = 10_000;
    doc
}

#[test]
fn timeline_maps_split_speed_and_freeze_segments() {
    let mut engine = CommandEngine::new(sample_project());
    let clip_id = engine.project().clips[0].id;
    engine
        .apply(EditCommand::SplitClip {
            clip_id,
            split_source_ms: 4_000,
        })
        .expect("split");
    let second_id = engine.project().clips[1].id;
    engine
        .apply(EditCommand::TrimClip {
            clip_id: second_id,
            source_start_ms: 6_000,
            source_end_ms: 10_000,
        })
        .expect("trim");
    engine
        .apply(EditCommand::AddFreezeSegment(FreezeSegment {
            id: Uuid::new_v4(),
            source_timestamp_ms: 2_000,
            duration_ms: 1_500,
        }))
        .expect("freeze");
    engine
        .apply(EditCommand::AddSpeedSegment(SpeedSegment {
            id: Uuid::new_v4(),
            start_ms: 6_000,
            end_ms: 10_000,
            rate: 2.0,
        }))
        .expect("speed");

    let map = TimelineMap::from_project(engine.project());
    // Clip1: 4000ms + 1500ms freeze = 5500ms; Clip2: 4000ms at 2x = 2000ms => total 7500ms
    assert_eq!(map.output_duration_ms(), 7_500);

    let frozen = map.map_output_to_source(2_750).expect("frozen frame");
    assert!(frozen.is_frozen);
    assert_eq!(frozen.source_ms, 2_000);

    let fast = map.map_output_to_source(6_500).expect("fast frame");
    assert!(!fast.is_frozen);
    assert!((fast.effective_speed - 2.0).abs() < 1e-3);
    assert_eq!(fast.source_ms, 8_000);
}

#[test]
fn command_engine_rejects_invalid_ranges_and_supports_atomic_transactions() {
    let mut engine = CommandEngine::new(sample_project());
    let clip_id = engine.project().clips[0].id;

    let err = engine.apply(EditCommand::TrimClip {
        clip_id,
        source_start_ms: 5_000,
        source_end_ms: 5_000,
    });
    assert!(matches!(err, Err(CoreError::InvalidRange(_))));

    // Empty transaction must not pollute undo stack.
    engine.begin_transaction();
    engine.commit_transaction();
    assert!(!engine.can_undo());

    // Multi-command AI transaction undoes in one step.
    engine.begin_transaction();
    engine
        .apply(EditCommand::SetTitle {
            title: "AI Polished".into(),
        })
        .unwrap();
    engine
        .apply(EditCommand::AddZoomRegion(ZoomRegion {
            id: Uuid::new_v4(),
            start_ms: 1_000,
            end_ms: 3_000,
            scale: 2.0,
            focus_x: 0.7,
            focus_y: 0.3,
            auto_generated: true,
        }))
        .unwrap();
    engine.commit_transaction();

    assert_eq!(engine.project().title, "AI Polished");
    assert_eq!(engine.project().zoom_regions.len(), 1);
    assert!(engine.undo());
    assert_eq!(engine.project().title, "Timeline Test");
    assert!(engine.project().zoom_regions.is_empty());
    assert!(engine.redo());
    assert_eq!(engine.project().zoom_regions.len(), 1);
}

#[test]
fn spring_camera_cursor_pose_and_stage_transform_round_trip() {
    let mut spring = SpringState::new(1.0);
    for _ in 0..90 {
        spring.step(2.0, 180.0, 26.0, 1.0 / 60.0);
    }
    assert!((spring.value - 2.0).abs() < 0.02);

    let mut doc = sample_project();
    doc.zoom_regions.push(ZoomRegion {
        id: Uuid::new_v4(),
        start_ms: 500,
        end_ms: 2_500,
        scale: 2.0,
        focus_x: 0.75,
        focus_y: 0.25,
        auto_generated: false,
    });
    let cam = evaluate_camera_pose(&doc, 1_500);
    assert!(cam.scale > 1.8 && cam.scale <= 2.05);

    let events = vec![
        TelemetryEvent::cursor_move(0, 0.2, 0.2),
        TelemetryEvent::cursor_click(1_000, 0.5, 0.5, MouseButton::Left, None),
    ];
    let pose = evaluate_cursor_pose(&events, &CursorStyle::default(), 1_040).expect("pose");
    assert!(pose.scale < CursorStyle::default().scale); // click bounce compresses scale briefly

    let stage = StageTransform::compute(
        1280.0,
        720.0,
        1920,
        1080,
        &FrameStyle::default(),
        &CameraPose {
            scale: 1.8,
            focus_x: 0.6,
            focus_y: 0.4,
            velocity_x: 0.0,
            velocity_y: 0.0,
        },
    );
    let (sx, sy) = stage.source_to_stage_point(0.6, 0.4);
    let (rx, ry) = stage.stage_to_source_point(sx, sy);
    assert!((rx - 0.6).abs() < 1e-4 && (ry - 0.4).abs() < 1e-4);
}
