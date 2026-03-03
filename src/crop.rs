use image::{RgbaImage, Rgba};
use std::collections::VecDeque;

/// Removes the background from an image by flood-filling from the corners.
/// Pixels similar to the corner colors (within `tolerance`) are made transparent.
/// Then the image is cropped to the bounding box of remaining opaque pixels.
pub fn remove_background(image_data: &[u8], tolerance: u8) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let img = image::load_from_memory(image_data)?;
    let mut rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();

    if width == 0 || height == 0 {
        return Err("Image has zero dimensions".into());
    }

    // Sample background color from the four corners
    let corners = [
        (0, 0),
        (width - 1, 0),
        (0, height - 1),
        (width - 1, height - 1),
    ];

    // Create a visited mask
    let mut visited = vec![false; (width * height) as usize];

    // Flood fill from each corner
    for &(cx, cy) in &corners {
        let bg_color = *rgba.get_pixel(cx, cy);
        // Skip if this corner is already transparent
        if bg_color[3] == 0 {
            continue;
        }
        flood_fill_transparent(&mut rgba, &mut visited, cx, cy, bg_color, tolerance);
    }

    // Find bounding box of remaining opaque pixels
    let mut min_x = width;
    let mut min_y = height;
    let mut max_x = 0u32;
    let mut max_y = 0u32;
    let mut has_opaque = false;

    for y in 0..height {
        for x in 0..width {
            let pixel = rgba.get_pixel(x, y);
            if pixel[3] > 0 {
                has_opaque = true;
                min_x = min_x.min(x);
                min_y = min_y.min(y);
                max_x = max_x.max(x);
                max_y = max_y.max(y);
            }
        }
    }

    // If no opaque pixels remain, return the original
    if !has_opaque {
        println!("Warning: Background removal made entire image transparent, returning original");
        let mut output = std::io::Cursor::new(Vec::new());
        let original = img.to_rgba8();
        original.write_to(&mut output, image::ImageOutputFormat::Png)?;
        return Ok(output.into_inner());
    }

    // Add a small padding (2px) around the crop
    let padding = 2u32;
    let crop_x = min_x.saturating_sub(padding);
    let crop_y = min_y.saturating_sub(padding);
    let crop_w = (max_x - min_x + 1 + padding * 2).min(width - crop_x);
    let crop_h = (max_y - min_y + 1 + padding * 2).min(height - crop_y);

    let cropped = image::imageops::crop_imm(&rgba, crop_x, crop_y, crop_w, crop_h).to_image();

    println!(
        "Background removed: {}x{} -> {}x{} (tolerance: {})",
        width, height, crop_w, crop_h, tolerance
    );

    let mut output = std::io::Cursor::new(Vec::new());
    cropped.write_to(&mut output, image::ImageOutputFormat::Png)?;
    Ok(output.into_inner())
}

/// Flood-fills from (start_x, start_y), making all connected pixels
/// that are similar to `bg_color` transparent.
fn flood_fill_transparent(
    img: &mut RgbaImage,
    visited: &mut [bool],
    start_x: u32,
    start_y: u32,
    bg_color: Rgba<u8>,
    tolerance: u8,
) {
    let (width, height) = img.dimensions();
    let mut queue = VecDeque::new();
    queue.push_back((start_x, start_y));

    while let Some((x, y)) = queue.pop_front() {
        let idx = (y * width + x) as usize;
        if visited[idx] {
            continue;
        }
        visited[idx] = true;

        let pixel = *img.get_pixel(x, y);
        if !colors_similar(pixel, bg_color, tolerance) {
            continue;
        }

        // Make this pixel transparent
        img.put_pixel(x, y, Rgba([0, 0, 0, 0]));

        // Add neighbors (4-connected)
        if x > 0 {
            queue.push_back((x - 1, y));
        }
        if x + 1 < width {
            queue.push_back((x + 1, y));
        }
        if y > 0 {
            queue.push_back((x, y - 1));
        }
        if y + 1 < height {
            queue.push_back((x, y + 1));
        }
    }
}

/// Checks if two colors are similar within a given tolerance per channel.
fn colors_similar(a: Rgba<u8>, b: Rgba<u8>, tolerance: u8) -> bool {
    let tol = tolerance as i16;
    (a[0] as i16 - b[0] as i16).abs() <= tol
        && (a[1] as i16 - b[1] as i16).abs() <= tol
        && (a[2] as i16 - b[2] as i16).abs() <= tol
}
