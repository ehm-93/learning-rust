pub mod ca;
pub mod dla;
pub mod simplex;

use image::{ImageBuffer, Rgb};
use super::post;

use rand::Rng;

pub fn freeform(outfile: &str, floodfile: &str) {
    let width = 768;
    let height = width;
    let mut map = vec![vec![false; width]; height];

    square_fill_seeds(&mut map, 8);

    let mut rng = rand::rng();

    // cardinal edges
    let (cx, cy) = center(&map);
    let cardinal_points = [
        (cx, 0),
        (cx, height - 1),
        (0, cy),
        (width - 1, cy),
    ];
    for &t in &cardinal_points {
        random_walk_fill(
            &mut map,
            &(cx, cy),
            &t,
            0.08,
            width,
            2,
            1.0,
        );
    }

    let seed_points= random(&map, 16);
    dla::dla_generation(
        &mut map,
        5000,
        10_000,
        &seed_points,
        &vec![(cx, cy)],
        0.02,
        |current, _, map| {
            if current % 500 == 0 {
                save_png(outfile, &map).unwrap();
            }
         }
    );

    let cardinal_halfway = [
        (cx, height / 4),
        (3 * width / 4, cy),
        (cx, 3 * height / 4),
        (width / 4, cy),
    ];
    for i in 0..cardinal_halfway.len() {
        random_walk_fill(
            &mut map,
            &cardinal_halfway[i],
            &cardinal_halfway[(i + 1) % cardinal_halfway.len()],
            0.08,
            width,
            2,
            1.0,
        );
    }

    dla::dla_generation(
        &mut map,
        5000,
        10_000,
        &seed_points,
        &vec![(cx, cy)],
        0.02,
        |current, _, map| {
            if current % 500 == 0 {
                save_png(outfile, &map).unwrap();
            }
         }
    );

    ca::cellular_automata(&mut map, 2, |_, _| { });

    let external_centers = [
        (width / 8, height / 8),
        (7 * width / 8, height / 8),
        (7 * width / 8, 7 * height / 8),
        (width / 8, 7 * height / 8),
    ];
    for &center in &external_centers {
        let n_w = width / 8;
        let n_h = height / 8;
        let n_area = n_w * n_h;
        map[center.1][center.0] = true;

        let radius = width / 8;
        let circle = diamond(&map, 2, radius, center);
        dla::dla_generation(
            &mut map,
            n_area / 4,
            n_area,
            &circle,
            &vec![center],
            0.02,
            |current, _, map| {
                if current % 500 == 0 {
                    save_png(outfile, &map).unwrap();
                }
             }
        );
    }

    resize(&mut map, 1024, 1024);
    let (cx, cy) = center(&map);

    simplex::generate_simplex_noise(&mut map, &mut rng, 0.05, 0.9);
    simplex::generate_simplex_noise(&mut map, &mut rng, 0.1, 0.9);
    simplex::generate_simplex_noise(&mut map, &mut rng, 0.25, 0.9);

    ca::cellular_automata(&mut map, 3, |_, _| { });

    save_png(outfile, &map).unwrap();

    let flooded = post::flood::flood_fill(&mut map, (cx, cy));
    post::flood::write_choropleth(&flooded, floodfile).unwrap();
}

fn save_png(path: &str, map: &Vec<Vec<bool>>) -> Result<(), std::io::Error> {
    let height = map.len();
    let width = map[0].len();
    let mut imgbuf: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(width as u32, height as u32);

    for (y, row) in map.iter().enumerate() {
        for (x, &value) in row.iter().enumerate() {
            if value {
                imgbuf.put_pixel(x as u32, y as u32, Rgb([255, 255, 255]));
            } else {
                imgbuf.put_pixel(x as u32, y as u32, Rgb([0, 0, 0]));
            }
        }
    }

    imgbuf.save(path)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
}

/// Crop the map to the specified rectangle
fn resize(map: &mut Vec<Vec<bool>>, new_width: usize, new_height: usize) {
    let old_width = map[0].len();
    let old_height = map.len();

    let offset_x = (new_width.saturating_sub(old_width)) / 2;
    let offset_y = (new_height.saturating_sub(old_height)) / 2;

    let mut new_map = vec![vec![false; new_width]; new_height];

    for y in 0..old_height.min(new_height) {
        for x in 0..old_width.min(new_width) {
            new_map[y + offset_y][x + offset_x] = map[y][x];
        }
    }

    *map = new_map;
}

fn fill(map: &mut Vec<Vec<bool>>, points: &Vec<(usize, usize)>) {
    for (x, y) in points {
        map[*y][*x] = true;
    }
}

/// Bresenham's line algorithm to get all points between two coordinates
fn line(map: &Vec<Vec<bool>>, a: (usize, usize), b: (usize, usize)) -> Vec<(usize, usize)> {
    let (x0, y0) = (a.0 as isize, a.1 as isize);
    let (x1, y1) = (b.0 as isize, b.1 as isize);
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    let mut points = Vec::new();

    let mut x = x0;
    let mut y = y0;

    loop {
        if x >= 0 && y >= 0 && (x as usize) < map[0].len() && (y as usize) < map.len() {
            points.push((x as usize, y as usize));
        }
        if x == x1 && y == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x += sx;
        }
        if e2 <= dx {
            err += dx;
            y += sy;
        }
    }

    points
}

fn random_walk_fill(
    map: &mut Vec<Vec<bool>>,
    spawn_point: &(usize, usize),
    bias_point: &(usize, usize),
    bias_chance: f64,
    path_length: usize,
    path_thickness: usize,
    steer_aggression: f64,
) {
    let path = random_walk(map, spawn_point, bias_point, bias_chance, path_length, path_thickness, steer_aggression);
    fill(map, &path);
}

fn random_walk(
    map: &Vec<Vec<bool>>,
    spawn_point: &(usize, usize),
    bias_point: &(usize, usize),
    bias_chance: f64,
    path_length: usize,
    path_thickness: usize,
    steer_aggression: f64,
) -> Vec<(usize, usize)> {
    let width = map[0].len();
    let height = map.len();
    let mut rng = rand::rng();
    let (mut x, mut y) = *spawn_point;
    let mut path = Vec::new();

    // Initialize direction towards the bias point
    let dx = bias_point.0 as f64 - x as f64;
    let dy = bias_point.1 as f64 - y as f64;
    let mut direction = dy.atan2(dx);

    for _ in 0..path_length {
        // Draw thickness
        for tx in x.saturating_sub(path_thickness / 2)..=(x + path_thickness / 2).min(width - 1) {
            for ty in y.saturating_sub(path_thickness / 2)..=(y + path_thickness / 2).min(height - 1) {
                path.push((tx, ty));
            }
        }

        if rng.random::<f64>() < bias_chance {
            // Bias towards the target point - immediately set direction
            let dx = bias_point.0 as f64 - x as f64;
            let dy = bias_point.1 as f64 - y as f64;
            direction = dy.atan2(dx);
        } else {
            // Random steering: adjust direction by a small random amount
            let steering_amount = rng.random_range(-steer_aggression..steer_aggression);
            direction += steering_amount;
        }

        // Normalize direction to [0, 2π)
        direction = direction % std::f64::consts::TAU;
        if direction < 0.0 {
            direction += std::f64::consts::TAU;
        }

        // Calculate next position based on direction
        let next_x = x as f64 + direction.cos();
        let next_y = y as f64 + direction.sin();

        // Convert to grid coordinates with bounds checking
        let new_x = (next_x.round() as isize).clamp(0, width as isize - 1) as usize;
        let new_y = (next_y.round() as isize).clamp(0, height as isize - 1) as usize;

        x = new_x;
        y = new_y;
    }

    path
}

/// Place a single seed in the center of the map
fn center_seed(map: &mut Vec<Vec<bool>>) {
    let (cx, cy) = center(map);
}

fn center(map: &Vec<Vec<bool>>) -> (usize, usize) {
    let width = map[0].len();
    let height = map.len();
    (width / 2, height / 2)
}

/// Generate `count` random seeds within the map bounds
fn random_seeds(map: &mut Vec<Vec<bool>>, count: usize) {
    for (dx, dy) in random(map, count) {
        map[dy][dx] = true;
    }
}

fn random(map: &Vec<Vec<bool>>, count: usize) -> Vec<(usize, usize)> {
    let mut rng = rand::rng();
    let width = map[0].len();
    let height = map.len();
    (0..count).map(|_| (rng.random_range(0..width), rng.random_range(0..height))).collect()
}

/// Generate seeds in a regular grid pattern
fn regular_seeds(map: &mut Vec<Vec<bool>>, spacing: usize) {
    for (dx, dy) in regular(map, spacing) {
        map[dy][dx] = true;
    }
}

fn regular(map: &Vec<Vec<bool>>, spacing: usize) -> Vec<(usize, usize)> {
    let width = map[0].len();
    let height = map.len();
    let mut seeds = Vec::new();
    for i in (0..width).step_by(spacing) {
        for j in (0..height).step_by(spacing) {
            seeds.push((i, j));
        }
    }
    seeds
}

/// Generate seeds in a ring pattern around the center
fn ring_seeds(map: &mut Vec<Vec<bool>>, count: usize, radius: usize) {
    for (dx, dy) in ring(map, count, radius) {
        map[dy][dx] = true;
    }
}

fn ring(map: &Vec<Vec<bool>>, count: usize, radius: usize) -> Vec<(usize, usize)> {
    let width = map[0].len();
    let height = map.len();
    let center_x = width / 2;
    let center_y = height / 2;
    let mut seeds = Vec::new();

    for i in 0..count {
        let angle = (i as f64 / count as f64) * std::f64::consts::TAU;
        let x = (center_x as f64 + angle.cos() * radius as f64) as usize;
        let y = (center_y as f64 + angle.sin() * radius as f64) as usize;
        seeds.push((x, y));
    }

    seeds
}

/// Generate clusters of seeds scattered across the map
fn cluster_seeds(map: &mut Vec<Vec<bool>>, cluster_count: usize, cluster_size: usize, cluster_radius: usize) {
    for (dx, dy) in cluster(map, cluster_count, cluster_size, cluster_radius) {
        map[dy][dx] = true;
    }
}

fn cluster(map: &Vec<Vec<bool>>, cluster_count: usize, cluster_size: usize, cluster_radius: usize) -> Vec<(usize, usize)> {
    let mut rng = rand::rng();
    let width = map[0].len();
    let height = map.len();
    let mut seeds = Vec::new();

    for _ in 0..cluster_count {
        let center_x = rng.random_range(cluster_radius..(width - cluster_radius));
        let center_y = rng.random_range(cluster_radius..(height - cluster_radius));

        for _ in 0..cluster_size {
            let angle = rng.random_range(0.0..std::f64::consts::TAU);
            let radius = rng.random_range(0..cluster_radius) as f64;
            let x = (center_x as f64 + angle.cos() * radius).clamp(0.0, (width - 1) as f64) as usize;
            let y = (center_y as f64 + angle.sin() * radius).clamp(0.0, (height - 1) as f64) as usize;
            seeds.push((x, y));
        }
    }

    seeds
}

/// Draw a filled circle of seeds centered in the map with the given radius
fn circle_fill_seeds(map: &mut Vec<Vec<bool>>, radius: usize) {
    let width = map[0].len();
    let height = map.len();
    let center_x = width as f64 / 2.0;
    let center_y = height as f64 / 2.0;

    for y in 0..height {
        for x in 0..width {
            let dx = x as f64 - center_x;
            let dy = y as f64 - center_y;
            if (dx * dx + dy * dy).sqrt() <= radius as f64 {
                map[y][x] = true;
            }
        }
    }
}

/// Draw a contiguous, hollow circle of seeds centered in the map with the given thickness
fn circle_seeds(map: &mut Vec<Vec<bool>>, thickness: usize, radius: usize) {
    for (dx, dy) in circle(map, thickness, radius, (map[0].len() / 2, map.len() / 2)) {
        map[dy][dx] = true;
    }
}

fn circle(map: &Vec<Vec<bool>>, thickness: usize, radius: usize, center: (usize, usize)) -> Vec<(usize, usize)> {
    let width = map[0].len();
    let height = map.len();
    let center_x = center.0 as f64;
    let center_y = center.1 as f64;
    let mut seeds = Vec::new();

    for t in 0..thickness {
        let r = radius as f64 + t as f64 - (thickness as f64 / 2.0);
        let steps = (2.0 * std::f64::consts::PI * r).ceil() as usize;

        for i in 0..steps {
            let angle = 2.0 * std::f64::consts::PI * i as f64 / steps as f64;
            let x = (center_x + angle.cos() * r).round() as isize;
            let y = (center_y + angle.sin() * r).round() as isize;

            if x >= 0 && x < width as isize && y >= 0 && y < height as isize {
                seeds.push((x as usize, y as usize));
            }
        }
    }

    seeds
}

/// Draw a filled square of seeds centered on the map with the given half-size
fn square_fill_seeds(map: &mut Vec<Vec<bool>>, half_size: usize) {
    let width = map[0].len();
    let height = map.len();
    let center_x = width / 2;
    let center_y = height / 2;

    for y in center_y.saturating_sub(half_size)..=(center_y + half_size).min(height - 1) {
        for x in center_x.saturating_sub(half_size)..=(center_x + half_size).min(width - 1) {
            map[y][x] = true;
        }
    }
}

/// Draw a cross pattern of seeds centered in the map
fn cross_seeds(map: &mut Vec<Vec<bool>>, thickness: usize, length: usize) {
    let width = map[0].len();
    let height = map.len();
    let center_x = width / 2;
    let center_y = height / 2;

    for t in 0..thickness {
        let offset = t as isize - (thickness as isize / 2);
        for l in 0..length {
            if center_x + l < width {
                map[center_y.wrapping_add(offset as usize)][center_x + l] = true;
            }
            if center_x >= l {
                map[center_y.wrapping_add(offset as usize)][center_x - l] = true;
            }
            if center_y + l < height {
                map[center_y + l][center_x.wrapping_add(offset as usize)] = true;
            }
            if center_y >= l {
                map[center_y - l][center_x.wrapping_add(offset as usize)] = true;
            }
        }
    }
}

fn diamond_seeds(map: &mut Vec<Vec<bool>>, thickness: usize, half_size: usize) {
    // Draw hollow diamond by only drawing points at specific Manhattan distances
    for (dx, dy) in diamond(map, thickness, half_size, (map[0].len() / 2, map.len() / 2)) {
        map[dy][dx] = true;
    }
}

fn diamond(map: &Vec<Vec<bool>>, thickness: usize, half_size: usize, center: (usize, usize)) -> Vec<(usize, usize)> {
    let width = map[0].len();
    let height = map.len();
    let center_x = center.0;
    let center_y = center.1;
    let mut seeds = Vec::new();

    // Draw hollow diamond by only drawing points at specific Manhattan distances
    for t in 0..thickness {
        let target_distance = (half_size as isize + t as isize).max(0) as usize;

        // Iterate through all points in the map
        for y in 0..height {
            for x in 0..width {
                // Calculate Manhattan distance from center
                let manhattan_distance =
                    ((x as isize - center_x as isize).abs() +
                     (y as isize - center_y as isize).abs()) as usize;

                // Only draw points that are exactly at the target distance
                if manhattan_distance == target_distance {
                    seeds.push((x, y));
                }
            }
        }
    }
    seeds
}
