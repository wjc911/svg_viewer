//! Profiling tool: measures time spent in each stage of SVG loading and rendering.
//!
//! Usage: cargo run --release --bin profile_render [SVG_FILE]

use std::path::PathBuf;
use std::time::Instant;

fn main() {
    let path = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/test_fixtures/simple_rect.svg")
        });

    println!("=== SVG Render Profiling ===");
    println!("File: {}", path.display());

    let t = Instant::now();
    let raw_data = std::fs::read(&path).expect("Failed to read file");
    let read_ms = t.elapsed().as_secs_f64() * 1000.0;
    println!("File size: {} bytes", raw_data.len());
    println!("[read]  {:>8.3} ms", read_ms);

    let t = Instant::now();
    let tree =
        usvg::Tree::from_data(&raw_data, &usvg::Options::default()).expect("Failed to parse SVG");
    let parse_ms = t.elapsed().as_secs_f64() * 1000.0;
    let svg_w = tree.size().width();
    let svg_h = tree.size().height();
    println!("[parse] {:>8.3} ms  ({}x{})", parse_ms, svg_w, svg_h);

    // Test at various render resolutions
    for &(label, rw, rh) in &[
        ("native 1x", svg_w as u32, svg_h as u32),
        ("native 2x", svg_w as u32 * 2, svg_h as u32 * 2),
        ("native 4x", svg_w as u32 * 4, svg_h as u32 * 4),
        (
            "720p fit",
            {
                let z = (720.0 / svg_w).min(720.0 / svg_h);
                ((svg_w * z) as u32, (svg_h * z) as u32)
            }
            .0,
            {
                let z = (720.0 / svg_w).min(720.0 / svg_h);
                ((svg_w * z) as u32, (svg_h * z) as u32)
            }
            .1,
        ),
        (
            "1080p fit",
            {
                let z = (1080.0 / svg_w).min(1080.0 / svg_h);
                ((svg_w * z) as u32, (svg_h * z) as u32)
            }
            .0,
            {
                let z = (1080.0 / svg_w).min(1080.0 / svg_h);
                ((svg_w * z) as u32, (svg_h * z) as u32)
            }
            .1,
        ),
        (
            "1620p HiDPI",
            {
                let z = (1080.0 / svg_w).min(1080.0 / svg_h) * 1.5;
                ((svg_w * z) as u32, (svg_h * z) as u32)
            }
            .0,
            {
                let z = (1080.0 / svg_w).min(1080.0 / svg_h) * 1.5;
                ((svg_w * z) as u32, (svg_h * z) as u32)
            }
            .1,
        ),
    ] {
        let rw = rw.clamp(1, 4096);
        let rh = rh.clamp(1, 4096);
        let mut pixmap = tiny_skia::Pixmap::new(rw, rh).unwrap();
        let sx = rw as f32 / svg_w;
        let sy = rh as f32 / svg_h;
        let s = sx.min(sy);
        let t = Instant::now();
        resvg::render(
            &tree,
            tiny_skia::Transform::from_scale(s, s),
            &mut pixmap.as_mut(),
        );
        let ms = t.elapsed().as_secs_f64() * 1000.0;
        println!("[render {label:>12}] {rw:>4}x{rh:<4} {:>8.3} ms", ms);
    }
}
