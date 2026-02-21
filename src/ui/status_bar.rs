use egui::Ui;

use crate::svg_document::SvgDocument;
use crate::viewport::Viewport;

pub fn draw_status_bar(
    ui: &mut Ui,
    doc: Option<&SvgDocument>,
    viewport: &Viewport,
    position_display: &str,
    error_msg: Option<&str>,
    render_size: Option<(u32, u32)>,
) {
    ui.horizontal(|ui| {
        if let Some(err) = error_msg {
            ui.colored_label(egui::Color32::RED, err);
            return;
        }

        if let Some(doc) = doc {
            ui.label(doc.filename());
            ui.separator();
            ui.label(format!("{}x{}", doc.width as u32, doc.height as u32));
            if let Some((rw, rh)) = render_size {
                ui.separator();
                ui.label(format!("Render: {}x{}", rw, rh));
            }
            ui.separator();
            ui.label(format!("Zoom: {:.0}%", viewport.zoom_percent()));
            if !position_display.is_empty() {
                ui.separator();
                ui.label(position_display);
            }
            ui.separator();
            ui.label(doc.file_size_display());
        } else {
            ui.label("No file loaded");
        }
    });
}
