use uuid::Uuid;

use fovea_core::{AnnotationItem, AnnotationKind, EditCommand, TimelineMap, ZoomRegion};

use crate::state::StudioState;

pub fn split_clip_at_playhead(state: &mut StudioState) {
    let timeline = TimelineMap::from_project(state.project());
    let Some(mapped) = timeline.map_output_to_source(state.playhead_ms) else {
        return;
    };
    if state
        .apply_edit(EditCommand::SplitClip {
            clip_id: mapped.clip_id,
            split_source_ms: mapped.source_ms,
        })
        .is_ok()
    {
        state.last_status = "Split clip at playhead".into();
    } else {
        state.last_status = "Move playhead inside clip to split".into();
    }
}

pub fn delete_selected_clip(state: &mut StudioState) {
    let Some(clip_id) = state.selected_clip_id else {
        return;
    };
    if state.project().clips.len() <= 1 {
        state.last_status = "Cannot delete the last remaining clip".into();
        return;
    }
    if state.apply_edit(EditCommand::DeleteClip { clip_id }).is_ok() {
        state.selected_clip_id = state.project().clips.first().map(|c| c.id);
        state.last_status = "Deleted clip segment".into();
    }
}

pub fn set_selected_zoom_scale(state: &mut StudioState, scale: f32) {
    let Some(id) = state.selected_zoom_id else {
        return;
    };
    let Some(mut z) = state.project().zoom_regions.iter().find(|r| r.id == id).cloned() else {
        return;
    };
    z.scale = scale;
    let _ = state.apply_edit(EditCommand::RemoveZoomRegion { id });
    let _ = state.apply_edit(EditCommand::AddZoomRegion(z));
    state.last_status = format!("Zoom scale set to {scale:.1}x");
}

pub fn delete_selected_zoom(state: &mut StudioState) {
    let Some(id) = state.selected_zoom_id.take() else {
        return;
    };
    let _ = state.apply_edit(EditCommand::RemoveZoomRegion { id });
    state.last_status = "Removed zoom region".into();
}

pub fn add_zoom_at_playhead(state: &mut StudioState) -> u64 {
    let start = state.playhead_ms;
    let total = state.output_duration_ms();
    let end = (start + 2_600).min(total.max(start + 800));
    let id = Uuid::new_v4();
    let _ = state.apply_edit(EditCommand::AddZoomRegion(ZoomRegion {
        id,
        start_ms: start,
        end_ms: end,
        scale: 1.8,
        focus_x: 0.52,
        focus_y: 0.48,
        auto_generated: false,
    }));
    state.selected_zoom_id = Some(id);
    state.last_status = "Added 1.8x zoom region".into();
    (start + 450).min(end)
}

pub fn add_privacy_blur_at_playhead(state: &mut StudioState) {
    let start = state.playhead_ms;
    let _ = state.apply_edit(EditCommand::AddAnnotation(AnnotationItem {
        id: Uuid::new_v4(),
        start_ms: start,
        end_ms: start + 4_000,
        x: 0.58,
        y: 0.16,
        width: 0.30,
        height: 0.14,
        payload: AnnotationKind::PrivacyBlur { blur_strength: 18.0 },
    }));
    state.last_status = "Added Privacy Blur box on Stage".into();
}
