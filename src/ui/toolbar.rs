use egui::Ui;

pub struct ToolbarAction {
    pub open_file: bool,
    pub prev_file: bool,
    pub next_file: bool,
    pub fit_to_window: bool,
    pub actual_size: bool,
    pub zoom_in: bool,
    pub zoom_out: bool,
    pub rotate_cw: bool,
    pub rotate_ccw: bool,
    pub mirror_h: bool,
    pub mirror_v: bool,
    pub export: bool,
    pub copy_clipboard: bool,
    pub toggle_bg: bool,
    pub toggle_theme: bool,
    pub reset_view: bool,
}

impl Default for ToolbarAction {
    fn default() -> Self {
        Self {
            open_file: false,
            prev_file: false,
            next_file: false,
            fit_to_window: false,
            actual_size: false,
            zoom_in: false,
            zoom_out: false,
            rotate_cw: false,
            rotate_ccw: false,
            mirror_h: false,
            mirror_v: false,
            export: false,
            copy_clipboard: false,
            toggle_bg: false,
            toggle_theme: false,
            reset_view: false,
        }
    }
}

pub fn draw_toolbar(ui: &mut Ui, has_file: bool) -> ToolbarAction {
    let mut action = ToolbarAction::default();

    ui.horizontal(|ui| {
        action.open_file = ui.button("Open").clicked();

        ui.separator();

        ui.add_enabled_ui(has_file, |ui| {
            action.prev_file = ui
                .button("\u{25C0}")
                .on_hover_text("Previous file")
                .clicked();
            action.next_file = ui.button("\u{25B6}").on_hover_text("Next file").clicked();
        });

        ui.separator();

        ui.add_enabled_ui(has_file, |ui| {
            action.fit_to_window = ui
                .button("Fit")
                .on_hover_text("Fit to window (Ctrl+0)")
                .clicked();
            action.actual_size = ui
                .button("1:1")
                .on_hover_text("Actual size (Ctrl+1)")
                .clicked();
        });

        ui.separator();

        ui.add_enabled_ui(has_file, |ui| {
            action.zoom_in = ui.button("+").on_hover_text("Zoom in (Ctrl++)").clicked();
            action.zoom_out = ui
                .button("\u{2212}")
                .on_hover_text("Zoom out (Ctrl+-)")
                .clicked();
        });

        ui.separator();

        ui.add_enabled_ui(has_file, |ui| {
            action.rotate_cw = ui
                .button("\u{21BB}")
                .on_hover_text("Rotate CW (R)")
                .clicked();
            action.rotate_ccw = ui
                .button("\u{21BA}")
                .on_hover_text("Rotate CCW (Shift+R)")
                .clicked();
            action.mirror_h = ui
                .button("\u{21D4}")
                .on_hover_text("Mirror H (H)")
                .clicked();
            action.mirror_v = ui
                .button("\u{21D5}")
                .on_hover_text("Mirror V (V)")
                .clicked();
        });

        ui.separator();

        ui.add_enabled_ui(has_file, |ui| {
            action.export = ui
                .button("Export")
                .on_hover_text("Export (Ctrl+Shift+E)")
                .clicked();
            action.copy_clipboard = ui
                .button("Copy")
                .on_hover_text("Copy to clipboard (Ctrl+C)")
                .clicked();
        });

        ui.separator();

        action.toggle_bg = ui
            .button("BG")
            .on_hover_text("Toggle background (T)")
            .clicked();
        action.toggle_theme = ui
            .button("Theme")
            .on_hover_text("Toggle dark/light theme")
            .clicked();

        ui.separator();

        ui.add_enabled_ui(has_file, |ui| {
            action.reset_view = ui
                .button("Reset")
                .on_hover_text("Reset view (Ctrl+R)")
                .clicked();
        });
    });

    action
}
