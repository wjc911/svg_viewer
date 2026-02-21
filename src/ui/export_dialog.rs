use egui::{Context, Window};

use crate::export::{ExportFormat, ExportSettings};

pub struct ExportDialogState {
    pub open: bool,
    pub settings: ExportSettings,
    pub aspect_locked: bool,
    pub original_width: f32,
    pub original_height: f32,
    pub result: ExportDialogResult,
}

#[derive(Clone, PartialEq)]
pub enum ExportDialogResult {
    None,
    Export,
    Cancel,
}

impl ExportDialogState {
    pub fn new() -> Self {
        Self {
            open: false,
            settings: ExportSettings::default(),
            aspect_locked: true,
            original_width: 800.0,
            original_height: 600.0,
            result: ExportDialogResult::None,
        }
    }

    pub fn open_with_dimensions(&mut self, width: f32, height: f32) {
        self.open = true;
        self.original_width = width;
        self.original_height = height;
        self.settings.width = width as u32;
        self.settings.height = height as u32;
        self.result = ExportDialogResult::None;
    }
}

pub fn draw_export_dialog(ctx: &Context, state: &mut ExportDialogState) {
    if !state.open {
        return;
    }

    let mut open = state.open;

    Window::new("Export Image")
        .open(&mut open)
        .resizable(false)
        .collapsible(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Format:");
                for fmt in ExportFormat::all() {
                    if ui
                        .selectable_label(state.settings.format == *fmt, fmt.name())
                        .clicked()
                    {
                        state.settings.format = fmt.clone();
                        // Reset alpha if format doesn't support it
                        if !fmt.supports_alpha() {
                            state.settings.include_alpha = false;
                        }
                    }
                }
            });

            ui.add_space(5.0);

            // Dimensions
            ui.horizontal(|ui| {
                ui.label("Width:");
                let old_w = state.settings.width;
                let w_response =
                    ui.add(egui::DragValue::new(&mut state.settings.width).range(1..=8192));
                if w_response.changed() && state.aspect_locked && old_w > 0 {
                    let ratio = state.original_height / state.original_width;
                    state.settings.height = (state.settings.width as f32 * ratio).round() as u32;
                }

                ui.label("Height:");
                let old_h = state.settings.height;
                let h_response =
                    ui.add(egui::DragValue::new(&mut state.settings.height).range(1..=8192));
                if h_response.changed() && state.aspect_locked && old_h > 0 {
                    let ratio = state.original_width / state.original_height;
                    state.settings.width = (state.settings.height as f32 * ratio).round() as u32;
                }

                let lock_label = if state.aspect_locked {
                    "\u{1F512}"
                } else {
                    "\u{1F513}"
                };
                if ui
                    .button(lock_label)
                    .on_hover_text("Lock aspect ratio")
                    .clicked()
                {
                    state.aspect_locked = !state.aspect_locked;
                }
            });

            // Scale presets
            ui.horizontal(|ui| {
                ui.label("Scale:");
                for (label, scale) in [("1x", 1.0f32), ("2x", 2.0), ("4x", 4.0)] {
                    if ui.button(label).clicked() {
                        state.settings.width = (state.original_width * scale).round() as u32;
                        state.settings.height = (state.original_height * scale).round() as u32;
                    }
                }
            });

            ui.add_space(5.0);

            // Alpha / background options
            if state.settings.format.supports_alpha() {
                ui.checkbox(&mut state.settings.include_alpha, "Transparent background");
            }

            if !state.settings.include_alpha || !state.settings.format.supports_alpha() {
                ui.horizontal(|ui| {
                    ui.label("Background:");
                    let mut color = egui::Color32::from_rgb(
                        state.settings.background_color[0],
                        state.settings.background_color[1],
                        state.settings.background_color[2],
                    );
                    if ui.color_edit_button_srgba(&mut color).changed() {
                        state.settings.background_color = [color.r(), color.g(), color.b()];
                    }
                });
            }

            // JPEG quality
            if state.settings.format == ExportFormat::Jpeg {
                ui.horizontal(|ui| {
                    ui.label("Quality:");
                    let mut quality = state.settings.jpeg_quality as i32;
                    ui.add(egui::Slider::new(&mut quality, 1..=100));
                    state.settings.jpeg_quality = quality as u8;
                });
            }

            ui.add_space(10.0);

            // Buttons
            ui.horizontal(|ui| {
                if ui.button("Export").clicked() {
                    state.result = ExportDialogResult::Export;
                    state.open = false;
                }
                if ui.button("Cancel").clicked() {
                    state.result = ExportDialogResult::Cancel;
                    state.open = false;
                }
            });
        });

    if !open {
        state.open = false;
    }
}
