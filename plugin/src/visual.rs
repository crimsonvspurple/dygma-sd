//! Dynamic SVG key art for dual battery bars.

use crate::battery::BatteryLevels;
use base64::Engine;
use base64::engine::general_purpose::STANDARD as B64;

const VIEW: f32 = 72.0;
const BLOCKS: u8 = 5;

/// Map charge % → filled block count (1..=5), 0 only when empty.
///
/// - 5: charge > 80  
/// - 4: 60 < charge ≤ 80  
/// - 3: 40 < charge ≤ 60  
/// - 2: 20 < charge ≤ 40  
/// - 1: 0 < charge ≤ 20  
/// - 0: charge == 0  
pub fn blocks_for_percent(pct: u8) -> u8 {
    match pct.min(100) {
        0 => 0,
        1..=20 => 1,
        21..=40 => 2,
        41..=60 => 3,
        61..=80 => 4,
        _ => 5,
    }
}

/// Color for the filled blocks of a bar at this block count.
pub fn color_for_blocks(blocks: u8) -> &'static str {
    match blocks {
        5 => "#22c55e", // green
        4 => "#a3e635", // lime / towards yellow
        3 => "#eab308", // yellow
        2 => "#f97316", // orange
        1 => "#ef4444", // red (~20%)
        _ => "#3f3f46", // empty outline shade
    }
}

/// Whether Focus status indicates charging for that side.
///
/// Older Focus docs list `2 = charging`, `1 = discharging`. On Defy FW 2.2.1
/// we observe `0` on RF battery and `1` while a side is on charge, so both
/// `1` and `2` are treated as charging.
pub fn is_charging(status: Option<u8>) -> bool {
    matches!(status, Some(1) | Some(2))
}

/// Build a `data:image/svg+xml;base64,...` URI for Stream Deck `setImage`.
pub fn key_image_data_uri(levels: &BatteryLevels, show_percentage: bool) -> String {
    let svg = render_levels_svg(levels, show_percentage);
    format!("data:image/svg+xml;base64,{}", B64.encode(svg.as_bytes()))
}

pub fn loading_image_data_uri() -> String {
    let mut svg = String::from(
        r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 72 72"><rect width="72" height="72" rx="8" fill="#141418"/>"##,
    );
    draw_dygma_logo(&mut svg, 36.0, 30.0, 28.0);
    svg.push_str(
        r##"<text x="36" y="58" text-anchor="middle" fill="#a1a1aa" font-family="Segoe UI,system-ui,sans-serif" font-size="12" font-weight="600">…</text></svg>"##,
    );
    format!("data:image/svg+xml;base64,{}", B64.encode(svg.as_bytes()))
}

pub fn error_image_data_uri() -> String {
    let mut svg = String::from(
        r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 72 72"><rect width="72" height="72" rx="8" fill="#141418"/>"##,
    );
    draw_dygma_logo(&mut svg, 36.0, 26.0, 22.0);
    svg.push_str(
        r##"<text x="36" y="50" text-anchor="middle" fill="#f87171" font-family="Segoe UI,system-ui,sans-serif" font-size="11" font-weight="700">ERR</text>
<text x="36" y="63" text-anchor="middle" fill="#fca5a5" font-family="Segoe UI,system-ui,sans-serif" font-size="10">COM</text></svg>"##,
    );
    format!("data:image/svg+xml;base64,{}", B64.encode(svg.as_bytes()))
}

pub fn render_levels_svg(levels: &BatteryLevels, show_percentage: bool) -> String {
    let left_charge = is_charging(levels.left_status);
    let right_charge = is_charging(levels.right_status);
    // While a side is on cable charge the fuel-gauge % is unreliable, so we
    // keep empty outlines + bolt only (no filled blocks / number for that side).
    let left_blocks = if left_charge {
        0
    } else {
        blocks_for_percent(levels.left)
    };
    let right_blocks = if right_charge {
        0
    } else {
        blocks_for_percent(levels.right)
    };

    // Layout: two columns, padding, room for bolts / optional numbers.
    let pad_x = 6.0_f32;
    let pad_top = if left_charge || right_charge {
        14.0
    } else {
        8.0
    };
    // Extra bottom pad when numbers are on; logo sits above them in the taper gap.
    let pad_bottom = if show_percentage { 18.0 } else { 10.0 };
    let gap = 10.0_f32;
    let col_w = (VIEW - pad_x * 2.0 - gap) / 2.0;
    let stack_h = VIEW - pad_top - pad_bottom;
    let block_gap = 2.0_f32;
    let block_h = (stack_h - block_gap * (BLOCKS as f32 - 1.0)) / BLOCKS as f32;

    let mut out = String::with_capacity(4096);
    out.push_str(
        r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 72 72" shape-rendering="geometricPrecision">"##,
    );
    out.push_str(r##"<rect width="72" height="72" rx="8" fill="#141418"/>"##);

    // Left column (left-aligned taper: bottom narrower)
    let left_x0 = pad_x;
    draw_bar(
        &mut out,
        left_x0,
        pad_top,
        col_w,
        block_h,
        block_gap,
        left_blocks,
        Align::Left,
        left_charge,
    );

    // Right column (right-aligned taper)
    let right_x0 = pad_x + col_w + gap;
    draw_bar(
        &mut out,
        right_x0,
        pad_top,
        col_w,
        block_h,
        block_gap,
        right_blocks,
        Align::Right,
        right_charge,
    );

    // Dygma mark in the center gap. Bars taper narrower at the bottom, so we
    // can sit the logo a bit higher and larger without colliding with them.
    // Numbers (no %) live under each column; logo sits above that baseline.
    let logo_size = if show_percentage { 11.0_f32 } else { 12.0_f32 };
    let logo_cy = if show_percentage {
        VIEW - 14.0
    } else {
        VIEW - 10.0
    };
    draw_dygma_logo(&mut out, VIEW * 0.5, logo_cy, logo_size);

    if show_percentage {
        let y = VIEW - 5.0;
        // Skip numbers on charging sides (level is not trustworthy while charging).
        if !left_charge {
            out.push_str(&format!(
                r##"<text x="{:.1}" y="{:.1}" text-anchor="middle" fill="#e4e4e7" font-family="Segoe UI,system-ui,sans-serif" font-size="9" font-weight="600">{}</text>"##,
                left_x0 + col_w / 2.0,
                y,
                levels.left.min(100)
            ));
        }
        if !right_charge {
            out.push_str(&format!(
                r##"<text x="{:.1}" y="{:.1}" text-anchor="middle" fill="#e4e4e7" font-family="Segoe UI,system-ui,sans-serif" font-size="9" font-weight="600">{}</text>"##,
                right_x0 + col_w / 2.0,
                y,
                levels.right.min(100)
            ));
        }
    }

    out.push_str("</svg>");
    out
}

#[derive(Clone, Copy)]
enum Align {
    Left,
    Right,
}

/// Block index 0 = bottom (narrowest), 4 = top (widest).
fn block_width(col_w: f32, index_from_bottom: u8) -> f32 {
    let t = index_from_bottom as f32 / (BLOCKS - 1) as f32; // 0..1
    let min_frac = 0.42;
    let max_frac = 1.0;
    col_w * (min_frac + (max_frac - min_frac) * t)
}

#[allow(clippy::too_many_arguments)]
fn draw_bar(
    out: &mut String,
    col_x: f32,
    top: f32,
    col_w: f32,
    block_h: f32,
    block_gap: f32,
    filled: u8,
    align: Align,
    charging: bool,
) {
    let fill = color_for_blocks(filled);
    let empty = "#2a2a30";
    let stroke = "#3f3f46";

    // Blocks bottom → top
    for i in 0..BLOCKS {
        let from_bottom = i;
        let y = top + (BLOCKS - 1 - i) as f32 * (block_h + block_gap);
        let w = block_width(col_w, from_bottom);
        let x = match align {
            Align::Left => col_x,
            Align::Right => col_x + (col_w - w),
        };
        let is_filled = from_bottom < filled;
        let color = if is_filled { fill } else { empty };
        out.push_str(&format!(
            r##"<rect x="{:.2}" y="{:.2}" width="{:.2}" height="{:.2}" rx="2" fill="{}" stroke="{}" stroke-width="0.6"/>"##,
            x, y, w, block_h, color, stroke
        ));
    }

    if charging {
        // Thunderbolt above the column center (bright so it reads on dark key)
        let cx = col_x + col_w / 2.0;
        let cy = top - 6.5;
        draw_bolt(out, cx, cy, "#fde047");
    }
}

fn draw_bolt(out: &mut String, cx: f32, cy: f32, color: &str) {
    // Compact lightning bolt path centered at (cx, cy)
    let s = 0.72_f32;
    let points = [
        (2.0, -7.0),
        (-3.5, 0.5),
        (-0.5, 0.5),
        (-2.5, 7.0),
        (3.5, -0.2),
        (0.8, -0.2),
    ];
    out.push_str(&format!(
        r##"<path fill="{}" stroke="#1a1a1e" stroke-width="0.4" d="M"##,
        color
    ));
    for (i, (px, py)) in points.iter().enumerate() {
        let x = cx + px * s;
        let y = cy + py * s;
        if i == 0 {
            out.push_str(&format!("{x:.2} {y:.2}"));
        } else {
            out.push_str(&format!(" L{x:.2} {y:.2}"));
        }
    }
    out.push_str(r##" Z"/>"##);
}

/// Tiny Dygma brand mark (geometry from Bazecor `logo.svg`, viewBox 0 0 56 56).
/// Drawn as solid brand-color facets so it stays legible at ~8–10 px.
fn draw_dygma_logo(out: &mut String, cx: f32, cy: f32, size: f32) {
    let s = size / 56.0;
    let tx = cx - size * 0.5;
    let ty = cy - size * 0.5;
    out.push_str(&format!(
        r##"<g transform="translate({tx:.2} {ty:.2}) scale({s:.5})" aria-label="Dygma">"##
    ));
    // Faceted mark (red → magenta → purple), center stays transparent (key bg).
    out.push_str(
        r##"<path fill="#F43F27" d="M42.5 6.8c-0.2 0-0.5 0-0.8 0-0.5 0-1.1 0-1.6 0-3.3 0.1-6.5 0.7-9.5 1.7l10 3.7 2.3 0.9 2.1 0.8-3.5 15.6c-1.4 6.9-3.9 13.4-7.3 19.4l6.5-7.4 2.3-2.6 0.3-0.3 3.4-3.9c0.5-0.6 0.9-1.2 1.1-2l1.2-4 5.9-19.6C51 7.7 46.8 6.9 42.5 6.8z"/>
<path fill="#E11D48" d="M42.5 6.8c-0.2 0-0.5 0-0.8 0-0.5 0-1.1 0-1.6 0-3.3 0.1-6.5 0.7-9.5 1.7l10 3.7 2.3 0.9 2.1 0.8 9.7-4.6C51 7.7 46.8 6.9 42.5 6.8z"/>
<path fill="#7C1D6F" d="M9.2 22.8c0.5 0.5 0.9 1.1 1.4 1.6 1.1 1.1 2.2 2.2 3.3 3.2l-1.4-6.1L12 18.9l-1.1-5L28 7.5c3.7-1.3 7.8-2.1 11.9-2.1h0.2c0.7 0 1.5 0 2.2 0.1 1 0.1 2 0.2 2.9 0.3l-3-1.1-2.5-0.9-5.6-2-4.3-1.5c-1.1-0.4-2.4-0.4-3.5 0L21.4 2 5.9 7.5 1.2 9.2C3.1 14.2 5.8 18.8 9.2 22.8z"/>
<path fill="#4C0783" d="M9.2 22.8c0.5 0.5 0.9 1.1 1.4 1.6 1.1 1.1 2.2 2.2 3.3 3.2l-1.4-6.1L12 18.9l-1.1-5L1.2 9.2C3.1 14.2 5.8 18.8 9.2 22.8z"/>
<path fill="#A80B2D" d="M33.6 37.5l-2.7 2.3L28 42.1 14.9 31.4c-1.9-1.5-3.6-3.2-5.3-5.1-0.5-0.5-1-1.1-1.4-1.7-1.7-2.2-3.2-4.5-4.5-6.9l2.7 9.1 0.3 1.1 0.9 3.1 0.6 1.9c0.2 0.7 0.6 1.4 1.1 2 0.5 0.5 1.1 1.3 1.9 2.2 0.5 0.6 1.1 1.3 1.7 2 5.7 6.5 14.7 16.8 15 17.1 4.8-6.7 8.5-14.4 10.6-22.6L33.6 37.5z"/>
<path fill="#8E0939" d="M33.6 37.5l-2.7 2.3L28 42.1v13.9c4.8-6.7 8.5-14.4 10.6-22.6L33.6 37.5z"/>
"##,
    );
    out.push_str("</g>");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn block_thresholds() {
        assert_eq!(blocks_for_percent(0), 0);
        assert_eq!(blocks_for_percent(1), 1);
        assert_eq!(blocks_for_percent(20), 1);
        assert_eq!(blocks_for_percent(21), 2);
        assert_eq!(blocks_for_percent(40), 2);
        assert_eq!(blocks_for_percent(41), 3);
        assert_eq!(blocks_for_percent(60), 3);
        assert_eq!(blocks_for_percent(61), 4);
        assert_eq!(blocks_for_percent(80), 4);
        assert_eq!(blocks_for_percent(81), 5);
        assert_eq!(blocks_for_percent(100), 5);
    }

    #[test]
    fn colors_follow_tier() {
        assert_eq!(color_for_blocks(5), "#22c55e");
        assert_eq!(color_for_blocks(4), "#a3e635");
        assert_eq!(color_for_blocks(3), "#eab308");
        assert_eq!(color_for_blocks(2), "#f97316");
        assert_eq!(color_for_blocks(1), "#ef4444");
    }

    #[test]
    fn charging_status() {
        assert!(!is_charging(None));
        assert!(!is_charging(Some(0)));
        assert!(is_charging(Some(1))); // observed on Defy FW 2.2.1 while charging
        assert!(is_charging(Some(2))); // Focus docs / other firmware
        assert!(!is_charging(Some(3)));
    }

    #[test]
    fn svg_contains_bars_and_optional_percent() {
        let levels = BatteryLevels {
            left: 100,
            right: 40,
            left_status: Some(0),
            right_status: Some(0),
        };
        let with_pct = render_levels_svg(&levels, true);
        // Numbers only — no trailing % (avoids collision with center logo)
        assert!(with_pct.contains(">100</text>"));
        assert!(with_pct.contains(">40</text>"));
        assert!(!with_pct.contains("100%"));
        assert!(!with_pct.contains("40%"));
        assert!(with_pct.contains("#22c55e")); // left full green
        assert!(with_pct.contains("#f97316")); // right 2 blocks orange
        // Dygma brand mark present
        assert!(with_pct.contains("aria-label=\"Dygma\""));
        assert!(with_pct.contains("#F43F27"));

        let no_pct = render_levels_svg(&levels, false);
        assert!(!no_pct.contains(">100</text>"));
        assert!(!no_pct.contains("100%"));
        assert!(no_pct.contains("aria-label=\"Dygma\""));
    }

    #[test]
    fn charging_hides_fill_and_number_keeps_bolt() {
        // Stale/wrong levels while charging must not paint filled bars or digits.
        let levels = BatteryLevels {
            left: 100,
            right: 40,
            left_status: Some(1),
            right_status: Some(2),
        };
        let svg = render_levels_svg(&levels, true);
        assert!(!svg.contains(">100</text>"));
        assert!(!svg.contains(">40</text>"));
        assert!(!svg.contains("#22c55e"));
        assert!(!svg.contains("#f97316"));
        // Empty block fill + yellow bolt path
        assert!(svg.contains("#2a2a30"));
        assert!(svg.contains("#fde047"));
        assert!(svg.contains("aria-label=\"Dygma\""));

        // Mixed: only the charging side is blanked.
        let mixed = BatteryLevels {
            left: 55,
            right: 80,
            left_status: Some(0),
            right_status: Some(1),
        };
        let mixed_svg = render_levels_svg(&mixed, true);
        assert!(mixed_svg.contains(">55</text>"));
        assert!(!mixed_svg.contains(">80</text>"));
        assert!(mixed_svg.contains("#eab308")); // left 3 blocks yellow
        assert!(mixed_svg.contains("#fde047")); // right bolt
    }

    #[test]
    fn data_uri_prefix() {
        let levels = BatteryLevels {
            left: 50,
            right: 50,
            left_status: None,
            right_status: None,
        };
        let uri = key_image_data_uri(&levels, true);
        assert!(uri.starts_with("data:image/svg+xml;base64,"));
    }
}
