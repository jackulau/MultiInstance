//! Build script for MultiInstance
//!
//! Sets up Windows resources (icon, manifest) and macOS configuration

fn main() {
    // Windows-specific configuration
    #[cfg(windows)]
    {
        use std::path::Path;

        let res_path = Path::new("resources/windows/app.rc");
        let ico_path = Path::new("resources/windows/app.ico");

        // Generate icon if it doesn't exist
        if !ico_path.exists() {
            println!("cargo:warning=Generating Windows icon...");
            if let Err(e) = generate_icon(ico_path) {
                println!("cargo:warning=Failed to generate icon: {}", e);
            } else {
                println!("cargo:warning=Windows icon generated successfully");
            }
        }

        // Embed Windows resource file if it exists and the icon is present
        if res_path.exists() && ico_path.exists() {
            embed_resource::compile("resources/windows/app.rc", embed_resource::NONE);
            println!("cargo:warning=Embedding Windows resources with icon");
        }

        // Set Windows subsystem to prevent console window (MSVC linker syntax)
        #[cfg(target_env = "msvc")]
        {
            println!("cargo:rustc-link-arg=/SUBSYSTEM:WINDOWS");
            println!("cargo:rustc-link-arg=/ENTRY:mainCRTStartup");
        }
    }

    // macOS-specific configuration
    #[cfg(target_os = "macos")]
    {
        // Link against macOS frameworks
        println!("cargo:rustc-link-lib=framework=Cocoa");
        println!("cargo:rustc-link-lib=framework=Foundation");
        println!("cargo:rustc-link-lib=framework=CoreFoundation");
        println!("cargo:rustc-link-lib=framework=Security");
        println!("cargo:rustc-link-lib=framework=AppKit");
    }

    // Rerun if build.rs changes or resources change
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=resources/");
    println!("cargo:rerun-if-changed=resources/windows/app.rc");
    println!("cargo:rerun-if-changed=resources/windows/app.ico");
    println!("cargo:rerun-if-changed=resources/windows/app.manifest");
}

/// Generate a Windows ICO file with the app icon
#[cfg(windows)]
fn generate_icon(ico_path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    use image::ImageEncoder;
    use std::fs::File;
    use std::io::{BufWriter, Cursor, Write};

    // Icon sizes for ICO file (Windows standard sizes)
    let sizes: &[u32] = &[16, 32, 48, 256];

    // Generate PNG images for each size
    let mut images: Vec<Vec<u8>> = Vec::new();

    for &size in sizes {
        let img = generate_icon_image(size);
        let mut png_data: Vec<u8> = Vec::new();
        {
            let mut cursor = Cursor::new(&mut png_data);
            let encoder = image::codecs::png::PngEncoder::new(&mut cursor);
            encoder.write_image(img.as_raw(), size, size, image::ExtendedColorType::Rgba8)?;
        }
        images.push(png_data);
    }

    // Write ICO file
    let file = File::create(ico_path)?;
    let mut writer = BufWriter::new(file);

    // ICO header
    writer.write_all(&[0, 0])?; // Reserved
    writer.write_all(&[1, 0])?; // Type: 1 = ICO
    writer.write_all(&(sizes.len() as u16).to_le_bytes())?; // Number of images

    // Calculate offsets
    let header_size = 6 + (sizes.len() * 16);
    let mut offset = header_size;

    // Write directory entries
    for (i, &size) in sizes.iter().enumerate() {
        let width = if size >= 256 { 0u8 } else { size as u8 };
        let height = if size >= 256 { 0u8 } else { size as u8 };

        writer.write_all(&[width])?; // Width
        writer.write_all(&[height])?; // Height
        writer.write_all(&[0])?; // Color palette
        writer.write_all(&[0])?; // Reserved
        writer.write_all(&[1, 0])?; // Color planes
        writer.write_all(&[32, 0])?; // Bits per pixel
        writer.write_all(&(images[i].len() as u32).to_le_bytes())?; // Image size
        writer.write_all(&(offset as u32).to_le_bytes())?; // Offset

        offset += images[i].len();
    }

    // Write image data
    for png_data in &images {
        writer.write_all(png_data)?;
    }

    writer.flush()?;
    Ok(())
}

/// Generate a single icon image at the specified size
#[cfg(windows)]
fn generate_icon_image(size: u32) -> image::RgbaImage {
    use image::{Rgba, RgbaImage};

    let mut img = RgbaImage::new(size, size);
    let center = size as f32 / 2.0;
    let radius = center - 2.0;

    for y in 0..size {
        for x in 0..size {
            let dx = x as f32 - center;
            let dy = y as f32 - center;
            let dist = (dx * dx + dy * dy).sqrt();

            if dist < radius {
                // Inside the circle - blue gradient
                let t = dist / radius;

                // Primary blue color: #3B82F6 -> #1E40AF
                let r = (59.0 - t * 29.0) as u8; // 59 -> 30
                let g = (130.0 - t * 66.0) as u8; // 130 -> 64
                let b = (246.0 - t * 71.0) as u8; // 246 -> 175

                img.put_pixel(x, y, Rgba([r, g, b, 255]));
            } else if dist < radius + 1.5 {
                // Anti-aliased edge
                let alpha = ((radius + 1.5 - dist) / 1.5 * 255.0) as u8;
                img.put_pixel(x, y, Rgba([59, 130, 246, alpha]));
            } else {
                // Transparent
                img.put_pixel(x, y, Rgba([0, 0, 0, 0]));
            }
        }
    }

    // Draw hexagon shape in center
    draw_hexagon(&mut img, size);

    // Draw plus symbol in bottom-right
    draw_plus_symbol(&mut img, size);

    img
}

/// Draw a hexagon pattern in the center
#[cfg(windows)]
fn draw_hexagon(img: &mut image::RgbaImage, size: u32) {
    use image::Rgba;

    let center = size as f32 / 2.0;
    let hex_size = size as f32 * 0.25;

    // Draw hexagon outline (simplified)
    let white = Rgba([255, 255, 255, 200]);

    // Calculate hexagon vertices
    let vertices: Vec<(f32, f32)> = (0..6)
        .map(|i| {
            let angle = std::f32::consts::PI / 3.0 * i as f32 - std::f32::consts::PI / 2.0;
            (
                center + hex_size * angle.cos(),
                center * 0.85 + hex_size * angle.sin(),
            )
        })
        .collect();

    // Draw lines between vertices
    for i in 0..6 {
        let (x1, y1) = vertices[i];
        let (x2, y2) = vertices[(i + 1) % 6];
        draw_line(img, x1, y1, x2, y2, white);
    }

    // Draw center dot
    let dot_radius = (size as f32 * 0.05).max(2.0);
    for dy in -(dot_radius as i32)..=(dot_radius as i32) {
        for dx in -(dot_radius as i32)..=(dot_radius as i32) {
            let dist = ((dx * dx + dy * dy) as f32).sqrt();
            if dist <= dot_radius {
                let px = (center + dx as f32) as u32;
                let py = (center * 0.85 + dy as f32) as u32;
                if px < size && py < size {
                    let alpha = ((1.0 - dist / dot_radius) * 255.0) as u8;
                    img.put_pixel(px, py, Rgba([255, 255, 255, alpha.max(180)]));
                }
            }
        }
    }
}

/// Draw a plus symbol in the bottom-right corner
#[cfg(windows)]
fn draw_plus_symbol(img: &mut image::RgbaImage, size: u32) {
    use image::Rgba;

    let plus_center_x = size as f32 * 0.75;
    let plus_center_y = size as f32 * 0.75;
    let plus_radius = size as f32 * 0.12;
    let bar_width = (size as f32 * 0.04).max(2.0);
    let bar_length = plus_radius * 0.7;

    // Draw green circle background
    let green = Rgba([16, 185, 129, 255]); // #10B981
    for dy in -(plus_radius as i32)..=(plus_radius as i32) {
        for dx in -(plus_radius as i32)..=(plus_radius as i32) {
            let dist = ((dx * dx + dy * dy) as f32).sqrt();
            if dist <= plus_radius {
                let px = (plus_center_x + dx as f32) as u32;
                let py = (plus_center_y + dy as f32) as u32;
                if px < size && py < size {
                    img.put_pixel(px, py, green);
                }
            }
        }
    }

    // Draw white plus
    let white = Rgba([255, 255, 255, 255]);
    let half_width = bar_width / 2.0;

    // Horizontal bar
    for dy in -(half_width as i32)..=(half_width as i32) {
        for dx in -(bar_length as i32)..=(bar_length as i32) {
            let px = (plus_center_x + dx as f32) as u32;
            let py = (plus_center_y + dy as f32) as u32;
            if px < size && py < size {
                img.put_pixel(px, py, white);
            }
        }
    }

    // Vertical bar
    for dy in -(bar_length as i32)..=(bar_length as i32) {
        for dx in -(half_width as i32)..=(half_width as i32) {
            let px = (plus_center_x + dx as f32) as u32;
            let py = (plus_center_y + dy as f32) as u32;
            if px < size && py < size {
                img.put_pixel(px, py, white);
            }
        }
    }
}

/// Draw a line between two points
#[cfg(windows)]
fn draw_line(
    img: &mut image::RgbaImage,
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    color: image::Rgba<u8>,
) {
    let dx = x2 - x1;
    let dy = y2 - y1;
    let steps = dx.abs().max(dy.abs()) as i32;

    if steps == 0 {
        return;
    }

    let x_inc = dx / steps as f32;
    let y_inc = dy / steps as f32;

    let mut x = x1;
    let mut y = y1;

    let (width, height) = img.dimensions();

    for _ in 0..=steps {
        let px = x as u32;
        let py = y as u32;
        if px < width && py < height {
            img.put_pixel(px, py, color);
        }
        x += x_inc;
        y += y_inc;
    }
}
