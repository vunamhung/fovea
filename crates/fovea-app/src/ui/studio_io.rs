use std::path::Path;
use std::sync::Arc;

use gpui_kit::*;
use image::{Frame, RgbaImage};
use smallvec::smallvec;

use fovea_core::{BackgroundStyle, EditCommand, FoveaBundle};
use fovea_render::extract_video_frame;

use crate::state::{BackgroundTab, StudioState};
use crate::ui::studio::StudioView;

pub fn decode_render_image(path: &Path, ms: u64) -> Option<Arc<RenderImage>> {
    let decoded = extract_video_frame(path, ms, 1280).ok()?;
    let buf = RgbaImage::from_raw(decoded.width, decoded.height, decoded.bgra)?;
    Some(Arc::new(RenderImage::new(smallvec![Frame::new(buf)])))
}

pub fn load_image_bgra(path: &Path) -> Option<Arc<RenderImage>> {
    let mut rgba = image::open(path).ok()?.to_rgba8();
    for px in rgba.as_chunks_mut::<4>().0 {
        px.swap(0, 2); // GPUI RenderImage expects BGRA pixel order.
    }
    Some(Arc::new(RenderImage::new(smallvec![Frame::new(rgba)])))
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
        let loaded = load_image_bgra(&path);
        let _ = this.update(cx, |view, cx| {
            if let Some(img_handle) = loaded {
                view.custom_bg_image = Some(img_handle);
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
    match FoveaBundle::create(
        &out_path,
        &doc.title,
        doc.source.width,
        doc.source.height,
        doc.source.fps,
    ) {
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
