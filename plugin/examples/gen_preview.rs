//! Write `docs/preview.svg` from real plugin key SVG bodies.
//!
//! ```bash
//! cargo run --manifest-path plugin/Cargo.toml --example gen_preview
//! ```

use dygma_sd_plugin::battery::BatteryLevels;
use dygma_sd_plugin::visual::render_levels_svg_body;
use std::env;
use std::fs;
use std::path::PathBuf;

fn levels(left: u8, right: u8, left_chg: bool, right_chg: bool) -> BatteryLevels {
    BatteryLevels {
        left,
        right,
        left_status: Some(if left_chg { 1 } else { 0 }),
        right_status: Some(if right_chg { 1 } else { 0 }),
    }
}

fn main() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("plugin parent")
        .to_path_buf();
    let out = root.join("docs/preview.svg");

    // Same scenarios as marketplace thumbnail (updated key art).
    let samples: [(&str, BatteryLevels, bool); 4] = [
        (
            "Charging",
            levels(100, 40, true, true),
            true,
        ),
        (
            "Mid Charge",
            levels(72, 55, false, false),
            true,
        ),
        ("Low", levels(18, 8, false, false), true),
        (
            "Bars Only",
            levels(90, 90, false, false),
            false,
        ),
    ];

    let scale = 2.5_f32;
    let key = 72.0 * scale;
    let gap = 28.0_f32;
    let pad_x = 24.0_f32;
    let pad_top = 44.0_f32;
    let label_y = pad_top + key + 28.0;
    let width = pad_x * 2.0 + 4.0 * key + 3.0 * gap;
    let height = label_y + 24.0;

    let mut body = String::new();
    for (i, (label, lv, show_pct)) in samples.iter().enumerate() {
        let x = pad_x + i as f32 * (key + gap);
        let svg_body = render_levels_svg_body(lv, *show_pct);
        body.push_str(&format!(
            r##"<g transform="translate({x},{pad_top}) scale({scale})">
  <rect x="-2" y="-2" width="76" height="76" rx="10" fill="#0a0a0c" stroke="#2a2a32" stroke-width="1.2"/>
  {svg_body}
</g>
<text x="{cx}" y="{label_y}" text-anchor="middle" fill="#71717a" font-family="Segoe UI,system-ui,sans-serif" font-size="12">{label}</text>
"##,
            cx = x + key / 2.0,
        ));
    }

    let svg = format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{width:.0}" height="{height:.0}" viewBox="0 0 {width:.1} {height:.1}" role="img" aria-label="Dygma Stream Deck battery key previews">
<defs>
  <linearGradient id="bg" x1="0" y1="0" x2="1" y2="1">
    <stop offset="0%" stop-color="#0c0c10"/>
    <stop offset="100%" stop-color="#18181f"/>
  </linearGradient>
</defs>
<rect width="100%" height="100%" rx="16" fill="url(#bg)"/>
<text x="24" y="28" fill="#a1a1aa" font-family="Segoe UI,system-ui,sans-serif" font-size="14" font-weight="600">Dygma Battery · Stream Deck Key Art</text>
{body}
</svg>
"##
    );

    fs::create_dir_all(out.parent().unwrap()).unwrap();
    fs::write(&out, svg).unwrap();
    println!("Wrote {}", out.display());
}
