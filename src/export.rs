use std::path::Path;
use tiny_skia::Pixmap;

use crate::error::{Result, SvgError};
use crate::renderer::Renderer;
use crate::svg_document::SvgDocument;
use crate::viewport::Viewport;

#[derive(Clone, Debug, PartialEq)]
pub enum ExportFormat {
    Png,
    Jpeg,
    Bmp,
    Tiff,
    WebP,
}

impl ExportFormat {
    pub fn extension(&self) -> &str {
        match self {
            ExportFormat::Png => "png",
            ExportFormat::Jpeg => "jpg",
            ExportFormat::Bmp => "bmp",
            ExportFormat::Tiff => "tiff",
            ExportFormat::WebP => "webp",
        }
    }

    pub fn supports_alpha(&self) -> bool {
        matches!(
            self,
            ExportFormat::Png | ExportFormat::Tiff | ExportFormat::WebP
        )
    }

    pub fn all() -> &'static [ExportFormat] {
        &[
            ExportFormat::Png,
            ExportFormat::Jpeg,
            ExportFormat::Bmp,
            ExportFormat::Tiff,
            ExportFormat::WebP,
        ]
    }

    pub fn name(&self) -> &str {
        match self {
            ExportFormat::Png => "PNG",
            ExportFormat::Jpeg => "JPEG",
            ExportFormat::Bmp => "BMP",
            ExportFormat::Tiff => "TIFF",
            ExportFormat::WebP => "WebP",
        }
    }
}

#[derive(Clone)]
pub struct ExportSettings {
    pub format: ExportFormat,
    pub width: u32,
    pub height: u32,
    pub include_alpha: bool,
    pub jpeg_quality: u8,
    pub background_color: [u8; 3],
}

impl Default for ExportSettings {
    fn default() -> Self {
        Self {
            format: ExportFormat::Png,
            width: 800,
            height: 600,
            include_alpha: true,
            jpeg_quality: 90,
            background_color: [255, 255, 255],
        }
    }
}

/// Un-premultiply alpha from premultiplied RGBA pixel data.
fn un_premultiply_alpha(data: &[u8]) -> Vec<u8> {
    let mut result = Vec::with_capacity(data.len());
    for chunk in data.chunks_exact(4) {
        let a = chunk[3] as f32 / 255.0;
        if a > 0.0 {
            result.push((chunk[0] as f32 / a).round().min(255.0) as u8);
            result.push((chunk[1] as f32 / a).round().min(255.0) as u8);
            result.push((chunk[2] as f32 / a).round().min(255.0) as u8);
            result.push(chunk[3]);
        } else {
            result.extend_from_slice(&[0, 0, 0, 0]);
        }
    }
    result
}

/// Composite premultiplied RGBA over a solid background color, producing RGB.
fn composite_over_background(data: &[u8], bg: [u8; 3]) -> Vec<u8> {
    let mut result = Vec::with_capacity((data.len() / 4) * 3);
    for chunk in data.chunks_exact(4) {
        let a = chunk[3] as f32 / 255.0;
        // data is premultiplied, so: final = premul_color + bg * (1 - a)
        let r = (chunk[0] as f32 + bg[0] as f32 * (1.0 - a))
            .round()
            .min(255.0) as u8;
        let g = (chunk[1] as f32 + bg[1] as f32 * (1.0 - a))
            .round()
            .min(255.0) as u8;
        let b = (chunk[2] as f32 + bg[2] as f32 * (1.0 - a))
            .round()
            .min(255.0) as u8;
        result.push(r);
        result.push(g);
        result.push(b);
    }
    result
}

pub fn export_svg(
    doc: &SvgDocument,
    viewport: &Viewport,
    settings: &ExportSettings,
    output_path: &Path,
) -> Result<()> {
    let pixmap = Renderer::render_for_export(doc, settings.width, settings.height, viewport)?;
    save_pixmap(&pixmap, settings, output_path)
}

pub fn save_pixmap(pixmap: &Pixmap, settings: &ExportSettings, output_path: &Path) -> Result<()> {
    let width = pixmap.width();
    let height = pixmap.height();
    let data = pixmap.data();

    match settings.format {
        ExportFormat::Png if settings.include_alpha => {
            let rgba = un_premultiply_alpha(data);
            let img = image::RgbaImage::from_raw(width, height, rgba)
                .ok_or_else(|| SvgError::Export("Failed to create RGBA image".into()))?;
            img.save(output_path)
                .map_err(|e| SvgError::Export(e.to_string()))?;
        }
        ExportFormat::Tiff if settings.include_alpha => {
            let rgba = un_premultiply_alpha(data);
            let img = image::RgbaImage::from_raw(width, height, rgba)
                .ok_or_else(|| SvgError::Export("Failed to create RGBA image".into()))?;
            img.save(output_path)
                .map_err(|e| SvgError::Export(e.to_string()))?;
        }
        ExportFormat::WebP if settings.include_alpha => {
            let rgba = un_premultiply_alpha(data);
            let img = image::RgbaImage::from_raw(width, height, rgba)
                .ok_or_else(|| SvgError::Export("Failed to create RGBA image".into()))?;
            img.save(output_path)
                .map_err(|e| SvgError::Export(e.to_string()))?;
        }
        ExportFormat::Jpeg => {
            let rgb = composite_over_background(data, settings.background_color);
            let img = image::RgbImage::from_raw(width, height, rgb)
                .ok_or_else(|| SvgError::Export("Failed to create RGB image".into()))?;
            // For quality control, use the jpeg encoder directly
            let file = std::fs::File::create(output_path)?;
            let mut buf_writer = std::io::BufWriter::new(file);
            let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(
                &mut buf_writer,
                settings.jpeg_quality,
            );
            image::ImageEncoder::write_image(
                encoder,
                &img,
                width,
                height,
                image::ExtendedColorType::Rgb8,
            )
            .map_err(|e| SvgError::Export(e.to_string()))?;
        }
        _ => {
            // Formats without alpha support or alpha disabled: composite over background
            let rgb = composite_over_background(data, settings.background_color);
            let img = image::RgbImage::from_raw(width, height, rgb)
                .ok_or_else(|| SvgError::Export("Failed to create RGB image".into()))?;
            img.save(output_path)
                .map_err(|e| SvgError::Export(e.to_string()))?;
        }
    }

    Ok(())
}

/// Get pixmap data as un-premultiplied RGBA bytes (for clipboard).
pub fn pixmap_to_rgba(pixmap: &Pixmap) -> Vec<u8> {
    un_premultiply_alpha(pixmap.data())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn fixture_path(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("assets")
            .join("test_fixtures")
            .join(name)
    }

    #[test]
    fn test_export_format_extensions() {
        assert_eq!(ExportFormat::Png.extension(), "png");
        assert_eq!(ExportFormat::Jpeg.extension(), "jpg");
        assert_eq!(ExportFormat::Bmp.extension(), "bmp");
        assert_eq!(ExportFormat::Tiff.extension(), "tiff");
        assert_eq!(ExportFormat::WebP.extension(), "webp");
    }

    #[test]
    fn test_alpha_support() {
        assert!(ExportFormat::Png.supports_alpha());
        assert!(!ExportFormat::Jpeg.supports_alpha());
        assert!(!ExportFormat::Bmp.supports_alpha());
        assert!(ExportFormat::Tiff.supports_alpha());
        assert!(ExportFormat::WebP.supports_alpha());
    }

    #[test]
    fn test_un_premultiply_alpha() {
        // Fully opaque red pixel (premultiplied)
        let data = vec![255, 0, 0, 255];
        let result = un_premultiply_alpha(&data);
        assert_eq!(result, vec![255, 0, 0, 255]);

        // Half-transparent red (premultiplied: r=128 means r_actual=255 at a=128)
        let data = vec![128, 0, 0, 128];
        let result = un_premultiply_alpha(&data);
        // 128 / (128/255) ≈ 255
        assert_eq!(result[0], 255); // red
        assert_eq!(result[3], 128); // alpha preserved
    }

    #[test]
    fn test_un_premultiply_zero_alpha() {
        let data = vec![0, 0, 0, 0];
        let result = un_premultiply_alpha(&data);
        assert_eq!(result, vec![0, 0, 0, 0]);
    }

    #[test]
    fn test_composite_over_background() {
        // Fully opaque red pixel over white background
        let data = vec![255, 0, 0, 255];
        let result = composite_over_background(&data, [255, 255, 255]);
        assert_eq!(result, vec![255, 0, 0]);

        // Fully transparent pixel over white background
        let data = vec![0, 0, 0, 0];
        let result = composite_over_background(&data, [255, 255, 255]);
        assert_eq!(result, vec![255, 255, 255]);
    }

    #[test]
    fn test_export_png() {
        let doc = crate::svg_document::SvgDocument::load(&fixture_path("simple_rect.svg")).unwrap();
        let viewport = crate::viewport::Viewport::default();
        let settings = ExportSettings {
            format: ExportFormat::Png,
            width: 100,
            height: 75,
            include_alpha: true,
            ..Default::default()
        };
        let output = std::env::temp_dir().join("svg_viewer_test_export.png");
        export_svg(&doc, &viewport, &settings, &output).unwrap();
        assert!(output.exists());
        let metadata = std::fs::metadata(&output).unwrap();
        assert!(metadata.len() > 0);
        std::fs::remove_file(&output).ok();
    }

    #[test]
    fn test_export_jpeg() {
        let doc = crate::svg_document::SvgDocument::load(&fixture_path("simple_rect.svg")).unwrap();
        let viewport = crate::viewport::Viewport::default();
        let settings = ExportSettings {
            format: ExportFormat::Jpeg,
            width: 100,
            height: 75,
            include_alpha: false,
            jpeg_quality: 80,
            ..Default::default()
        };
        let output = std::env::temp_dir().join("svg_viewer_test_export.jpg");
        export_svg(&doc, &viewport, &settings, &output).unwrap();
        assert!(output.exists());
        std::fs::remove_file(&output).ok();
    }
}
