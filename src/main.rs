mod app;
mod clipboard;
mod error;
mod export;
mod file_navigator;
mod renderer;
mod svg_document;
mod ui;
mod viewport;

use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "svg-viewer",
    version,
    about = "A fast, cross-platform SVG viewer"
)]
struct Cli {
    /// SVG file to open
    file: Option<PathBuf>,
}

fn main() -> eframe::Result<()> {
    env_logger::init();

    let cli = Cli::parse();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1024.0, 768.0])
            .with_min_inner_size([400.0, 300.0])
            .with_drag_and_drop(true),
        ..Default::default()
    };

    eframe::run_native(
        "SVG Viewer",
        options,
        Box::new(move |_cc| Ok(Box::new(app::SvgViewerApp::new(cli.file)))),
    )
}
