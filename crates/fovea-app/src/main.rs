use clap::Parser;
use gpui_kit::component::{Theme, ThemeMode};
use tracing_subscriber::EnvFilter;

use fovea_app::cli::{CliArgs, CliCommand};
use fovea_app::ui::{open_hud_launcher_window, open_studio_editor_window};

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
    let start_editor = args.editor || open_path.is_some();

    gpui_kit::application()
        .with_assets(gpui_kit::assets::Assets)
        .run(move |cx| {
            gpui_kit::init(cx);
            Theme::change(ThemeMode::Light, None, cx);

            if start_editor {
                open_studio_editor_window(open_path.clone(), cx);
            } else {
                open_hud_launcher_window(cx);
            }
        });
}
