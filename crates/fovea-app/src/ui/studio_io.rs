use std::path::Path;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;

use gpui_kit::*;
use image::{Frame, RgbaImage};
use smallvec::smallvec;

use fovea_core::{BackgroundStyle, EditCommand, FoveaBundle};
use fovea_render::{DecodedFrame, ExportOptions, export_project, extract_video_frame};

use crate::state::{BackgroundTab, StudioState};
use crate::ui::studio::StudioView;

pub fn decode_render_image(path: &Path, ms: u64) -> Option<Arc<RenderImage>> {
    let decoded = extract_video_frame(path, ms, 1280).ok()?;
    decoded_to_render_image(&decoded)
}

pub fn decoded_to_render_image(decoded: &DecodedFrame) -> Option<Arc<RenderImage>> {
    let buf = RgbaImage::from_raw(decoded.width, decoded.height, decoded.bgra.clone())?;
    Some(Arc::new(RenderImage::new(smallvec![Frame::new(buf)])))
}

pub fn load_custom_bg(path: &Path) -> Option<(Arc<RenderImage>, DecodedFrame)> {
    let mut rgba = image::open(path).ok()?.to_rgba8();
    let (w, h) = (rgba.width(), rgba.height());
    for px in rgba.as_chunks_mut::<4>().0 {
        px.swap(0, 2); // GPUI RenderImage and compositor expect BGRA pixel order.
    }
    let bgra = rgba.into_raw();
    let decoded = DecodedFrame { width: w, height: h, bgra };
    let render = decoded_to_render_image(&decoded)?;
    Some((render, decoded))
}

pub fn pick_custom_wallpaper(cx: &mut Context<StudioView>) {
    let rx = cx.prompt_for_paths(PathPromptOptions {
        files: true,
        directories: false,
        multiple: false,
        prompt: Some("Choose Wallpaper Image".into()),
    });
    cx.spawn(async move |this, cx| {
        let Ok(Ok(Some(paths))) = rx.await else {
            return;
        };
        let Some(path) = paths.into_iter().next() else {
            return;
        };
        let loaded = load_custom_bg(&path);
        let _ = this.update(cx, |view, cx| {
            if let Some((img_handle, decoded)) = loaded {
                view.custom_bg_image = Some(img_handle);
                view.custom_bg_decoded = Some(decoded);
                view.state.bg_tab = BackgroundTab::Image;
                let mut f = view.state.project().frame.clone();
                f.background = BackgroundStyle::Image {
                    path: path.to_string_lossy().into(),
                    blur_radius: view.state.bg_blur_px,
                };
                let _ = view.state.apply_edit(EditCommand::UpdateFrameStyle(f));
                view.state.last_status = format!(
                    "Wallpaper loaded: {}",
                    path.file_name().and_then(|s| s.to_str()).unwrap_or("image")
                );
            } else {
                view.state.last_status = "Unsupported image file (choose PNG/JPG)".into();
            }
            cx.notify();
        });
    })
    .detach();
}

pub fn spawn_video_export(view: &mut StudioView, cx: &mut Context<StudioView>) {
    if view.state.export_progress.is_some() {
        return;
    }
    let out_path = view.state.next_export_path();
    let opts = ExportOptions {
        format: view.state.export_format,
        output_path: out_path.clone(),
        height: view.state.export_height,
        fps: view.state.export_fps,
        remove_background: view.state.remove_background,
    };
    let source_video = view.state.active_bundle.as_ref().map(|b| b.screen_video_path());
    let custom_bg = view.custom_bg_decoded.clone();
    let project = view.state.project().clone();
    let telemetry = view.state.telemetry.clone();

    view.state.export_progress = Some(0.02);
    view.state.last_status = "Rendering video export...".into();
    cx.notify();

    let pct_bits = Arc::new(AtomicU32::new(2));
    let worker_pct = Arc::clone(&pct_bits);
    let (tx, rx) = std::sync::mpsc::channel::<Result<std::path::PathBuf, String>>();
    std::thread::spawn(move || {
        let res = export_project(
            source_video.as_deref(),
            custom_bg.as_ref(),
            &project,
            &telemetry,
            &opts,
            |p| {
                worker_pct.store((p * 100.0).round() as u32, Ordering::Relaxed);
            },
        )
        .map_err(|e| e.to_string());
        let _ = tx.send(res);
    });

    cx.spawn(async move |this, cx| {
        loop {
            cx.background_executor().timer(Duration::from_millis(80)).await;
            if let Ok(result) = rx.try_recv() {
                let _ = this.update(cx, |view, cx| {
                    view.state.export_progress = None;
                    match result {
                        Ok(saved) => {
                            view.state.last_status = format!(
                                "Exported {}",
                                saved.file_name().and_then(|s| s.to_str()).unwrap_or("video")
                            );
                            view.state.last_export_path = Some(saved);
                        }
                        Err(err) => {
                            view.state.last_status = format!("Export failed: {err}");
                        }
                    }
                    cx.notify();
                });
                break;
            }
            let p = (pct_bits.load(Ordering::Relaxed) as f32 / 100.0).clamp(0.02, 0.99);
            let alive = this
                .update(cx, |view, cx| {
                    view.state.export_progress = Some(p);
                    cx.notify();
                    true
                })
                .unwrap_or(false);
            if !alive {
                break;
            }
        }
    })
    .detach();
}

pub fn export_project_bundle(state: &mut StudioState) {
    let doc = state.project().clone();
    if let Some(ref mut bundle) = state.active_bundle {
        bundle.set_project(doc);
        match bundle.save_project() {
            Ok(()) => state.last_status = format!("Saved {}", bundle.root().display()),
            Err(e) => state.last_status = format!("Save error: {e}"),
        }
        return;
    }
    let out_path = StudioState::next_bundle_path();
    match FoveaBundle::create(&out_path, &doc.title, doc.source.width, doc.source.height, doc.source.fps) {
        Ok(mut bundle) => {
            bundle.set_project(doc);
            let _ = bundle.save_project();
            state.last_status = format!("Saved project: {}", out_path.display());
            state.active_bundle_path = Some(out_path);
            state.active_bundle = Some(bundle);
        }
        Err(e) => state.last_status = format!("Export error: {e}"),
    }
}

pub fn reveal_path_in_finder(path: &Path) {
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open").arg("-R").arg(path).spawn();
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = path;
    }
}
