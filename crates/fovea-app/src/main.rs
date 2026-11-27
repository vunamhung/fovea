use clap::Parser;
use gpui_kit::component::{Theme, ThemeMode};
use gpui_kit::*;
use tracing_subscriber::EnvFilter;

use fovea_app::cli::{CliArgs, CliCommand};
use fovea_app::ui::StudioView;

fn main() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let _ = tracing_subscriber::fmt().with_env_filter(filter).try_init();

    let args = CliArgs::parse();
    let open_path = match args.command {
        Some(CliCommand::Mcp) => {
            eprintln!("Fovea MCP server (`fovea mcp`) is scheduled for Milestone 3.");
            return;
        }
        Some(CliCommand::Open { path }) => Some(path),
        None => None,
    };

    gpui_kit::application()
        .with_assets(gpui_kit::assets::Assets)
        .run(move |cx| {
            gpui_kit::init(cx);
            Theme::change(ThemeMode::Dark, None, cx);

            let bounds = Bounds::centered(None, size(px(1280.0), px(820.0)), cx);
            let options = WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(TitlebarOptions {
                    title: Some("Fovea Studio".into()),
                    ..Default::default()
                }),
                ..Default::default()
            };

            gpui_kit::open_window(options, cx, move |_, cx| {
                cx.new(|_| StudioView::new(open_path.clone()))
            })
            .expect("failed to open Fovea Studio window");
        });
}
