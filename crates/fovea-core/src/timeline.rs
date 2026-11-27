use uuid::Uuid;

use crate::schema::{ClipSegment, ProjectDoc};

#[derive(Debug, Clone, PartialEq)]
pub struct MappedFrame {
    pub source_ms: u64,
    pub clip_id: Uuid,
    pub is_frozen: bool,
    pub effective_speed: f32,
}

#[derive(Debug, Clone)]
struct TimelinePiece {
    clip_id: Uuid,
    output_start_ms: u64,
    output_duration_ms: u64,
    source_start_ms: u64,
    source_end_ms: u64,
    speed: f32,
    is_frozen: bool,
}

#[derive(Debug, Clone, Default)]
pub struct TimelineMap {
    pieces: Vec<TimelinePiece>,
    total_output_ms: u64,
}

impl TimelineMap {
    pub fn from_project(project: &ProjectDoc) -> Self {
        let mut pieces = Vec::new();
        let mut cursor_out_ms: u64 = 0;

        for clip in &project.clips {
            if clip.source_end_ms <= clip.source_start_ms {
                continue;
            }
            let breakpoints = collect_breakpoints(clip, project);
            for window in breakpoints.windows(2) {
                let seg_start = window[0];
                let seg_end = window[1];
                append_freezes(project, clip.id, seg_start, &mut pieces, &mut cursor_out_ms);

                if seg_end > seg_start {
                    let speed = active_speed(project, seg_start, seg_end);
                    let src_len = (seg_end - seg_start) as f64;
                    let out_len = ((src_len / speed as f64).round() as u64).max(1);
                    pieces.push(TimelinePiece {
                        clip_id: clip.id,
                        output_start_ms: cursor_out_ms,
                        output_duration_ms: out_len,
                        source_start_ms: seg_start,
                        source_end_ms: seg_end,
                        speed,
                        is_frozen: false,
                    });
                    cursor_out_ms += out_len;
                }
            }
            append_freezes(
                project,
                clip.id,
                clip.source_end_ms,
                &mut pieces,
                &mut cursor_out_ms,
            );
        }

        Self {
            pieces,
            total_output_ms: cursor_out_ms,
        }
    }

    pub fn output_duration_ms(&self) -> u64 {
        self.total_output_ms
    }

    pub fn map_output_to_source(&self, output_ms: u64) -> Option<MappedFrame> {
        if self.pieces.is_empty() {
            return None;
        }
        let clamped = output_ms.min(self.total_output_ms.saturating_sub(1));
        let idx = self
            .pieces
            .partition_point(|p| p.output_start_ms + p.output_duration_ms <= clamped)
            .min(self.pieces.len() - 1);
        let piece = &self.pieces[idx];
        if piece.is_frozen {
            return Some(MappedFrame {
                source_ms: piece.source_start_ms,
                clip_id: piece.clip_id,
                is_frozen: true,
                effective_speed: 0.0,
            });
        }
        let local_out = clamped.saturating_sub(piece.output_start_ms) as f64;
        let src_offset = (local_out * piece.speed as f64).round() as u64;
        let source_ms = (piece.source_start_ms + src_offset).min(piece.source_end_ms);
        Some(MappedFrame {
            source_ms,
            clip_id: piece.clip_id,
            is_frozen: false,
            effective_speed: piece.speed,
        })
    }
}

fn collect_breakpoints(clip: &ClipSegment, project: &ProjectDoc) -> Vec<u64> {
    let mut pts = vec![clip.source_start_ms, clip.source_end_ms];
    for s in &project.speed_segments {
        if s.start_ms > clip.source_start_ms && s.start_ms < clip.source_end_ms {
            pts.push(s.start_ms);
        }
        if s.end_ms > clip.source_start_ms && s.end_ms < clip.source_end_ms {
            pts.push(s.end_ms);
        }
    }
    for f in &project.freeze_segments {
        if f.source_timestamp_ms > clip.source_start_ms
            && f.source_timestamp_ms < clip.source_end_ms
        {
            pts.push(f.source_timestamp_ms);
        }
    }
    pts.sort_unstable();
    pts.dedup();
    pts
}

fn append_freezes(
    project: &ProjectDoc,
    clip_id: Uuid,
    at_source_ms: u64,
    pieces: &mut Vec<TimelinePiece>,
    cursor_out_ms: &mut u64,
) {
    for freeze in &project.freeze_segments {
        if freeze.source_timestamp_ms == at_source_ms && freeze.duration_ms > 0 {
            pieces.push(TimelinePiece {
                clip_id,
                output_start_ms: *cursor_out_ms,
                output_duration_ms: freeze.duration_ms,
                source_start_ms: at_source_ms,
                source_end_ms: at_source_ms,
                speed: 0.0,
                is_frozen: true,
            });
            *cursor_out_ms += freeze.duration_ms;
        }
    }
}

fn active_speed(project: &ProjectDoc, seg_start: u64, seg_end: u64) -> f32 {
    let mid = seg_start + (seg_end - seg_start) / 2;
    project
        .speed_segments
        .iter()
        .find(|s| mid >= s.start_ms && mid < s.end_ms && s.rate > 0.0)
        .map(|s| s.rate)
        .unwrap_or(1.0)
}
