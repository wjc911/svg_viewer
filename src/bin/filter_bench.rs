//! Benchmark: compare resvg's feMorphology cost vs. a manual implementation
//! to determine if resvg's filter pipeline has performance issues.

use std::time::Instant;

fn main() {
    let size: usize = 1620;
    let pixels = size * size;
    println!("=== Filter Benchmark ({size}x{size} = {pixels} pixels) ===\n");

    // --- Manual dilate (radius=1): what it SHOULD cost ---
    // Naive 3x3 max filter on a single channel
    let src: Vec<u8> = (0..pixels).map(|i| (i % 256) as u8).collect();
    let mut dst: Vec<u8> = vec![0; pixels];

    let t = Instant::now();
    for y in 0..size {
        for x in 0..size {
            let mut max_val: u8 = 0;
            for dy in -1i32..=1 {
                for dx in -1i32..=1 {
                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;
                    if nx >= 0 && nx < size as i32 && ny >= 0 && ny < size as i32 {
                        let v = src[ny as usize * size + nx as usize];
                        if v > max_val {
                            max_val = v;
                        }
                    }
                }
            }
            dst[y * size + x] = max_val;
        }
    }
    let manual_1ch = t.elapsed().as_secs_f64() * 1000.0;
    println!("[manual dilate 1ch]    {:>8.3} ms", manual_1ch);

    // 4 channels (RGBA) = 4x
    let t = Instant::now();
    let src4: Vec<[u8; 4]> = (0..pixels).map(|i| [(i % 256) as u8; 4]).collect();
    let mut dst4: Vec<[u8; 4]> = vec![[0; 4]; pixels];
    for y in 0..size {
        for x in 0..size {
            let mut max_val = [0u8; 4];
            for dy in -1i32..=1 {
                for dx in -1i32..=1 {
                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;
                    if nx >= 0 && nx < size as i32 && ny >= 0 && ny < size as i32 {
                        let v = src4[ny as usize * size + nx as usize];
                        for c in 0..4 {
                            if v[c] > max_val[c] {
                                max_val[c] = v[c];
                            }
                        }
                    }
                }
            }
            dst4[y * size + x] = max_val;
        }
    }
    let manual_4ch = t.elapsed().as_secs_f64() * 1000.0;
    println!("[manual dilate RGBA]   {:>8.3} ms", manual_4ch);

    // Simulate full filter chain cost: dilate + flood + composite + merge
    // Each step is ~1 pass over the buffer
    let t = Instant::now();
    let mut buf_a: Vec<[u8; 4]> = vec![[128; 4]; pixels]; // dilated
    let buf_b: Vec<[u8; 4]> = vec![[0, 0, 0, 255]; pixels]; // flood (black)
    let mut buf_c: Vec<[u8; 4]> = vec![[0; 4]; pixels]; // composite result
    // composite "in": flood color masked by dilated alpha
    for i in 0..pixels {
        let a = buf_a[i][3];
        buf_c[i] = [
            ((buf_b[i][0] as u16 * a as u16) / 255) as u8,
            ((buf_b[i][1] as u16 * a as u16) / 255) as u8,
            ((buf_b[i][2] as u16 * a as u16) / 255) as u8,
            a,
        ];
    }
    // merge: composite buf_c under buf_a (source-over)
    for i in 0..pixels {
        let sa = buf_a[i][3] as u16;
        let da = buf_c[i][3] as u16;
        let out_a = sa + da * (255 - sa) / 255;
        buf_a[i][3] = out_a as u8;
    }
    let chain_extra = t.elapsed().as_secs_f64() * 1000.0;
    println!("[flood+composite+merge]{:>8.3} ms", chain_extra);

    let theoretical_total = manual_4ch + chain_extra;
    println!("\n[theoretical total]    {:>8.3} ms", theoretical_total);

    // --- resvg actual: render the filter SVG ---
    let svg_data = br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100" width="100" height="100">
    <defs>
        <filter id="outline">
            <feMorphology in="SourceAlpha" result="DILATED" operator="dilate" radius="1" />
            <feFlood flood-color="black" flood-opacity="1" result="FLOOD" />
            <feComposite in="FLOOD" in2="DILATED" operator="in" result="OUTLINE" />
            <feMerge>
                <feMergeNode in="OUTLINE" />
                <feMergeNode in="SourceGraphic" />
            </feMerge>
        </filter>
    </defs>
    <g filter="url(#outline)">
        <rect x="10" y="10" width="80" height="80" fill="#00FFFF" />
    </g>
</svg>"##;

    let tree = usvg::Tree::from_data(svg_data, &usvg::Options::default()).unwrap();
    let mut pixmap = tiny_skia::Pixmap::new(size as u32, size as u32).unwrap();
    let s = size as f32 / 100.0;
    let t = Instant::now();
    resvg::render(
        &tree,
        tiny_skia::Transform::from_scale(s, s),
        &mut pixmap.as_mut(),
    );
    let resvg_ms = t.elapsed().as_secs_f64() * 1000.0;
    println!("[resvg actual]         {:>8.3} ms", resvg_ms);
    println!(
        "\nresvg is {:.1}x slower than theoretical",
        resvg_ms / theoretical_total
    );

    // --- Also test without filter for baseline ---
    let svg_nofilter = br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100" width="100" height="100">
    <rect x="10" y="10" width="80" height="80" fill="#00FFFF" stroke="black" stroke-width="2" />
</svg>"##;
    let tree2 = usvg::Tree::from_data(svg_nofilter, &usvg::Options::default()).unwrap();
    let mut pixmap2 = tiny_skia::Pixmap::new(size as u32, size as u32).unwrap();
    let t = Instant::now();
    resvg::render(
        &tree2,
        tiny_skia::Transform::from_scale(s, s),
        &mut pixmap2.as_mut(),
    );
    let no_filter_ms = t.elapsed().as_secs_f64() * 1000.0;
    println!("[resvg no filter]      {:>8.3} ms", no_filter_ms);
    println!(
        "[filter overhead]      {:>8.3} ms",
        resvg_ms - no_filter_ms
    );

    // Prevent optimizing away
    assert!(dst[0] < 255 || dst4[0][0] < 255 || buf_a[0][3] > 0);
}
