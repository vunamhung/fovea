use uuid::Uuid;

use crate::command::EditCommand;
use crate::error::{CoreError, CoreResult};
use crate::schema::{ClipSegment, ProjectDoc};

#[derive(Debug, Clone)]
pub struct CommandEngine {
    project: ProjectDoc,
    undo_stack: Vec<ProjectDoc>,
    redo_stack: Vec<ProjectDoc>,
    active_tx: Option<(ProjectDoc, usize)>,
}

impl CommandEngine {
    pub fn new(project: ProjectDoc) -> Self {
        Self {
            project,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            active_tx: None,
        }
    }

    pub fn project(&self) -> &ProjectDoc {
        &self.project
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    pub fn begin_transaction(&mut self) {
        if self.active_tx.is_none() {
            self.active_tx = Some((self.project.clone(), 0));
        }
    }

    pub fn commit_transaction(&mut self) {
        if let Some((snapshot, applied)) = self.active_tx.take()
            && applied > 0
        {
            self.undo_stack.push(snapshot);
            self.redo_stack.clear();
        }
    }

    pub fn rollback_transaction(&mut self) {
        if let Some((snapshot, _)) = self.active_tx.take() {
            self.project = snapshot;
        }
    }

    pub fn apply(&mut self, command: EditCommand) -> CoreResult<()> {
        let mut next = self.project.clone();
        apply_to_doc(&mut next, command)?;

        if let Some((_, count)) = self.active_tx.as_mut() {
            *count += 1;
        } else {
            self.undo_stack.push(self.project.clone());
            self.redo_stack.clear();
        }
        self.project = next;
        Ok(())
    }

    pub fn undo(&mut self) -> bool {
        if let Some(prev) = self.undo_stack.pop() {
            self.redo_stack.push(self.project.clone());
            self.project = prev;
            true
        } else {
            false
        }
    }

    pub fn redo(&mut self) -> bool {
        if let Some(next) = self.redo_stack.pop() {
            self.undo_stack.push(self.project.clone());
            self.project = next;
            true
        } else {
            false
        }
    }
}

fn apply_to_doc(doc: &mut ProjectDoc, command: EditCommand) -> CoreResult<()> {
    match command {
        EditCommand::SetTitle { title } => {
            if title.trim().is_empty() {
                return Err(CoreError::InvalidRange("title cannot be empty".into()));
            }
            doc.title = title;
        }
        EditCommand::TrimClip {
            clip_id,
            source_start_ms,
            source_end_ms,
        } => {
            if source_end_ms <= source_start_ms {
                return Err(CoreError::InvalidRange(
                    "source_end_ms must exceed source_start_ms".into(),
                ));
            }
            let clip = doc
                .clips
                .iter_mut()
                .find(|c| c.id == clip_id)
                .ok_or_else(|| CoreError::NotFound(clip_id.to_string()))?;
            clip.source_start_ms = source_start_ms;
            clip.source_end_ms = source_end_ms;
        }
        EditCommand::SplitClip {
            clip_id,
            split_source_ms,
        } => {
            let idx = doc
                .clips
                .iter()
                .position(|c| c.id == clip_id)
                .ok_or_else(|| CoreError::NotFound(clip_id.to_string()))?;
            let clip = &doc.clips[idx];
            if split_source_ms <= clip.source_start_ms || split_source_ms >= clip.source_end_ms {
                return Err(CoreError::InvalidRange(
                    "split timestamp must lie strictly inside clip bounds".into(),
                ));
            }
            let tail = ClipSegment {
                id: Uuid::new_v4(),
                source_start_ms: split_source_ms,
                source_end_ms: clip.source_end_ms,
            };
            doc.clips[idx].source_end_ms = split_source_ms;
            doc.clips.insert(idx + 1, tail);
        }
        EditCommand::DeleteClip { clip_id } => {
            if doc.clips.len() <= 1 {
                return Err(CoreError::InvalidRange(
                    "cannot delete the final remaining clip".into(),
                ));
            }
            let idx = doc
                .clips
                .iter()
                .position(|c| c.id == clip_id)
                .ok_or_else(|| CoreError::NotFound(clip_id.to_string()))?;
            doc.clips.remove(idx);
        }
        EditCommand::ReorderClip { clip_id, new_index } => {
            let old = doc
                .clips
                .iter()
                .position(|c| c.id == clip_id)
                .ok_or_else(|| CoreError::NotFound(clip_id.to_string()))?;
            if new_index >= doc.clips.len() {
                return Err(CoreError::InvalidRange("new_index out of bounds".into()));
            }
            let clip = doc.clips.remove(old);
            doc.clips.insert(new_index, clip);
        }
        EditCommand::AddSpeedSegment(seg) => {
            if seg.end_ms <= seg.start_ms || seg.rate <= 0.0 || seg.rate > 16.0 {
                return Err(CoreError::InvalidRange("invalid speed segment".into()));
            }
            doc.speed_segments.push(seg);
        }
        EditCommand::AddFreezeSegment(seg) => {
            if seg.duration_ms == 0 {
                return Err(CoreError::InvalidRange("freeze duration must be > 0".into()));
            }
            doc.freeze_segments.push(seg);
        }
        EditCommand::AddZoomRegion(zoom) => {
            if zoom.end_ms <= zoom.start_ms || zoom.scale < 1.0 {
                return Err(CoreError::InvalidRange("invalid zoom region".into()));
            }
            doc.zoom_regions.push(zoom);
        }
        EditCommand::RemoveZoomRegion { id } => {
            let before = doc.zoom_regions.len();
            doc.zoom_regions.retain(|z| z.id != id);
            if doc.zoom_regions.len() == before {
                return Err(CoreError::NotFound(id.to_string()));
            }
        }
        EditCommand::AddAnnotation(item) => {
            if item.end_ms <= item.start_ms {
                return Err(CoreError::InvalidRange("annotation duration must be > 0".into()));
            }
            doc.annotations.push(item);
        }
        EditCommand::RemoveAnnotation { id } => {
            let before = doc.annotations.len();
            doc.annotations.retain(|a| a.id != id);
            if doc.annotations.len() == before {
                return Err(CoreError::NotFound(id.to_string()));
            }
        }
        EditCommand::UpsertCaptions { words } => doc.captions = words,
        EditCommand::InsertVoiceClip(clip) => {
            if clip.duration_ms == 0 {
                return Err(CoreError::InvalidRange("voice clip duration must be > 0".into()));
            }
            doc.voice_clips.push(clip);
        }
        EditCommand::UpdateFrameStyle(style) => doc.frame = style,
        EditCommand::UpdateCursorStyle(style) => doc.cursor = style,
        EditCommand::UpdateKeystrokeHud(style) => doc.keystroke_hud = style,
        EditCommand::UpdateWebcam(cfg) => doc.webcam = cfg,
    }
    Ok(())
}
