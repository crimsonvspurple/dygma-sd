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

/// Focus status: 2 = charging (Bazecor / Focus API table).
pub fn is_charging(status: Option<u8>) -> bool {
    matches!(status, Some(2))
}

/// Build a `data:image/svg+xml;base64,...` URI for Stream Deck `setImage`.
pub fn key_image_data_uri(levels: &BatteryLevels, show_percentage: bool) -> String {
    let svg = render_levels_svg(levels, show_percentage);
    format!("data:image/svg+xml;base64,{}", B64.encode(svg.as_bytes()))
}

pub fn loading_image_data_uri() -> String {
    let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 72 72">
  <rect width="72" height="72" fill="#1a1a1e"/>
  <text x="36" y="40" text-anchor="middle" fill="#a1a1aa" font-family="Segoe UI,system-ui,sans-serif" font-size="18" font-weight="600">…</text>
</svg>"##;
    format!("data:image/svg+xml;base64,{}", B64.encode(svg.as_bytes()))
}

pub fn error_image_data_uri() -> String {
    let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 72 72">
  <rect width="72" height="72" fill="#1a1a1e"/>
  <text x="36" y="32" text-anchor="middle" fill="#f87171" font-family="Segoe UI,system-ui,sans-serif" font-size="14" font-weight="700">ERR</text>
  <text x="36" y="50" text-anchor="middle" fill="#fca5a5" font-family="Segoe UI,system-ui,sans-serif" font-size="12">COM</text>
</svg>"##;
    format!("data:image/svg+xml;base64,{}", B64.encode(svg.as_bytes()))
}

pub fn render_levels_svg(levels: &BatteryLevels, show_percentage: bool) -> String {
    let left_blocks = blocks_for_percent(levels.left);
    let right_blocks = blocks_for_percent(levels.right);
    let left_charge = is_charging(levels.left_status);
    let right_charge = is_charging(levels.right_status);

    // Layout: two columns, padding, room for bolts / optional %.
    let pad_x = 6.0_f32;
    let pad_top = if left_charge || right_charge {
        14.0
    } else {
        8.0
    };
    let pad_bottom = if show_percentage { 16.0 } else { 8.0 };
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

    if show_percentage {
        let y = VIEW - 5.0;
        out.push_str(&format!(
            r##"<text x="{:.1}" y="{:.1}" text-anchor="middle" fill="#e4e4e7" font-family="Segoe UI,system-ui,sans-serif" font-size="9" font-weight="600">{}%</text>"##,
            left_x0 + col_w / 2.0,
            y,
            levels.left.min(100)
        ));
        out.push_str(&format!(
            r##"<text x="{:.1}" y="{:.1}" text-anchor="middle" fill="#e4e4e7" font-family="Segoe UI,system-ui,sans-serif" font-size="9" font-weight="600">{}%</text>"##,
            right_x0 + col_w / 2.0,
            y,
            levels.right.min(100)
        ));
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
        // Thunderbolt above the column center
        let cx = col_x + col_w / 2.0;
        let cy = top - 7.0;
        draw_bolt(out, cx, cy, fill);
    }
}

fn draw_bolt(out: &mut String, cx: f32, cy: f32, color: &str) {
    // Compact lightning bolt path centered at (cx, cy)
    let s = 0.55_f32;
    // Path relative to center
    let points = [
        (2.0, -7.0),
        (-3.5, 0.5),
        (-0.5, 0.5),
        (-2.5, 7.0),
        (3.5, -0.2),
        (0.8, -0.2),
    ];
    out.push_str(&format!(r##"<path fill="{}" d="M"##, color));
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
        assert!(!is_charging(Some(1)));
        assert!(is_charging(Some(2)));
        assert!(!is_charging(Some(3)));
    }

    #[test]
    fn svg_contains_bars_and_optional_percent() {
        let levels = BatteryLevels {
            left: 100,
            right: 40,
            left_status: Some(1),
            right_status: Some(2),
        };
        let with_pct = render_levels_svg(&levels, true);
        assert!(with_pct.contains("100%"));
        assert!(with_pct.contains("40%"));
        assert!(with_pct.contains("#22c55e")); // left full green
        assert!(with_pct.contains("#f97316")); // right 2 blocks orange
        // charging bolt on right (status 2)
        assert!(with_pct.contains("<path fill="));

        let no_pct = render_levels_svg(&levels, false);
        assert!(!no_pct.contains("100%"));
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
