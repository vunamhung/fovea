use std::fs::File;
use std::io::BufWriter;
use std::path::{Path, PathBuf};

use fovea_core::{AspectRatio, ProjectDoc, TelemetryEvent, TimelineMap};
use image::codecs::gif::{GifEncoder, Repeat};
use image::{Delay, Frame, RgbaImage};

use crate::compositor::composite_frame;
use crate::error::{RenderError, RenderResult};
use crate::frame_decoder::{DecodedFrame, extract_video_frame};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    Mp4,
    Gif,
}

#[derive(Debug, Clone)]
pub struct ExportOptions {
    pub format: ExportFormat,
    pub output_path: PathBuf,
    pub height: u32,
    pub fps: u32,
    pub remove_background: bool,
}

impl ExportOptions {
    pub fn dimensions(&self, ratio: AspectRatio) -> (u32, u32) {
        let h = self.height.max(360) & !1;
        let w = match ratio {
            AspectRatio::Ratio9x16 => ((h as f32 * 9.0 / 16.0).round() as u32) & !1,
            AspectRatio::Ratio1x1 => h,
            AspectRatio::Ratio4x3 => ((h as f32 * 4.0 / 3.0).round() as u32) & !1,
            _ => ((h as f32 * 16.0 / 9.0).round() as u32) & !1,
        };
        (w.max(2), h.max(2))
    }
}

/// Exports the edited project timeline to a playable hardware H.264 `.mp4` or animated `.gif`.
pub fn export_project(
    source_video_path: Option<&Path>,
    custom_bg: Option<&DecodedFrame>,
    project: &ProjectDoc,
    telemetry: &[TelemetryEvent],
    opts: &ExportOptions,
    mut on_progress: impl FnMut(f32),
) -> RenderResult<PathBuf> {
    let timeline = TimelineMap::from_project(project);
    let total_ms = timeline.output_duration_ms().max(1_000);
    let fps = match opts.format {
        ExportFormat::Mp4 => opts.fps.clamp(15, 60),
        ExportFormat::Gif => opts.fps.clamp(8, 15),
    };
    let (out_w, out_h) = opts.dimensions(project.frame.aspect_ratio);
    let total_frames = ((total_ms as f64 / 1000.0) * fps as f64).ceil().max(1.0) as u64;
    let has_video = source_video_path.is_some_and(|p| p.exists() && p.metadata().map(|m| m.len() > 0).unwrap_or(false));

    match opts.format {
        ExportFormat::Mp4 => {
            export_mp4(
                source_video_path.filter(|_| has_video),
                custom_bg,
                project,
                telemetry,
                &timeline,
                opts,
                out_w,
                out_h,
                fps,
                total_frames,
                &mut on_progress,
            )?;
        }
        ExportFormat::Gif => {
            export_gif(
                source_video_path.filter(|_| has_video),
                custom_bg,
                project,
                telemetry,
                &timeline,
                opts,
                out_w,
                out_h,
                fps,
                total_frames,
                &mut on_progress,
            )?;
        }
    }
    on_progress(1.0);
    Ok(opts.output_path.clone())
}

#[allow(clippy::too_many_arguments)]
fn export_mp4(
    source_video: Option<&Path>,
    custom_bg: Option<&DecodedFrame>,
    project: &ProjectDoc,
    telemetry: &[TelemetryEvent],
    timeline: &TimelineMap,
    opts: &ExportOptions,
    out_w: u32,
    out_h: u32,
    fps: u32,
    total_frames: u64,
    on_progress: &mut impl FnMut(f32),
) -> RenderResult<()> {
    #[cfg(target_os = "macos")]
    {
        let mut writer = crate::mp4_writer::Mp4ExportWriter::new(&opts.output_path, out_w, out_h, fps)?;
        let mut cached_src: Option<(u64, DecodedFrame)> = None;
        for idx in 0..total_frames {
            let out_ms = (idx * 1000) / fps as u64;
            let src_ms = timeline
                .map_output_to_source(out_ms)
                .map(|m| m.source_ms)
                .unwrap_or(out_ms);
            let src_frame = decode_cached(source_video, src_ms, out_w, &mut cached_src);
            let bgra = composite_frame(
                project,
                src_frame,
                custom_bg,
                telemetry,
                out_ms,
                out_w,
                out_h,
                opts.remove_background,
            );
            writer.append_bgra(&bgra)?;
            on_progress((idx + 1) as f32 / total_frames as f32);
        }
        writer.finish()
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (source_video, custom_bg, project, telemetry, timeline, opts, out_w, out_h, fps, total_frames, on_progress);
        Err(RenderError::Encode("MP4 hardware export requires macOS".into()))
    }
}

#[allow(clippy::too_many_arguments)]
fn export_gif(
    source_video: Option<&Path>,
    custom_bg: Option<&DecodedFrame>,
    project: &ProjectDoc,
    telemetry: &[TelemetryEvent],
    timeline: &TimelineMap,
    opts: &ExportOptions,
    out_w: u32,
    out_h: u32,
    fps: u32,
    total_frames: u64,
    on_progress: &mut impl FnMut(f32),
) -> RenderResult<()> {
    if let Some(parent) = opts.output_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let file = File::create(&opts.output_path)?;
    let mut encoder = GifEncoder::new_with_speed(BufWriter::new(file), 10);
    encoder
        .set_repeat(Repeat::Infinite)
        .map_err(|e| RenderError::Encode(e.to_string()))?;

    let delay = Delay::from_numer_denom_ms(1000, fps);
    let mut cached_src: Option<(u64, DecodedFrame)> = None;
    for idx in 0..total_frames {
        let out_ms = (idx * 1000) / fps as u64;
        let src_ms = timeline
            .map_output_to_source(out_ms)
            .map(|m| m.source_ms)
            .unwrap_or(out_ms);
        let src_frame = decode_cached(source_video, src_ms, out_w, &mut cached_src);
        let mut rgba = composite_frame(
            project,
            src_frame,
            custom_bg,
            telemetry,
            out_ms,
            out_w,
            out_h,
            opts.remove_background,
        );
        for px in rgba.as_chunks_mut::<4>().0 {
            px.swap(0, 2); // BGRA -> RGBA for GIF encoder.
        }
        let img = RgbaImage::from_raw(out_w, out_h, rgba)
            .ok_or_else(|| RenderError::Encode("invalid RGBA dimensions".into()))?;
        encoder
            .encode_frame(Frame::from_parts(img, 0, 0, delay))
            .map_err(|e| RenderError::Encode(e.to_string()))?;
        on_progress((idx + 1) as f32 / total_frames as f32);
    }
    Ok(())
}

fn decode_cached<'a>(
    source_video: Option<&Path>,
    src_ms: u64,
    max_w: u32,
    cache: &'a mut Option<(u64, DecodedFrame)>,
) -> Option<&'a DecodedFrame> {
    let path = source_video?;
    let bucket = (src_ms / 50) * 50;
    if cache.as_ref().is_none_or(|(t, _)| *t != bucket)
        && let Ok(frame) = extract_video_frame(path, bucket, max_w)
    {
        *cache = Some((bucket, frame));
    }
    cache.as_ref().map(|(_, f)| f)
}
