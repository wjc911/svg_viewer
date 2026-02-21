use egui::{Color32, Rect, Sense, TextureHandle, Ui, Vec2};

const CHECKER_SIZE: f32 = 10.0;
const CHECKER_LIGHT: Color32 = Color32::from_rgb(204, 204, 204);
const CHECKER_DARK: Color32 = Color32::from_rgb(170, 170, 170);

pub fn draw_canvas(
    ui: &mut Ui,
    texture: Option<&TextureHandle>,
    pan: Vec2,
    show_checkerboard: bool,
    bg_color: Color32,
    display_size: Vec2,
    zoom_ratio: f32,
) -> (egui::Response, Rect) {
    let available = ui.available_size();
    let (response, mut painter) = ui.allocate_painter(available, Sense::click_and_drag());
    let rect = response.rect;

    // Draw background
    if show_checkerboard {
        draw_checkerboard(&painter, rect);
    } else {
        painter.rect_filled(rect, 0.0, bg_color);
    }

    // Draw the SVG texture
    if let Some(tex) = texture {
        let img_size = display_size * zoom_ratio;
        let center = rect.center().to_vec2() + pan;
        let img_rect = Rect::from_center_size(center.to_pos2(), img_size);

        // Clip to canvas area
        painter.set_clip_rect(rect);

        painter.image(
            tex.id(),
            img_rect,
            Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            Color32::WHITE,
        );
    }

    (response, rect)
}

fn draw_checkerboard(painter: &egui::Painter, rect: Rect) {
    // Fill with light color first
    painter.rect_filled(rect, 0.0, CHECKER_LIGHT);

    // Draw dark squares
    let start_x = rect.left();
    let start_y = rect.top();
    let end_x = rect.right();
    let end_y = rect.bottom();

    let mut y = start_y;
    let mut row = 0;
    while y < end_y {
        let mut x = start_x + if row % 2 == 1 { CHECKER_SIZE } else { 0.0 };
        while x < end_x {
            let sq_rect = Rect::from_min_size(
                egui::pos2(x, y),
                Vec2::new(CHECKER_SIZE.min(end_x - x), CHECKER_SIZE.min(end_y - y)),
            );
            painter.rect_filled(sq_rect, 0.0, CHECKER_DARK);
            x += CHECKER_SIZE * 2.0;
        }
        y += CHECKER_SIZE;
        row += 1;
    }
}

pub fn draw_welcome(ui: &mut Ui) {
    ui.centered_and_justified(|ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(ui.available_height() / 3.0);
            ui.heading("SVG Viewer");
            ui.add_space(10.0);
            ui.label("Open a file or drag & drop an SVG here");
            ui.add_space(5.0);
            ui.label("Ctrl+O to open  |  Arrow keys to browse");
        });
    });
}
