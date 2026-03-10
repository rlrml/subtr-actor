use std::env;
use std::fs;
use std::path::PathBuf;

use subtr_actor::{standard_soccar_boost_pad_layout, BoostPadSize};

const FIELD_HALF_WIDTH: f32 = 4096.0;
const FIELD_HALF_LENGTH: f32 = 5120.0;
const PADDING: f32 = 140.0;
const LABEL_FONT_SIZE: usize = 42;

fn scale_x(x: f32) -> f32 {
    x + FIELD_HALF_WIDTH + PADDING
}

fn scale_y(y: f32) -> f32 {
    FIELD_HALF_LENGTH - y + PADDING
}

fn svg_circle_radius(size: BoostPadSize) -> f32 {
    match size {
        BoostPadSize::Big => 90.0,
        BoostPadSize::Small => 55.0,
    }
}

fn svg_fill(size: BoostPadSize) -> &'static str {
    match size {
        BoostPadSize::Big => "#f97316",
        BoostPadSize::Small => "#38bdf8",
    }
}

fn default_output_path() -> PathBuf {
    PathBuf::from("target/boost_pad_layout.svg")
}

fn render_svg() -> String {
    let width = FIELD_HALF_WIDTH * 2.0 + PADDING * 2.0;
    let height = FIELD_HALF_LENGTH * 2.0 + PADDING * 2.0;
    let field_left = PADDING;
    let field_top = PADDING;
    let field_width = FIELD_HALF_WIDTH * 2.0;
    let field_height = FIELD_HALF_LENGTH * 2.0;

    let mut svg = String::new();
    svg.push_str(&format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {width} {height}" width="{width}" height="{height}">
  <rect width="{width}" height="{height}" fill="#f8fafc" />
  <rect x="{field_left}" y="{field_top}" width="{field_width}" height="{field_height}" rx="24" fill="#ecfccb" stroke="#365314" stroke-width="12" />
  <line x1="{mid_x}" y1="{field_top}" x2="{mid_x}" y2="{bottom}" stroke="#65a30d" stroke-width="10" stroke-dasharray="36 24" />
  <line x1="{field_left}" y1="{mid_y}" x2="{right}" y2="{mid_y}" stroke="#65a30d" stroke-width="6" stroke-dasharray="18 18" />
  <text x="{legend_x}" y="70" font-size="44" font-family="monospace" fill="#1f2937">Standard Soccar Boost Pad Layout</text>
  <text x="{legend_x}" y="118" font-size="28" font-family="monospace" fill="#475569">orange = big pad, blue = small pad, labels = layout index</text>
"##,
        mid_x = scale_x(0.0),
        mid_y = scale_y(0.0),
        right = field_left + field_width,
        bottom = field_top + field_height,
        legend_x = PADDING,
    ));

    for (index, (position, size)) in standard_soccar_boost_pad_layout().iter().enumerate() {
        let cx = scale_x(position.x);
        let cy = scale_y(position.y);
        let label_y = cy - svg_circle_radius(*size) - 20.0;
        svg.push_str(&format!(
            r##"  <circle cx="{cx}" cy="{cy}" r="{radius}" fill="{fill}" fill-opacity="0.82" stroke="#0f172a" stroke-width="10" />
  <text x="{cx}" y="{label_y}" text-anchor="middle" font-size="{font_size}" font-family="monospace" font-weight="700" fill="#111827">{index}</text>
  <text x="{cx}" y="{coord_y}" text-anchor="middle" font-size="22" font-family="monospace" fill="#334155">({x:.0}, {y:.0})</text>
"##,
            radius = svg_circle_radius(*size),
            fill = svg_fill(*size),
            font_size = LABEL_FONT_SIZE,
            coord_y = cy + svg_circle_radius(*size) + 34.0,
            x = position.x,
            y = position.y,
        ));
    }

    svg.push_str("</svg>\n");
    svg
}

fn main() {
    let output_path = env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(default_output_path);

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).expect("failed to create output directory");
    }
    fs::write(&output_path, render_svg()).expect("failed to write SVG");
    println!("{}", output_path.display());
}
