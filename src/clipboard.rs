use arboard::{Clipboard, ImageData};

use crate::error::{Result, SvgError};
use crate::export::pixmap_to_rgba;
use crate::renderer::Renderer;
use crate::svg_document::SvgDocument;
use crate::viewport::Viewport;

pub fn copy_to_clipboard(
    doc: &SvgDocument,
    viewport: &Viewport,
    width: u32,
    height: u32,
) -> Result<()> {
    let pixmap = Renderer::render_for_export(doc, width, height, viewport)?;
    let rgba = pixmap_to_rgba(&pixmap);

    let img_data = ImageData {
        width: pixmap.width() as usize,
        height: pixmap.height() as usize,
        bytes: rgba.into(),
    };

    let mut clipboard = Clipboard::new().map_err(|e| SvgError::Clipboard(e.to_string()))?;
    clipboard
        .set_image(img_data)
        .map_err(|e| SvgError::Clipboard(e.to_string()))?;

    Ok(())
}
