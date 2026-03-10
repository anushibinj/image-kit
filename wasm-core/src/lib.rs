use wasm_bindgen::prelude::*;
use image::{ImageBuffer, RgbaImage, Rgba};
use ab_glyph::{FontRef, PxScale, Font, ScaleFont, point};
use std::io::Cursor;

const FONT_DATA: &[u8] = include_bytes!("OpenSans-Regular.ttf");

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

fn blend_pixel(background: &mut Rgba<u8>, color: &Rgba<u8>, alpha: f32) {
    let out_alpha = (color[3] as f32 / 255.0) * alpha;
    if out_alpha <= 0.0 { return; }

    let bg_alpha = background[3] as f32 / 255.0;

    let new_alpha = out_alpha + bg_alpha * (1.0 - out_alpha);
    if new_alpha <= 0.0 { return; }

    for i in 0..3 {
        let fg_c = color[i] as f32 / 255.0;
        let bg_c = background[i] as f32 / 255.0;
        let new_c = (fg_c * out_alpha + bg_c * bg_alpha * (1.0 - out_alpha)) / new_alpha;
        background[i] = (new_c * 255.0).round() as u8;
    }
    background[3] = (new_alpha * 255.0).round() as u8;
}

#[wasm_bindgen]
pub fn generate_text_pattern(
    text: String,
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

    let text_rgba = parse_hex_color(&text_color)?;
    let bg_rgba = parse_hex_color(&bg_color)?;

    let mut image: RgbaImage = ImageBuffer::from_pixel(width, height, bg_rgba);

    let font = FontRef::try_from_slice(FONT_DATA)
        .map_err(|_| JsValue::from_str("Failed to load font"))?;

    let scale = PxScale::from(font_size);
    let scaled_font = font.as_scaled(scale);

    let mut text_width = 0.0;
    let text_height = scaled_font.height();

    let mut glyphs = Vec::new();
    let mut current_x = 0.0;
    let baseline = scaled_font.ascent();

    for c in text.chars() {
        let glyph_id = font.glyph_id(c);
        let mut q_glyph = scaled_font.scaled_glyph(c);
        q_glyph.position = point(current_x, baseline);

        let advance = scaled_font.h_advance(glyph_id);

        glyphs.push((glyph_id, q_glyph));
        current_x += advance;
    }
    text_width = current_x;

    let rad = rotation_angle.to_radians();
    let cos_rad = rad.cos();
    let sin_rad = rad.sin();

    let diag = ((width as f32).hypot(height as f32)) as f32;
    let center_x = width as f32 / 2.0;
    let center_y = height as f32 / 2.0;

    let step_x = text_width + horizontal_spacing;
    let step_y = text_height + vertical_spacing;

    if step_x <= 0.0 || step_y <= 0.0 {
        return Err(JsValue::from_str("Invalid step size"));
    }

    let limit_x = (diag / step_x).ceil() as i32 + 2;
    let limit_y = (diag / step_y).ceil() as i32 + 2;

    for grid_y in -limit_y..=limit_y {
        for grid_x in -limit_x..=limit_x {
            let unrotated_start_x = grid_x as f32 * step_x - (text_width / 2.0);
            let unrotated_start_y = grid_y as f32 * step_y - (text_height / 2.0);

            let rotated_start_x = center_x + unrotated_start_x * cos_rad - unrotated_start_y * sin_rad;
            let rotated_start_y = center_y + unrotated_start_x * sin_rad + unrotated_start_y * cos_rad;

            for (_glyph_id, q_glyph) in &glyphs {
                let gx_offset = q_glyph.position.x;
                let gy_offset = q_glyph.position.y;

                let g_origin_x = rotated_start_x + gx_offset * cos_rad - gy_offset * sin_rad;
                let g_origin_y = rotated_start_y + gx_offset * sin_rad + gy_offset * cos_rad;

                if let Some(q_outline) = scaled_font.outline_glyph(q_glyph.clone()) {
                    let bb = q_outline.px_bounds();
                    q_outline.draw(|gx, gy, v| {
                        let px_offset_x = bb.min.x + gx as f32;
                        let px_offset_y = bb.min.y + gy as f32;

                        let pixel_x = g_origin_x + (px_offset_x - gx_offset) * cos_rad - (px_offset_y - gy_offset) * sin_rad;
                        let pixel_y = g_origin_y + (px_offset_x - gx_offset) * sin_rad + (px_offset_y - gy_offset) * cos_rad;

                        let px = pixel_x.round() as i32;
                        let py = pixel_y.round() as i32;

                        if px >= 0 && px < width as i32 && py >= 0 && py < height as i32 {
                            let pixel = image.get_pixel_mut(px as u32, py as u32);
                            blend_pixel(pixel, &text_rgba, v);
                        }
                    });
                }
            }
        }
    }

    let mut buffer = Cursor::new(Vec::new());
    image.write_to(&mut buffer, image::ImageOutputFormat::Png)
        .map_err(|_| JsValue::from_str("Failed to encode image"))?;

    Ok(buffer.into_inner())
}
