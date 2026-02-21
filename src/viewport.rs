use egui::Vec2;

#[derive(Clone, Debug, PartialEq)]
pub enum FitMode {
    Fit,
    ActualSize,
    Custom,
}

pub struct Viewport {
    pub zoom: f32,
    pub pan: Vec2,
    pub rotation_deg: f32,
    pub mirror_h: bool,
    pub mirror_v: bool,
    pub fit_mode: FitMode,
}

impl Default for Viewport {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            pan: Vec2::ZERO,
            rotation_deg: 0.0,
            mirror_h: false,
            mirror_v: false,
            fit_mode: FitMode::Fit,
        }
    }
}

impl Viewport {
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    pub fn fit_to_area(
        &mut self,
        svg_width: f32,
        svg_height: f32,
        area_width: f32,
        area_height: f32,
    ) {
        if svg_width <= 0.0 || svg_height <= 0.0 || area_width <= 0.0 || area_height <= 0.0 {
            return;
        }

        let (effective_w, effective_h) = if (self.rotation_deg % 180.0).abs() > 45.0 {
            (svg_height, svg_width)
        } else {
            (svg_width, svg_height)
        };

        let scale_x = area_width / effective_w;
        let scale_y = area_height / effective_h;
        self.zoom = scale_x.min(scale_y);
        self.pan = Vec2::ZERO;
        self.fit_mode = FitMode::Fit;
    }

    pub fn set_actual_size(&mut self, pixels_per_point: f32) {
        self.zoom = 1.0 / pixels_per_point;
        self.pan = Vec2::ZERO;
        self.fit_mode = FitMode::ActualSize;
    }

    pub fn zoom_by(&mut self, factor: f32, cursor_pos: Vec2) {
        let old_zoom = self.zoom;
        self.zoom = (self.zoom * factor).clamp(0.01, 100.0);
        let scale_ratio = self.zoom / old_zoom;
        self.pan = cursor_pos - scale_ratio * (cursor_pos - self.pan);
        self.fit_mode = FitMode::Custom;
    }

    pub fn zoom_in(&mut self, center: Vec2) {
        self.zoom_by(1.25, center);
    }

    pub fn zoom_out(&mut self, center: Vec2) {
        self.zoom_by(0.8, center);
    }

    pub fn pan_by(&mut self, delta: Vec2) {
        self.pan += delta;
        if self.fit_mode == FitMode::Fit {
            self.fit_mode = FitMode::Custom;
        }
    }

    pub fn rotate_cw(&mut self) {
        self.rotation_deg = (self.rotation_deg + 90.0) % 360.0;
    }

    pub fn rotate_ccw(&mut self) {
        self.rotation_deg = (self.rotation_deg - 90.0 + 360.0) % 360.0;
    }

    pub fn toggle_mirror_h(&mut self) {
        self.mirror_h = !self.mirror_h;
    }

    pub fn toggle_mirror_v(&mut self) {
        self.mirror_v = !self.mirror_v;
    }

    /// Build a usvg::Transform for the current viewport state.
    /// `render_width` and `render_height` are the pixmap dimensions.
    pub fn build_transform(
        &self,
        svg_width: f32,
        svg_height: f32,
        render_width: f32,
        render_height: f32,
    ) -> tiny_skia::Transform {
        let cx = render_width / 2.0;
        let cy = render_height / 2.0;

        let scale_x = render_width / svg_width;
        let scale_y = render_height / svg_height;
        let scale = scale_x.min(scale_y);

        let mut ts = tiny_skia::Transform::identity();
        // Move to center
        ts = ts.post_translate(cx, cy);
        // Apply rotation
        if self.rotation_deg != 0.0 {
            ts = ts.pre_rotate(self.rotation_deg);
        }
        // Apply mirror
        if self.mirror_h {
            ts = ts.pre_scale(-1.0, 1.0);
        }
        if self.mirror_v {
            ts = ts.pre_scale(1.0, -1.0);
        }
        // Move back and apply scale
        ts = ts.pre_translate(-svg_width / 2.0 * scale, -svg_height / 2.0 * scale);
        ts = ts.pre_scale(scale, scale);

        ts
    }

    pub fn zoom_percent(&self) -> f32 {
        self.zoom * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_viewport() {
        let vp = Viewport::default();
        assert_eq!(vp.zoom, 1.0);
        assert_eq!(vp.pan, Vec2::ZERO);
        assert_eq!(vp.rotation_deg, 0.0);
        assert!(!vp.mirror_h);
        assert!(!vp.mirror_v);
        assert_eq!(vp.fit_mode, FitMode::Fit);
    }

    #[test]
    fn test_fit_to_area() {
        let mut vp = Viewport::default();
        // SVG is 200x100, area is 400x400 -> scale by 2.0
        vp.fit_to_area(200.0, 100.0, 400.0, 400.0);
        assert_eq!(vp.zoom, 2.0);
        assert_eq!(vp.fit_mode, FitMode::Fit);
    }

    #[test]
    fn test_fit_to_area_wider() {
        let mut vp = Viewport::default();
        // SVG is 200x100, area is 100x200 -> scale by 0.5
        vp.fit_to_area(200.0, 100.0, 100.0, 200.0);
        assert_eq!(vp.zoom, 0.5);
    }

    #[test]
    fn test_fit_to_area_zero_dimensions() {
        let mut vp = Viewport::default();
        vp.zoom = 2.0;
        vp.fit_to_area(0.0, 100.0, 400.0, 400.0);
        assert_eq!(vp.zoom, 2.0); // Unchanged
    }

    #[test]
    fn test_zoom_clamp() {
        let mut vp = Viewport::default();
        vp.zoom = 0.02;
        vp.zoom_by(0.1, Vec2::ZERO); // Would go to 0.002, clamped to 0.01
        assert_eq!(vp.zoom, 0.01);

        vp.zoom = 90.0;
        vp.zoom_by(2.0, Vec2::ZERO); // Would go to 180, clamped to 100
        assert_eq!(vp.zoom, 100.0);
    }

    #[test]
    fn test_rotate_cw() {
        let mut vp = Viewport::default();
        vp.rotate_cw();
        assert_eq!(vp.rotation_deg, 90.0);
        vp.rotate_cw();
        assert_eq!(vp.rotation_deg, 180.0);
        vp.rotate_cw();
        assert_eq!(vp.rotation_deg, 270.0);
        vp.rotate_cw();
        assert_eq!(vp.rotation_deg, 0.0);
    }

    #[test]
    fn test_rotate_ccw() {
        let mut vp = Viewport::default();
        vp.rotate_ccw();
        assert_eq!(vp.rotation_deg, 270.0);
        vp.rotate_ccw();
        assert_eq!(vp.rotation_deg, 180.0);
    }

    #[test]
    fn test_mirror_toggle() {
        let mut vp = Viewport::default();
        assert!(!vp.mirror_h);
        vp.toggle_mirror_h();
        assert!(vp.mirror_h);
        vp.toggle_mirror_h();
        assert!(!vp.mirror_h);
    }

    #[test]
    fn test_pan_changes_fit_mode() {
        let mut vp = Viewport::default();
        assert_eq!(vp.fit_mode, FitMode::Fit);
        vp.pan_by(Vec2::new(10.0, 5.0));
        assert_eq!(vp.fit_mode, FitMode::Custom);
        assert_eq!(vp.pan, Vec2::new(10.0, 5.0));
    }

    #[test]
    fn test_reset() {
        let mut vp = Viewport::default();
        vp.zoom = 3.0;
        vp.pan = Vec2::new(100.0, 200.0);
        vp.rotation_deg = 90.0;
        vp.mirror_h = true;
        vp.reset();
        assert_eq!(vp.zoom, 1.0);
        assert_eq!(vp.pan, Vec2::ZERO);
        assert_eq!(vp.rotation_deg, 0.0);
        assert!(!vp.mirror_h);
    }

    #[test]
    fn test_zoom_percent() {
        let mut vp = Viewport::default();
        vp.zoom = 1.5;
        assert_eq!(vp.zoom_percent(), 150.0);
    }
}
