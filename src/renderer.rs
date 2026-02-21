use egui::{ColorImage, TextureHandle, TextureOptions};
use tiny_skia::Pixmap;

use crate::error::{Result, SvgError};
use crate::svg_document::SvgDocument;
use crate::viewport::Viewport;

const MAX_RENDER_DIM: u32 = 4096;

pub struct Renderer {
    pub texture: Option<TextureHandle>,
    pub rendered_width: u32,
    pub rendered_height: u32,
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            texture: None,
            rendered_width: 0,
            rendered_height: 0,
        }
    }

    /// Render the SVG document at the given viewport zoom level and return a pixmap.
    pub fn render_to_pixmap(
        doc: &SvgDocument,
        viewport: &Viewport,
        area_width: f32,
        area_height: f32,
        pixels_per_point: f32,
    ) -> Result<Pixmap> {
        let svg_w = doc.width;
        let svg_h = doc.height;

        if svg_w <= 0.0 || svg_h <= 0.0 {
            return Err(SvgError::Render("SVG has zero dimensions".into()));
        }

        let (effective_svg_w, effective_svg_h) = if (viewport.rotation_deg % 180.0).abs() > 45.0 {
            (svg_h, svg_w)
        } else {
            (svg_w, svg_h)
        };

        // Calculate the displayed size of the SVG on screen (in logical pixels)
        // zoom represents how many screen pixels per SVG unit
        let displayed_w = effective_svg_w * viewport.zoom;
        let displayed_h = effective_svg_h * viewport.zoom;

        // Cap to the available area so we don't render more than what's visible
        let capped_w = displayed_w.min(area_width);
        let capped_h = displayed_h.min(area_height);

        // Convert to physical pixels
        let render_w = (capped_w * pixels_per_point).round() as u32;
        let render_h = (capped_h * pixels_per_point).round() as u32;

        // Clamp to safe maximum
        let render_w = render_w.clamp(1, MAX_RENDER_DIM);
        let render_h = render_h.clamp(1, MAX_RENDER_DIM);

        let mut pixmap = Pixmap::new(render_w, render_h)
            .ok_or_else(|| SvgError::Render("Failed to create pixmap".into()))?;

        let transform = viewport.build_transform(svg_w, svg_h, render_w as f32, render_h as f32);
        resvg::render(&doc.tree, transform, &mut pixmap.as_mut());

        Ok(pixmap)
    }

    /// Render SVG and upload as a GPU texture.
    pub fn render_and_upload(
        &mut self,
        ctx: &egui::Context,
        doc: &SvgDocument,
        viewport: &Viewport,
        area_width: f32,
        area_height: f32,
    ) -> Result<()> {
        let pixels_per_point = ctx.pixels_per_point();
        let pixmap =
            Self::render_to_pixmap(doc, viewport, area_width, area_height, pixels_per_point)?;

        let width = pixmap.width() as usize;
        let height = pixmap.height() as usize;

        let image = ColorImage::from_rgba_premultiplied([width, height], pixmap.data());

        let options = TextureOptions {
            magnification: egui::TextureFilter::Linear,
            minification: egui::TextureFilter::Linear,
            ..Default::default()
        };

        match &mut self.texture {
            Some(handle) => handle.set(image, options),
            None => {
                self.texture = Some(ctx.load_texture("svg_render", image, options));
            }
        }

        self.rendered_width = width as u32;
        self.rendered_height = height as u32;

        Ok(())
    }

    /// Render an SVG at a specific resolution for export (no viewport transforms).
    pub fn render_for_export(
        doc: &SvgDocument,
        width: u32,
        height: u32,
        viewport: &Viewport,
    ) -> Result<Pixmap> {
        let width = width.clamp(1, MAX_RENDER_DIM);
        let height = height.clamp(1, MAX_RENDER_DIM);

        let mut pixmap = Pixmap::new(width, height)
            .ok_or_else(|| SvgError::Render("Failed to create pixmap".into()))?;

        let transform =
            viewport.build_transform(doc.width, doc.height, width as f32, height as f32);
        resvg::render(&doc.tree, transform, &mut pixmap.as_mut());

        Ok(pixmap)
    }
}
