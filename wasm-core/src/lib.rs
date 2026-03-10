use wasm_bindgen::prelude::*;
use image::{ImageBuffer, RgbaImage, Rgba};
use ab_glyph::{FontRef, PxScale, Font, ScaleFont, point};
use std::io::Cursor;

const FONT_OPEN_SANS: &[u8] = include_bytes!("OpenSans-Regular.ttf");
const FONT_ROBOTO: &[u8] = include_bytes!("Roboto-Regular.ttf");
const FONT_LORA: &[u8] = include_bytes!("Lora-Regular.ttf");
const FONT_PACIFICO: &[u8] = include_bytes!("Pacifico-Regular.ttf");

fn parse_hex_color(hex: &str) -> Result<Rgba<u8>, JsValue> {
    if hex == "transparent" {
        return Ok(Rgba([0, 0, 0, 0]));
    }

    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 && hex.len() != 8 {
        return Err(JsValue::from_str("Invalid hex color format"));
    }

    let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| JsValue::from_str("Invalid red channel"))?;
    let g = u8::from_str_radix(&hex[2..4], 16).map_err(|_| JsValue::from_str("Invalid green channel"))?;
    let b = u8::from_str_radix(&hex[4..6], 16).map_err(|_| JsValue::from_str("Invalid blue channel"))?;
    let a = if hex.len() == 8 {
        u8::from_str_radix(&hex[6..8], 16).map_err(|_| JsValue::from_str("Invalid alpha channel"))?
    } else {
        255
    };

    Ok(Rgba([r, g, b, a]))
}

// Helper to pre-calculate linear sRGB values to save powf operations during blending
struct SrgbToLinearTable {
    table: [f32; 256],
}

impl SrgbToLinearTable {
    fn new() -> Self {
        let mut table = [0.0; 256];
        for i in 0..256 {
            table[i] = (i as f32 / 255.0).powf(2.2);
        }
        Self { table }
    }

    fn get(&self, srgb_val: u8) -> f32 {
        self.table[srgb_val as usize]
    }
}

// Improved blending with gamma correction for better antialiasing
fn blend_pixel(background: &mut Rgba<u8>, color: &Rgba<u8>, alpha: f32, linear_table: &SrgbToLinearTable) {
    if alpha <= 0.0 { return; }

    let out_alpha = (color[3] as f32 / 255.0) * alpha;
    if out_alpha <= 0.0 { return; }

    let bg_alpha = background[3] as f32 / 255.0;

    // Fast path if background is completely transparent
    if bg_alpha <= 0.0 {
        for i in 0..3 {
            background[i] = color[i];
        }
        background[3] = (out_alpha * 255.0).round() as u8;
        return;
    }

    let new_alpha = out_alpha + bg_alpha * (1.0 - out_alpha);
    if new_alpha <= 0.0 { return; }

    // Convert to linear space for better blending
    for i in 0..3 {
        let fg_c = linear_table.get(color[i]);
        let bg_c = linear_table.get(background[i]);
        let new_c = (fg_c * out_alpha + bg_c * bg_alpha * (1.0 - out_alpha)) / new_alpha;
        // Convert back to sRGB space
        background[i] = (new_c.powf(1.0 / 2.2) * 255.0).round() as u8;
    }
    background[3] = (new_alpha * 255.0).round() as u8;
}

// Helper to bilinearly filter from the tile for smoother subpixel offsets and supersampling
fn sample_tile(tile: &RgbaImage, u: f32, v: f32) -> Rgba<u8> {
    let w = tile.width() as f32;
    let h = tile.height() as f32;

    // Positive modulo to wrap around seamlessly
    let mut u = u % w;
    if u < 0.0 { u += w; }
    let mut v = v % h;
    if v < 0.0 { v += h; }

    let x0 = u.floor() as u32 % tile.width();
    let y0 = v.floor() as u32 % tile.height();
    let x1 = (x0 + 1) % tile.width();
    let y1 = (y0 + 1) % tile.height();

    let fx = u - u.floor();
    let fy = v - v.floor();

    let p00 = tile.get_pixel(x0, y0);
    let p10 = tile.get_pixel(x1, y0);
    let p01 = tile.get_pixel(x0, y1);
    let p11 = tile.get_pixel(x1, y1);

    let mut out = [0u8; 4];
    for i in 0..4 {
        let c00 = p00[i] as f32;
        let c10 = p10[i] as f32;
        let c01 = p01[i] as f32;
        let c11 = p11[i] as f32;

        let c0 = c00 * (1.0 - fx) + c10 * fx;
        let c1 = c01 * (1.0 - fx) + c11 * fx;
        let c = c0 * (1.0 - fy) + c1 * fy;
        out[i] = c.round() as u8;
    }

    Rgba(out)
}

#[wasm_bindgen]
pub fn generate_text_pattern(
    text: String,
    font_family: String,
    font_size: f32,
    text_color: String,
    bg_color: String,
    width: u32,
    height: u32,
    horizontal_spacing: f32,
    vertical_spacing: f32,
    rotation_angle: f32,
) -> Result<Vec<u8>, JsValue> {
    console_error_panic_hook::set_once();

    let linear_table = SrgbToLinearTable::new();
    let text_rgba = parse_hex_color(&text_color)?;
    let bg_rgba = parse_hex_color(&bg_color)?;

    let font_data = match font_family.as_str() {
        "Roboto" => FONT_ROBOTO,
        "Lora" => FONT_LORA,
        "Pacifico" => FONT_PACIFICO,
        _ => FONT_OPEN_SANS,
    };

    let font = FontRef::try_from_slice(font_data)
        .map_err(|_| JsValue::from_str("Failed to load font"))?;

    // We render into the tile with SSAA to retain quality
    let supersample_factor = 2.0;
    let render_scale = PxScale::from(font_size * supersample_factor);
    let render_font = font.as_scaled(render_scale);
    let text_height = render_font.height();

    let mut glyphs = Vec::new();
    let mut current_x = 0.0;
    let baseline = render_font.ascent();

    for c in text.chars() {
        let glyph_id = font.glyph_id(c);
        let mut q_glyph = render_font.scaled_glyph(c);
        q_glyph.position = point(current_x, baseline);

        let advance = render_font.h_advance(glyph_id);

        glyphs.push((glyph_id, q_glyph));
        current_x += advance;
    }
    let text_width = current_x;

    let step_x = text_width + (horizontal_spacing * supersample_factor);
    let step_y = text_height + (vertical_spacing * supersample_factor);

    if step_x <= 0.0 || step_y <= 0.0 {
        return Err(JsValue::from_str("Invalid step size"));
    }

    // 1. Render the text ONCE into a single supersampled tile buffer
    let ss_tile_width = step_x.ceil() as u32;
    let ss_tile_height = step_y.ceil() as u32;
    let mut ss_tile = RgbaImage::new(ss_tile_width, ss_tile_height);

    for p in ss_tile.pixels_mut() {
        *p = Rgba([0, 0, 0, 0]);
    }

    // Center the text inside the supersampled tile
    let offset_x = (step_x - text_width) / 2.0;
    let offset_y = (step_y - text_height) / 2.0;

    for (_glyph_id, q_glyph) in &glyphs {
        if let Some(q_outline) = render_font.outline_glyph(q_glyph.clone()) {
            let bb = q_outline.px_bounds();
            q_outline.draw(|gx, gy, v| {
                let coverage = v.powf(1.0 / 2.2);
                let px = (bb.min.x + gx as f32 + offset_x).round() as i32;
                let py = (bb.min.y + gy as f32 + offset_y).round() as i32;

                if px >= 0 && px < ss_tile_width as i32 && py >= 0 && py < ss_tile_height as i32 {
                    let pixel = ss_tile.get_pixel_mut(px as u32, py as u32);
                    blend_pixel(pixel, &text_rgba, coverage, &linear_table);
                }
            });
        }
    }

    // 2. Downsample the tile to its final unrotated size
    let tile_width = (step_x / supersample_factor).ceil() as u32;
    let tile_height = (step_y / supersample_factor).ceil() as u32;
    let mut tile = RgbaImage::new(tile_width, tile_height);

    for ty in 0..tile_height {
        for tx in 0..tile_width {
            let mut sums = [0.0f32; 4];
            let mut count = 0.0;

            let start_x = tx as f32 * supersample_factor;
            let start_y = ty as f32 * supersample_factor;

            for dy in 0..(supersample_factor as u32) {
                for dx in 0..(supersample_factor as u32) {
                    let rx = start_x as u32 + dx;
                    let ry = start_y as u32 + dy;

                    if rx < ss_tile_width && ry < ss_tile_height {
                        let p = ss_tile.get_pixel(rx, ry);
                        for i in 0..3 {
                            sums[i] += linear_table.get(p[i]);
                        }
                        sums[3] += p[3] as f32 / 255.0;
                        count += 1.0;
                    }
                }
            }

            if count > 0.0 {
                let out_alpha = sums[3] / count;
                let mut final_color = [0u8; 4];
                if out_alpha > 0.0 {
                    for i in 0..3 {
                        let linear_c = sums[i] / count;
                        final_color[i] = (linear_c.powf(1.0 / 2.2) * 255.0).round() as u8;
                    }
                }
                final_color[3] = (out_alpha * 255.0).round() as u8;
                tile.put_pixel(tx, ty, Rgba(final_color));
            }
        }
    }

    // 3. Texture Map: Iterate over the final image and sample from the tile
    let mut image: RgbaImage = ImageBuffer::from_pixel(width, height, bg_rgba);

    let rad = rotation_angle.to_radians();
    let cos_rad = rad.cos();
    let sin_rad = rad.sin();

    let center_x = width as f32 / 2.0;
    let center_y = height as f32 / 2.0;

    for y in 0..height {
        for x in 0..width {
            let dx = x as f32 - center_x;
            let dy = y as f32 - center_y;

            // Inverse rotation around center to find tile coordinates
            let u = dx * cos_rad + dy * sin_rad;
            let v = -dx * sin_rad + dy * cos_rad;

            let sampled_color = sample_tile(&tile, u, v);

            if sampled_color[3] > 0 {
                let bg_pixel = image.get_pixel_mut(x, y);
                blend_pixel(bg_pixel, &sampled_color, 1.0, &linear_table);
            }
        }
    }

    let mut buffer = Cursor::new(Vec::new());
    image.write_to(&mut buffer, image::ImageOutputFormat::Png)
        .map_err(|_| JsValue::from_str("Failed to encode image"))?;

    Ok(buffer.into_inner())
}
