use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::Instant;

use tiny_skia::Pixmap;

use crate::clipboard;
use crate::export;
use crate::file_navigator::FileNavigator;
use crate::renderer::{Renderer, MAX_RENDER_SCALE};
use crate::svg_document::SvgDocument;
use crate::ui::canvas;
use crate::ui::export_dialog::{self, ExportDialogResult, ExportDialogState};
use crate::ui::shortcuts;
use crate::ui::status_bar;
use crate::ui::toolbar::{self, ToolbarAction};
use crate::viewport::Viewport;

struct PendingLoad {
    receiver: mpsc::Receiver<Result<LoadedFile, String>>,
}

struct LoadedFile {
    doc: SvgDocument,
    pixmap: Pixmap,
    viewport: Viewport,
    logical_display_w: f32,
    logical_display_h: f32,
}

pub struct SvgViewerApp {
    document: Option<SvgDocument>,
    viewport: Viewport,
    renderer: Renderer,
    navigator: FileNavigator,

    show_checkerboard: bool,
    dark_mode: bool,
    error_message: Option<String>,
    status_message: Option<String>,

    export_dialog: ExportDialogState,
    render_dirty: bool,
    last_area_size: (f32, f32),

    // Deferred zoom re-render
    zoom_idle_since: Option<Instant>,
    pending_rerender: bool,

    // Initial file to load
    initial_file: Option<PathBuf>,

    // Background loading
    pending_load: Option<PendingLoad>,
    last_pixels_per_point: f32,

    // Cap initial zoom to MAX_RENDER_SCALE (cleared after first auto-fit)
    cap_initial_zoom: bool,
}

impl SvgViewerApp {
    pub fn new(file_path: Option<PathBuf>) -> Self {
        Self {
            document: None,
            viewport: Viewport::default(),
            renderer: Renderer::new(),
            navigator: FileNavigator::new(),
            show_checkerboard: true,
            dark_mode: true,
            error_message: None,
            status_message: None,
            export_dialog: ExportDialogState::new(),
            render_dirty: true,
            last_area_size: (0.0, 0.0),
            zoom_idle_since: None,
            pending_rerender: false,
            initial_file: file_path,
            pending_load: None,
            last_pixels_per_point: 0.0,
            cap_initial_zoom: true,
        }
    }

    fn load_file(&mut self, path: &Path) {
        self.error_message = None;
        self.status_message = None;
        self.navigator.scan_directory(path);

        if self.last_pixels_per_point > 0.0 && self.last_area_size.0 > 0.0 {
            self.start_background_load(path);
        } else {
            // First frame: area size unknown, load synchronously
            match SvgDocument::load(path) {
                Ok(doc) => {
                    self.viewport.reset();
                    self.document = Some(doc);
                    self.render_dirty = true;
                    self.cap_initial_zoom = true;
                }
                Err(e) => {
                    self.error_message = Some(format!("Error: {}", e));
                    log::error!("Failed to load {}: {}", path.display(), e);
                }
            }
        }
    }

    fn open_file_dialog(&mut self) {
        let file = rfd::FileDialog::new()
            .add_filter("SVG Files", &["svg", "svgz"])
            .add_filter("All Files", &["*"])
            .pick_file();

        if let Some(path) = file {
            self.load_file(&path);
        }
    }

    fn navigate_prev(&mut self) {
        if let Some(path) = self.navigator.prev().map(|p| p.to_path_buf()) {
            self.load_file_keep_navigator(&path);
        }
    }

    fn navigate_next(&mut self) {
        if let Some(path) = self.navigator.next().map(|p| p.to_path_buf()) {
            self.load_file_keep_navigator(&path);
        }
    }

    fn load_file_keep_navigator(&mut self, path: &Path) {
        self.error_message = None;
        self.start_background_load(path);
    }

    fn start_background_load(&mut self, path: &Path) {
        let path = path.to_path_buf();
        let (area_w, area_h) = self.last_area_size;
        let ppp = self.last_pixels_per_point;
        let (tx, rx) = mpsc::channel();

        std::thread::spawn(move || {
            let result = (|| -> Result<LoadedFile, String> {
                let doc = SvgDocument::load(&path).map_err(|e| format!("{e}"))?;
                let mut viewport = Viewport::default();
                if area_w > 0.0 && area_h > 0.0 {
                    viewport.fit_to_area(doc.width, doc.height, area_w, area_h);
                    // Cap initial zoom so small SVGs don't get blown up beyond 4×
                    viewport.zoom = viewport.zoom.min(MAX_RENDER_SCALE);
                }
                let pixmap = Renderer::render_to_pixmap(&doc, &viewport, area_w, area_h, ppp)
                    .map_err(|e| format!("{e}"))?;
                let displayed_w = doc.width * viewport.zoom;
                let displayed_h = doc.height * viewport.zoom;
                let logical_display_w = displayed_w.min(area_w);
                let logical_display_h = displayed_h.min(area_h);
                Ok(LoadedFile {
                    doc,
                    pixmap,
                    viewport,
                    logical_display_w,
                    logical_display_h,
                })
            })();
            let _ = tx.send(result);
        });

        self.pending_load = Some(PendingLoad { receiver: rx });
    }

    fn poll_pending_load(&mut self, ctx: &egui::Context) {
        if let Some(pending) = self.pending_load.take() {
            match pending.receiver.try_recv() {
                Ok(Ok(loaded)) => {
                    self.renderer.upload_pixmap(
                        ctx,
                        &loaded.pixmap,
                        loaded.viewport.zoom,
                        loaded.logical_display_w,
                        loaded.logical_display_h,
                    );
                    self.viewport = loaded.viewport;
                    self.document = Some(loaded.doc);
                    self.render_dirty = false;
                    self.pending_rerender = false;
                }
                Ok(Err(msg)) => {
                    self.error_message = Some(format!("Error: {msg}"));
                    log::error!("Background load failed: {msg}");
                }
                Err(mpsc::TryRecvError::Empty) => {
                    // Still loading, put it back and keep polling
                    self.pending_load = Some(pending);
                    ctx.request_repaint();
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.error_message = Some("Loading failed unexpectedly".into());
                }
            }
        }
    }

    fn handle_action(&mut self, action: ToolbarAction, center: egui::Vec2) {
        if action.open_file {
            self.open_file_dialog();
        }
        if action.prev_file {
            self.navigate_prev();
        }
        if action.next_file {
            self.navigate_next();
        }
        if action.fit_to_window {
            if let Some(ref doc) = self.document {
                let (w, h) = self.last_area_size;
                self.viewport.fit_to_area(doc.width, doc.height, w, h);
                self.render_dirty = true;
            }
        }
        if action.actual_size {
            self.viewport.set_actual_size(1.0);
            self.render_dirty = true;
        }
        if action.zoom_in {
            self.viewport.zoom_in(center);
            self.schedule_rerender();
        }
        if action.zoom_out {
            self.viewport.zoom_out(center);
            self.schedule_rerender();
        }
        if action.rotate_cw {
            self.viewport.rotate_cw();
            self.render_dirty = true;
        }
        if action.rotate_ccw {
            self.viewport.rotate_ccw();
            self.render_dirty = true;
        }
        if action.mirror_h {
            self.viewport.toggle_mirror_h();
            self.render_dirty = true;
        }
        if action.mirror_v {
            self.viewport.toggle_mirror_v();
            self.render_dirty = true;
        }
        if action.export {
            if let Some(ref doc) = self.document {
                self.export_dialog
                    .open_with_dimensions(doc.width, doc.height);
            }
        }
        if action.copy_clipboard {
            self.copy_to_clipboard();
        }
        if action.toggle_bg {
            self.show_checkerboard = !self.show_checkerboard;
        }
        if action.toggle_theme {
            self.dark_mode = !self.dark_mode;
        }
        if action.reset_view {
            self.viewport.reset();
            if let Some(ref doc) = self.document {
                let (w, h) = self.last_area_size;
                self.viewport.fit_to_area(doc.width, doc.height, w, h);
            }
            self.cap_initial_zoom = true;
            self.render_dirty = true;
        }
    }

    fn copy_to_clipboard(&mut self) {
        if let Some(ref doc) = self.document {
            let width = self.renderer.rendered_width.max(doc.width as u32);
            let height = self.renderer.rendered_height.max(doc.height as u32);
            match clipboard::copy_to_clipboard(doc, &self.viewport, width, height) {
                Ok(()) => {
                    self.status_message = Some("Copied to clipboard".into());
                }
                Err(e) => {
                    self.error_message = Some(format!("Clipboard error: {}", e));
                }
            }
        }
    }

    fn do_export(&mut self) {
        let doc = match &self.document {
            Some(d) => d,
            None => return,
        };

        let settings = self.export_dialog.settings.clone();
        let default_name = format!(
            "{}.{}",
            doc.path.file_stem().unwrap_or_default().to_string_lossy(),
            settings.format.extension()
        );

        let file = rfd::FileDialog::new()
            .set_file_name(&default_name)
            .save_file();

        if let Some(path) = file {
            match export::export_svg(doc, &self.viewport, &settings, &path) {
                Ok(()) => {
                    self.status_message = Some(format!("Exported to {}", path.display()));
                }
                Err(e) => {
                    self.error_message = Some(format!("Export error: {}", e));
                }
            }
        }
    }

    fn schedule_rerender(&mut self) {
        self.zoom_idle_since = Some(Instant::now());
        self.pending_rerender = true;
    }

    fn check_deferred_rerender(&mut self) {
        if self.pending_rerender {
            if let Some(since) = self.zoom_idle_since {
                if since.elapsed().as_millis() >= 150 {
                    self.render_dirty = true;
                    self.pending_rerender = false;
                    self.zoom_idle_since = None;
                }
            }
        }
    }
}

impl eframe::App for SvgViewerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.last_pixels_per_point = ctx.pixels_per_point();

        // Load initial file on first frame
        if let Some(path) = self.initial_file.take() {
            self.load_file(&path);
        }

        // Poll for completed background loads
        self.poll_pending_load(ctx);

        // Apply theme
        if self.dark_mode {
            ctx.set_visuals(egui::Visuals::dark());
        } else {
            ctx.set_visuals(egui::Visuals::light());
        }

        // Disable egui's built-in keyboard zoom (Ctrl+/-) so it doesn't scale the whole UI
        ctx.options_mut(|o| o.zoom_with_keyboard = false);

        // Handle keyboard shortcuts
        let has_file = self.document.is_some();
        let kb_action = shortcuts::handle_shortcuts(ctx, has_file);

        // Handle dropped files
        let dropped: Vec<PathBuf> = ctx.input(|i| {
            i.raw
                .dropped_files
                .iter()
                .filter_map(|f| f.path.clone())
                .collect()
        });
        if let Some(path) = dropped.into_iter().next() {
            self.load_file(&path);
        }

        // Top toolbar
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            let tb_action = toolbar::draw_toolbar(ui, has_file);
            // Keyboard/toolbar zoom should zoom centered on the canvas (Vec2::ZERO),
            // not offset by half the area size (which would shift toward top-left).
            self.handle_action(tb_action, egui::Vec2::ZERO);
            self.handle_action(kb_action, egui::Vec2::ZERO);
        });

        // Bottom status bar
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            let position = self.navigator.position_display();
            let render_size = if self.renderer.rendered_width > 0 {
                Some((self.renderer.rendered_width, self.renderer.rendered_height))
            } else {
                None
            };
            status_bar::draw_status_bar(
                ui,
                self.document.as_ref(),
                &self.viewport,
                &position,
                self.error_message.as_deref(),
                render_size,
            );
            if self.error_message.is_none() {
                if let Some(ref msg) = self.status_message {
                    ui.label(msg);
                }
            }
        });

        // Export dialog
        export_dialog::draw_export_dialog(ctx, &mut self.export_dialog);
        if self.export_dialog.result == ExportDialogResult::Export {
            self.export_dialog.result = ExportDialogResult::None;
            self.do_export();
        } else if self.export_dialog.result == ExportDialogResult::Cancel {
            self.export_dialog.result = ExportDialogResult::None;
        }

        // Central panel - canvas
        egui::CentralPanel::default().show(ctx, |ui| {
            if self.document.is_none() {
                canvas::draw_welcome(ui);
                return;
            }

            let area = ui.available_size();
            self.last_area_size = (area.x, area.y);

            // Auto-fit on first render or when area changes significantly
            if self.render_dirty {
                if let Some(ref doc) = self.document {
                    if self.viewport.fit_mode == crate::viewport::FitMode::Fit {
                        self.viewport
                            .fit_to_area(doc.width, doc.height, area.x, area.y);
                        // Cap initial zoom so small SVGs don't get blown up beyond 4×
                        if self.cap_initial_zoom {
                            self.viewport.zoom =
                                self.viewport.zoom.min(MAX_RENDER_SCALE);
                            self.cap_initial_zoom = false;
                        }
                    }
                }
            }

            // Render SVG to texture if dirty
            if self.render_dirty {
                if let Some(ref doc) = self.document {
                    if let Err(e) =
                        self.renderer
                            .render_and_upload(ctx, doc, &self.viewport, area.x, area.y)
                    {
                        self.error_message = Some(format!("Render error: {}", e));
                    }
                    self.render_dirty = false;
                }
            }

            let bg_color = if self.dark_mode {
                egui::Color32::from_rgb(40, 40, 40)
            } else {
                egui::Color32::from_rgb(240, 240, 240)
            };

            let display_size = egui::Vec2::new(
                self.renderer.logical_display_w,
                self.renderer.logical_display_h,
            );
            let zoom_ratio = if self.renderer.rendered_zoom > 0.0 {
                self.viewport.zoom / self.renderer.rendered_zoom
            } else {
                1.0
            };

            let (response, rect) = canvas::draw_canvas(
                ui,
                self.renderer.texture.as_ref(),
                self.viewport.pan,
                self.show_checkerboard,
                bg_color,
                display_size,
                zoom_ratio,
            );

            // Handle drag to pan
            if response.dragged() {
                self.viewport.pan_by(response.drag_delta());
            }

            // Handle pinch-to-zoom (check first to avoid double-processing with scroll)
            let zoom_delta = ctx.input(|i| i.zoom_delta());
            if zoom_delta != 1.0 && response.hovered() {
                let hover_pos = ctx.input(|i| i.pointer.hover_pos().unwrap_or(rect.center()));
                let cursor_vec = hover_pos - rect.center();

                self.viewport.zoom_by(zoom_delta, cursor_vec);
                self.schedule_rerender();
                ctx.request_repaint();
            }

            // Handle scroll to zoom (skip when pinch gesture is active)
            if zoom_delta == 1.0 {
                let scroll_delta = ctx.input(|i| i.smooth_scroll_delta.y);
                if scroll_delta != 0.0 && response.hovered() {
                    let hover_pos = ctx.input(|i| i.pointer.hover_pos().unwrap_or(rect.center()));
                    let cursor_vec = hover_pos - rect.center();

                    let factor = if scroll_delta > 0.0 { 1.1 } else { 0.9 };
                    self.viewport.zoom_by(factor, cursor_vec);
                    self.schedule_rerender();
                    ctx.request_repaint();
                }
            }
        });

        // Check deferred rerender for smooth zoom
        self.check_deferred_rerender();
        if self.pending_rerender {
            ctx.request_repaint();
        }
    }
}
