//! Generate Elgato Marketplace media under `marketplace/`.
//!
//! Key tiles use the **same** SVG path as the Stream Deck plugin
//! (`dygma_sd_plugin::visual::render_levels_svg_body`). Banners are composed as
//! SVG (text, cards, product photos) and rasterized with `resvg`.
//!
//! ```text
//! cargo run --manifest-path plugin/Cargo.toml --features gen-marketplace --bin gen-marketplace
//! ```
//!
//! Optional: `--out DIR` (default: `<repo>/marketplace`).

use anyhow::{Context, Result};
use base64::Engine;
use base64::engine::general_purpose::STANDARD as B64;
use dygma_sd_plugin::battery::BatteryLevels;
use dygma_sd_plugin::visual::render_levels_svg_body;
use resvg::tiny_skia;
use resvg::usvg::{Options as UsvgOptions, Tree};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

const BANNER_W: u32 = 1920;
const BANNER_H: u32 = 960;
const FONT_STACK: &str = "Segoe UI, Arial, Helvetica, sans-serif";

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let out_dir = parse_out_dir(&args)?;
    fs::create_dir_all(&out_dir)?;

    let root = repo_root()?;
    // Sources always come from the repo (logo + product-photos).
    // --out only controls where PNGs are written (useful for A/B compare).
    let logo_path = root.join("plugin/assets/dygma-logo.png");
    let photos = root.join("marketplace/product-photos");
    let logo_uri = data_uri_png(&logo_path)?;
    let defy_uri = data_uri_png(&photos.join("defy.png"))?;
    let raise2_uri = data_uri_png(&photos.join("raise2.png"))?;
    let sonsei_uri = data_uri_png(&photos.join("sonsei.png"))?;

    let mut opt = UsvgOptions::default();
    opt.fontdb_mut().load_system_fonts();
    let opt = Arc::new(opt);

    // App icon 288×288
    write_png(
        &out_dir.join("app-icon-288.png"),
        &rasterize_svg(
            &app_icon_svg(&logo_uri),
            288,
            288,
            &opt,
        )?,
    )?;

    // Thumbnail
    let thumb = banner_shell(
        "Dygma Battery",
        "Wireless Left / Right Battery on Stream Deck  |  Defy Verified  |  Raise 2 / Sonsei Beta  |  macOS Beta",
        &logo_uri,
        &thumbnail_body(),
    );
    write_png(
        &out_dir.join("thumbnail-1920x960.png"),
        &rasterize_svg(&thumb, BANNER_W, BANNER_H, &opt)?,
    )?;

    // Gallery 01
    let g1 = banner_shell(
        "Live Key Art",
        "Dual Bars  |  Charge Colors  |  Optional Numbers  |  Charging = Bolt Only  |  Dygma Mark",
        &logo_uri,
        &gallery01_body(),
    );
    write_png(
        &out_dir.join("gallery-01-key-art.png"),
        &rasterize_svg(&g1, BANNER_W, BANNER_H, &opt)?,
    )?;

    // Gallery 02
    let g2 = banner_shell(
        "How It Works",
        "Neuron USB + RF Sides  |  Focus Serial  |  Close Bazecor While Reading",
        &logo_uri,
        &gallery02_body(),
    );
    write_png(
        &out_dir.join("gallery-02-setup.png"),
        &rasterize_svg(&g2, BANNER_W, BANNER_H, &opt)?,
    )?;

    // Gallery 03
    let g3 = banner_shell(
        "Supported Boards",
        "Any Wireless Dygma with Focus wireless.battery.* over Neuron USB",
        &logo_uri,
        &gallery03_body(&defy_uri, &raise2_uri, &sonsei_uri),
    );
    write_png(
        &out_dir.join("gallery-03-boards.png"),
        &rasterize_svg(&g3, BANNER_W, BANNER_H, &opt)?,
    )?;

    println!("Marketplace assets ready in {}", out_dir.display());
    Ok(())
}

fn parse_out_dir(args: &[String]) -> Result<PathBuf> {
    let mut i = 1;
    while i < args.len() {
        if args[i] == "--out" {
            let p = args.get(i + 1).context("--out requires a path")?;
            return Ok(PathBuf::from(p));
        }
        if args[i] == "--help" || args[i] == "-h" {
            println!(
                "Usage: gen-marketplace [--out DIR]\n\
                 Default out: <repo>/marketplace\n\
                 Key art: dygma_sd_plugin::visual (same as the plugin)."
            );
            std::process::exit(0);
        }
        i += 1;
    }
    Ok(repo_root()?.join("marketplace"))
}

fn repo_root() -> Result<PathBuf> {
    // plugin/Cargo.toml → parent of plugin/
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    Ok(manifest
        .parent()
        .context("plugin crate has no parent dir")?
        .to_path_buf())
}

fn data_uri_png(path: &Path) -> Result<String> {
    let bytes = fs::read(path).with_context(|| format!("read {}", path.display()))?;
    Ok(format!(
        "data:image/png;base64,{}",
        B64.encode(bytes)
    ))
}

fn levels(left: u8, right: u8, left_chg: bool, right_chg: bool) -> BatteryLevels {
    BatteryLevels {
        left,
        right,
        left_status: Some(if left_chg { 1 } else { 0 }),
        right_status: Some(if right_chg { 1 } else { 0 }),
    }
}

/// Nested 72×72 key art at pixel position/size, using real plugin SVG body.
fn nested_key(
    x: f32,
    y: f32,
    scale: f32,
    left: u8,
    right: u8,
    left_chg: bool,
    right_chg: bool,
    show_pct: bool,
) -> String {
    let px = 72.0 * scale;
    let body = render_levels_svg_body(&levels(left, right, left_chg, right_chg), show_pct);
    format!(
        r##"<svg x="{x}" y="{y}" width="{px}" height="{px}" viewBox="0 0 72 72" shape-rendering="geometricPrecision">{body}</svg>"##
    )
}

fn banner_shell(title: &str, subtitle: &str, logo_uri: &str, body: &str) -> String {
    let title = esc(title);
    let subtitle = esc(subtitle);
    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{BANNER_W}" height="{BANNER_H}" viewBox="0 0 {BANNER_W} {BANNER_H}">
  <defs>
    <linearGradient id="bg" x1="0" y1="0" x2="{BANNER_W}" y2="{BANNER_H}" gradientUnits="userSpaceOnUse">
      <stop offset="0%" stop-color="#0c0c10"/>
      <stop offset="100%" stop-color="#1c1424"/>
    </linearGradient>
  </defs>
  <rect width="{BANNER_W}" height="{BANNER_H}" fill="url(#bg)"/>
  <rect width="12" height="{BANNER_H}" fill="#f43f27"/>
  <image href="{logo_uri}" x="80" y="80" width="160" height="160"/>
  <text x="280" y="155" fill="#ffffff" font-family="{FONT_STACK}" font-size="54" font-weight="700">{title}</text>
  <text x="280" y="215" fill="#a1a1aa" font-family="{FONT_STACK}" font-size="22">{subtitle}</text>
  {body}
</svg>"##
    )
}

fn app_icon_svg(logo_uri: &str) -> String {
    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="288" height="288" viewBox="0 0 288 288">
  <rect width="288" height="288" fill="#121216"/>
  <image href="{logo_uri}" x="44" y="36" width="200" height="200"/>
</svg>"##
    )
}

fn thumbnail_body() -> String {
    let mut s = String::new();
    // Charging / mid / low / bars-only — same scenarios as the old PS1 generator.
    s.push_str(&nested_key(280.0, 360.0, 4.2, 100, 40, true, true, true));
    s.push_str(&nested_key(620.0, 360.0, 4.2, 72, 55, false, false, true));
    s.push_str(&nested_key(960.0, 360.0, 4.2, 18, 8, false, false, true));
    s.push_str(&nested_key(1300.0, 360.0, 4.2, 90, 90, false, false, false));
    // Separate labels (SVG collapses runs of spaces in a single text node).
    let key_w = 72.0 * 4.2;
    for (x, label) in [
        (280.0, "Charging"),
        (620.0, "Mid Charge"),
        (960.0, "Low"),
        (1300.0, "Bars Only"),
    ] {
        s.push_str(&format!(
            r##"<text x="{cx}" y="740" text-anchor="middle" fill="#71717a" font-family="{FONT_STACK}" font-size="16">{label}</text>"##,
            cx = x + key_w / 2.0,
            label = esc(label),
        ));
    }
    s.push_str(&format!(
        r##"<text x="80" y="900" fill="#f43f27" font-family="{FONT_STACK}" font-size="14" font-weight="700">By Eminence  |  Unofficial Community Plugin  |  Logo Used with Permission</text>"##
    ));
    s
}

fn gallery01_body() -> String {
    let mut s = String::new();
    s.push_str(&nested_key(420.0, 320.0, 6.5, 100, 40, true, true, true));
    s.push_str(&nested_key(1000.0, 320.0, 6.5, 55, 75, false, true, true));
    s
}

fn gallery02_body() -> String {
    let items = [
        (
            "1. Neuron on USB",
            "Focus Serial (Not Pure Bluetooth Mode)",
        ),
        (
            "2. Halves on RF",
            "Wireless Battery Fuel-Gauge over RF to Neuron",
        ),
        (
            "3. Close Bazecor",
            "One Process Owns the Serial Port at a Time",
        ),
        (
            "4. Drop on a Key",
            "Auto-Poll; Press Key to Force Refresh",
        ),
    ];
    let mut s = String::new();
    let mut x = 120.0_f32;
    let y = 340.0_f32;
    for (t, d) in items {
        let lines = wrap_text(d, 34);
        let mut text_lines = String::new();
        for (i, line) in lines.iter().enumerate() {
            let dy = if i == 0 { 0 } else { 22 };
            text_lines.push_str(&format!(
                r##"<tspan x="{tx}" dy="{dy}">{line}</tspan>"##,
                tx = x + 24.0,
                dy = dy,
                line = esc(line),
            ));
        }
        s.push_str(&format!(
            r##"<rect x="{x}" y="{y}" width="400" height="200" rx="16" fill="#1c1c24"/>
<text x="{tx}" y="{ty}" fill="#ffffff" font-family="{FONT_STACK}" font-size="20" font-weight="700">{title}</text>
<text x="{tx}" y="{dy}" fill="#b4b4be" font-family="{FONT_STACK}" font-size="16">{text_lines}</text>
"##,
            x = x,
            y = y,
            tx = x + 24.0,
            ty = y + 60.0,
            dy = y + 110.0,
            title = esc(t),
        ));
        x += 440.0;
    }
    s
}

fn gallery03_body(defy: &str, raise2: &str, sonsei: &str) -> String {
    let boards = [
        ("Defy", "Columnar Wireless", "Verified", defy),
        ("Raise 2", "Row-Staggered Wireless", "Beta", raise2),
        ("Sonsei", "Low-Profile Wireless", "Beta", sonsei),
    ];
    let card_w = 520.0_f32;
    let card_h = 480.0_f32;
    let gap = 40.0_f32;
    let total_w = 3.0 * card_w + 2.0 * gap;
    let mut x = (BANNER_W as f32 - total_w) / 2.0;
    let y = 300.0_f32;
    let photo_pad = 20.0_f32;
    let photo_h = 250.0_f32;
    let mut s = String::new();
    for (name, desc, status, img) in boards {
        let px = x + photo_pad;
        let py = y + photo_pad;
        let pw = card_w - 2.0 * photo_pad;
        let text_y = py + photo_h + 18.0;
        let text_x = x + 28.0;
        // Clip photo to rounded well via nested svg
        s.push_str(&format!(
            r##"<rect x="{x}" y="{y}" width="{card_w}" height="{card_h}" rx="20" fill="#1c1c24"/>
<svg x="{px}" y="{py}" width="{pw}" height="{photo_h}">
  <rect width="{pw}" height="{photo_h}" rx="14" fill="#121218"/>
  <image href="{img}" x="0" y="0" width="{pw}" height="{photo_h}" preserveAspectRatio="xMidYMid meet"/>
</svg>
<text x="{text_x}" y="{ny}" fill="#ffffff" font-family="{FONT_STACK}" font-size="26" font-weight="700">{name}</text>
<text x="{text_x}" y="{dy}" fill="#a1a1aa" font-family="{FONT_STACK}" font-size="15">{desc}</text>
<text x="{text_x}" y="{sy}" fill="#f43f27" font-family="{FONT_STACK}" font-size="14" font-weight="700">{status}</text>
<text x="{text_x}" y="{my}" fill="#a1a1aa" font-family="{FONT_STACK}" font-size="13">Windows Primary  |  macOS Beta</text>
"##,
            name = esc(name),
            desc = esc(desc),
            status = esc(status),
            ny = text_y + 28.0,
            dy = text_y + 58.0,
            sy = text_y + 96.0,
            my = text_y + 124.0,
        ));
        x += card_w + gap;
    }
    s
}

fn wrap_text(s: &str, max_chars: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut cur = String::new();
    for word in s.split_whitespace() {
        if cur.is_empty() {
            cur = word.to_string();
        } else if cur.len() + 1 + word.len() <= max_chars {
            cur.push(' ');
            cur.push_str(word);
        } else {
            lines.push(cur);
            cur = word.to_string();
        }
    }
    if !cur.is_empty() {
        lines.push(cur);
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn rasterize_svg(svg: &str, width: u32, height: u32, opt: &UsvgOptions) -> Result<tiny_skia::Pixmap> {
    let tree = Tree::from_str(svg, opt).context("parse SVG")?;
    let mut pixmap =
        tiny_skia::Pixmap::new(width, height).context("allocate pixmap")?;
    let size = tree.size();
    let sx = width as f32 / size.width().max(1.0);
    let sy = height as f32 / size.height().max(1.0);
    let transform = tiny_skia::Transform::from_scale(sx, sy);
    resvg::render(&tree, transform, &mut pixmap.as_mut());
    Ok(pixmap)
}

fn write_png(path: &Path, pixmap: &tiny_skia::Pixmap) -> Result<()> {
    pixmap
        .save_png(path)
        .with_context(|| format!("write {}", path.display()))?;
    println!("Wrote {} ({}x{})", path.display(), pixmap.width(), pixmap.height());
    Ok(())
}

