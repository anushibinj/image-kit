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

// Improved blending with gamma correction for better antialiasing
fn blend_pixel(background: &mut Rgba<u8>, color: &Rgba<u8>, alpha: f32) {
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

    // Convert to linear space for better blending (approximate gamma 2.2)
    for i in 0..3 {
        let fg_c = (color[i] as f32 / 255.0).powf(2.2);
        let bg_c = (background[i] as f32 / 255.0).powf(2.2);
        let new_c = (fg_c * out_alpha + bg_c * bg_alpha * (1.0 - out_alpha)) / new_alpha;
        // Convert back to sRGB space
        background[i] = (new_c.powf(1.0 / 2.2) * 255.0).round() as u8;
    }
    background[3] = (new_alpha * 255.0).round() as u8;
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

    let text_rgba = parse_hex_color(&text_color)?;
    let bg_rgba = parse_hex_color(&bg_color)?;

    let mut image: RgbaImage = ImageBuffer::from_pixel(width, height, bg_rgba);

    let font_data = match font_family.as_str() {
        "Roboto" => FONT_ROBOTO,
        "Lora" => FONT_LORA,
        "Pacifico" => FONT_PACIFICO,
        _ => FONT_OPEN_SANS,
    };

    let font = FontRef::try_from_slice(font_data)
        .map_err(|_| JsValue::from_str("Failed to load font"))?;

    // To improve quality, render at higher resolution and then scale down (supersampling/multisampling)
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

    let rad = rotation_angle.to_radians();
    let cos_rad = rad.cos();
    let sin_rad = rad.sin();

    // We are working in supersampled space
    let render_width = width as f32 * supersample_factor;
    let render_height = height as f32 * supersample_factor;

    let diag = render_width.hypot(render_height);
    let center_x = render_width / 2.0;
    let center_y = render_height / 2.0;

    let step_x = text_width + (horizontal_spacing * supersample_factor);
    let step_y = text_height + (vertical_spacing * supersample_factor);

    if step_x <= 0.0 || step_y <= 0.0 {
        return Err(JsValue::from_str("Invalid step size"));
    }

    let limit_x = (diag / step_x).ceil() as i32 + 2;
    let limit_y = (diag / step_y).ceil() as i32 + 2;

    let mut render_buf = RgbaImage::new((width as f32 * supersample_factor).ceil() as u32, (height as f32 * supersample_factor).ceil() as u32);
    // Initialize with transparent
    for p in render_buf.pixels_mut() {
        *p = Rgba([0, 0, 0, 0]);
    }

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

                if let Some(q_outline) = render_font.outline_glyph(q_glyph.clone()) {
                    let bb = q_outline.px_bounds();
                    q_outline.draw(|gx, gy, v| {
                        // Apply gamma correction to coverage value for better text rendering
                        let coverage = v.powf(1.0 / 2.2);

                        let px_offset_x = bb.min.x + gx as f32;
                        let px_offset_y = bb.min.y + gy as f32;

                        let pixel_x = g_origin_x + (px_offset_x - gx_offset) * cos_rad - (px_offset_y - gy_offset) * sin_rad;
                        let pixel_y = g_origin_y + (px_offset_x - gx_offset) * sin_rad + (px_offset_y - gy_offset) * cos_rad;

                        let px = pixel_x.round() as i32;
                        let py = pixel_y.round() as i32;

                        if px >= 0 && px < render_buf.width() as i32 && py >= 0 && py < render_buf.height() as i32 {
                            let pixel = render_buf.get_pixel_mut(px as u32, py as u32);
                            blend_pixel(pixel, &text_rgba, coverage);
                        }
                    });
                }
            }
        }
    }

    // Downsample the buffer onto the final image
    for y in 0..height {
        for x in 0..width {
            let mut sums = [0.0f32; 4];
            let mut count = 0.0;

            let start_x = x as f32 * supersample_factor;
            let start_y = y as f32 * supersample_factor;

            for dy in 0..(supersample_factor as u32) {
                for dx in 0..(supersample_factor as u32) {
                    let rx = start_x as u32 + dx;
                    let ry = start_y as u32 + dy;

                    if rx < render_buf.width() && ry < render_buf.height() {
                        let p = render_buf.get_pixel(rx, ry);
                        // Convert to linear space before downsampling
                        for i in 0..3 {
                            sums[i] += (p[i] as f32 / 255.0).powf(2.2);
                        }
                        sums[3] += p[3] as f32 / 255.0;
                        count += 1.0;
                    }
                }
            }

            if count > 0.0 {
                let out_alpha = sums[3] / count;
                if out_alpha > 0.0 {
                    let mut final_color = [0u8; 4];
                    for i in 0..3 {
                        let linear_c = sums[i] / count;
                        // Convert back to sRGB
                        final_color[i] = (linear_c.powf(1.0 / 2.2) * 255.0).round() as u8;
                    }
                    final_color[3] = (out_alpha * 255.0).round() as u8;

                    let bg_pixel = image.get_pixel_mut(x, y);
                    blend_pixel(bg_pixel, &Rgba(final_color), 1.0);
                }
            }
        }
    }

    let mut buffer = Cursor::new(Vec::new());
    image.write_to(&mut buffer, image::ImageOutputFormat::Png)
        .map_err(|_| JsValue::from_str("Failed to encode image"))?;

    Ok(buffer.into_inner())
}
