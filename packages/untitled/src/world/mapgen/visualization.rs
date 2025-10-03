//! PNG visualization for macro maps
//!
//! Debug functionality to save macro maps as PNG images for visual inspection.

use bevy::prelude::*;
use image::{ImageBuffer, Rgb, RgbImage};
use super::{MacroMap, analyze_macro_map, Level};

/// Save a macro map as a PNG image for visualization
/// White = walls, Black = caves, Gray = spawn point
pub fn save_macro_map_png(macro_map: &MacroMap, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
    let width = macro_map.width as u32;
    let height = macro_map.height as u32;

    // Create image buffer
    let mut img: RgbImage = ImageBuffer::new(width, height);

    // Get spawn position for highlighting
    let (spawn_x, spawn_y) = macro_map.get_spawn_position();

    // Fill the image
    for y in 0..height {
        for x in 0..width {
            // Note: image coordinates are top-down, but our grid is bottom-up
            let grid_y = (height - 1 - y) as usize; // Flip Y coordinate
            let grid_x = x as usize;

            let pixel = if grid_x == spawn_x && grid_y == spawn_y {
                // Spawn point - bright green
                Rgb([0, 255, 0])
            } else if macro_map.grid[grid_y][grid_x] {
                // Wall - white
                Rgb([255, 255, 255])
            } else {
                // Cave - black
                Rgb([0, 0, 0])
            };

            img.put_pixel(x, y, pixel);
        }
    }

    // Save the image
    img.save(filename)?;
    info!("Saved macro map visualization to: {}", filename);

    Ok(())
}

/// Save a macro map with enhanced visualization showing connectivity
pub fn save_detailed_macro_map_png(macro_map: &MacroMap, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
    let width = macro_map.width as u32;
    let height = macro_map.height as u32;

    // Analyze the map first
    let stats = analyze_macro_map(&macro_map.grid);

    // Create image buffer (scale up 4x for better visibility)
    let scale = 4;
    let img_width = width * scale;
    let img_height = height * scale;
    let mut img: RgbImage = ImageBuffer::new(img_width, img_height);

    // Get spawn position
    let (spawn_x, spawn_y) = macro_map.get_spawn_position();

    // Fill the image
    for y in 0..height {
        for x in 0..width {
            // Note: image coordinates are top-down, but our grid is bottom-up
            let grid_y = (height - 1 - y) as usize; // Flip Y coordinate
            let grid_x = x as usize;

            let base_color = if grid_x == spawn_x && grid_y == spawn_y {
                // Spawn point - bright green
                Rgb([0, 255, 0])
            } else if macro_map.grid[grid_y][grid_x] {
                // Wall color based on position
                if grid_x == 0 || grid_x == macro_map.width - 1 ||
                   grid_y == 0 || grid_y == macro_map.height - 1 {
                    // Edge walls - red (unbreakable boundary)
                    Rgb([255, 100, 100])
                } else {
                    // Interior walls - light gray
                    Rgb([200, 200, 200])
                }
            } else {
                // Cave - dark blue
                Rgb([20, 20, 60])
            };

            // Fill the scaled up area
            for dy in 0..scale {
                for dx in 0..scale {
                    let img_x = x * scale + dx;
                    let img_y = y * scale + dy;
                    img.put_pixel(img_x, img_y, base_color);
                }
            }
        }
    }

    // Add grid lines for chunk boundaries (every 4 pixels since we scaled 4x)
    for y in (0..img_height).step_by(scale as usize) {
        for x in 0..img_width {
            if x < img_width {
                img.put_pixel(x, y, Rgb([128, 128, 128])); // Gray grid lines
            }
        }
    }
    for x in (0..img_width).step_by(scale as usize) {
        for y in 0..img_height {
            if y < img_height {
                img.put_pixel(x, y, Rgb([128, 128, 128])); // Gray grid lines
            }
        }
    }

    // Save the image
    img.save(filename)?;

    info!("Saved detailed macro map to: {}", filename);
    info!("Map stats: {} open cells ({:.1}%), {} wall cells",
          stats.open_cells, stats.open_ratio * 100.0, stats.wall_cells);
    info!("Boundary valid: {}, Edge holes: {}", stats.is_valid_boundary, stats.open_edge_cells);

    Ok(())
}

/// System to auto-save macro maps when levels are generated (debug feature)
pub fn auto_save_macro_maps(
    level: Res<Level>,
) {
    if !level.is_changed() {
        return;
    }

    // Create filename with level info
    let filename = format!("debug_macro_level_{}_seed_{}.png", level.depth, level.seed);
    let detailed_filename = format!("debug_macro_level_{}_seed_{}_detailed.png", level.depth, level.seed);

    // Save both versions
    if let Err(e) = save_macro_map_png(&level.macro_map, &filename) {
        warn!("Failed to save macro map PNG: {}", e);
    }

    if let Err(e) = save_detailed_macro_map_png(&level.macro_map, &detailed_filename) {
        warn!("Failed to save detailed macro map PNG: {}", e);
    }
}

/// System to save macro map when F6 is pressed (manual debug)
pub fn manual_save_macro_map(
    level: Option<Res<Level>>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::F6) {
        if let Some(level) = level {
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();

            let filename = format!("manual_macro_{}_{}.png", level.depth, timestamp);

            match save_detailed_macro_map_png(&level.macro_map, &filename) {
                Ok(()) => info!("Manually saved macro map (F6 pressed)"),
                Err(e) => warn!("Failed to manually save macro map: {}", e),
            }
        } else {
            warn!("No level loaded to save macro map");
        }
    }
}
