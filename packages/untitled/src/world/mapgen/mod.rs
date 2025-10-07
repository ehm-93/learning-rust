pub mod ca;
pub mod simplex;

use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

pub fn freeform(size: usize, seed: u64) -> Vec<Vec<bool>> {
    let mut rng = StdRng::seed_from_u64(seed);
    let width = 3 * size / 4;
    let height = width;
    let mut map = vec![vec![false; width]; height];
    let (cx, cy) = center(&map);

    // Spawn room
    square_fill(&mut map, 8);

    // Add some random paths
    let mut path_count = (size as i32) / 256;
    path_count = rng.random_range((path_count - 2)..(path_count + 2)).max(4).min(32);
    let path_points = circle(&map, 1, width / 2, (cx, cy));
    let mut a = path_points[rng.random_range(0..path_points.len())];
    for _ in 0..path_count {
        let last_a = a;
        // make sure the new point is at least tau / path_count radians from the previous
        // prevents paths from overlapping too much
        loop {
            a = path_points[rng.random_range(0..path_points.len())];
            let angle_a = ((a.1 as isize - cy as isize) as f64).atan2((a.0 as isize - cx as isize) as f64) + std::f64::consts::TAU;
            let angle_a = angle_a % std::f64::consts::TAU;
            let angle_last_a = ((last_a.1 as isize - cy as isize) as f64).atan2((last_a.0 as isize - cx as isize) as f64) + std::f64::consts::TAU;
            let angle_last_a = angle_last_a % std::f64::consts::TAU;
            let angle_diff = (angle_a - angle_last_a).abs();
            if angle_diff >= (std::f64::consts::TAU / path_count as f64) {
                break;
            }
        }
        random_walk_fill(
            &mut map,
            &mut rng,
            &(cx, cy),
            &a,
            0.02,
            (((a.0 as isize - cx as isize).abs() + (a.1 as isize - cy as isize).abs()) as f64 * 1.5) as usize,
            4,
            0.3,
        );
    }

    // scale out to guarantee solid boarder
    resize(&mut map, size, size);

    // Add some noise to make it less uniform
    simplex::generate_simplex_noise(&mut map, &mut rng, 0.045, 0.8);
    simplex::generate_simplex_noise(&mut map, &mut rng, 0.1, 0.7);

    // Smoothing
    ca::cellular_automata(&mut map, 5, |_, _| { });

    save_png(&map, "out/macro_map.png");

    map
}

fn save_png(map: &Vec<Vec<bool>>, filename: &str) {
    let width = map[0].len() as u32;
    let height = map.len() as u32;
    let mut imgbuf: image::ImageBuffer<image::Rgb<u8>, Vec<u8>> = image::ImageBuffer::new(width, height);

    for (y, row) in map.iter().enumerate() {
        for (x, &cell) in row.iter().enumerate() {
            let pixel = if cell { image::Rgb([255, 255, 255]) } else { image::Rgb([0, 0, 0]) };
            imgbuf.put_pixel(x as u32, y as u32, pixel);
        }
    }

    imgbuf.save(filename).unwrap();
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

fn random_walk_fill(
    map: &mut Vec<Vec<bool>>,
    rng: &mut impl Rng,
    spawn_point: &(usize, usize),
    bias_point: &(usize, usize),
    bias_chance: f64,
    path_length: usize,
    path_thickness: usize,
    steer_aggression: f64,
) {
    let path = random_walk(map, rng, spawn_point, bias_point, bias_chance, path_length, path_thickness, steer_aggression);
    for (x, y) in path {
        map[y][x] = true;
    }
}

fn random_walk(
    map: &Vec<Vec<bool>>,
    rng: &mut impl Rng,
    spawn_point: &(usize, usize),
    bias_point: &(usize, usize),
    bias_chance: f64,
    path_length: usize,
    path_thickness: usize,
    steer_aggression: f64,
) -> Vec<(usize, usize)> {
    let width = map[0].len();
    let height = map.len();
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

fn center(map: &Vec<Vec<bool>>) -> (usize, usize) {
    let width = map[0].len();
    let height = map.len();
    (width / 2, height / 2)
}

fn random(map: &Vec<Vec<bool>>, count: usize) -> Vec<(usize, usize)> {
    let mut rng = rand::rng();
    let width = map[0].len();
    let height = map.len();
    (0..count).map(|_| (rng.random_range(0..width), rng.random_range(0..height))).collect()
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
fn square_fill(map: &mut Vec<Vec<bool>>, half_size: usize) {
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
