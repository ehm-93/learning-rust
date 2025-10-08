use rand::Rng;

pub fn flood_fill(map: &Vec<Vec<bool>>, start: (usize, usize)) -> Vec<(usize, usize)> {
    let width = map[0].len();
    let height = map.len();
    let mut filled = vec![vec![false; width]; height];
    let mut to_fill = vec![start];
    let mut result = Vec::new();

    while let Some((x, y)) = to_fill.pop() {
        if x >= width || y >= height || filled[y][x] || !map[y][x] {
            continue;
        }
        filled[y][x] = true;
        result.push((x, y));

        if x > 0 { to_fill.push((x - 1, y)); }
        if x < width - 1 { to_fill.push((x + 1, y)); }
        if y > 0 { to_fill.push((x, y - 1)); }
        if y < height - 1 { to_fill.push((x, y + 1)); }
    }

    result
}

pub fn flood_fill_bool(map: &Vec<Vec<bool>>, start: (usize, usize)) -> Vec<Vec<bool>> {
    let width = map[0].len();
    let height = map.len();
    let mut filled = vec![vec![false; width]; height];
    let mut to_fill = vec![start];

    while let Some((x, y)) = to_fill.pop() {
        if x >= width || y >= height || filled[y][x] || !map[y][x] {
            continue;
        }
        filled[y][x] = true;

        if x > 0 { to_fill.push((x - 1, y)); }
        if x < width - 1 { to_fill.push((x + 1, y)); }
        if y > 0 { to_fill.push((x, y - 1)); }
        if y < height - 1 { to_fill.push((x, y + 1)); }
    }

    filled
}

pub fn save_png(map: &Vec<Vec<bool>>, filename: &str) {
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
pub fn resize(map: &mut Vec<Vec<bool>>, new_width: usize, new_height: usize) {
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

pub fn random_walk_fill(
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

pub fn random_walk(
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

pub fn center(map: &Vec<Vec<bool>>) -> (usize, usize) {
    let width = map[0].len();
    let height = map.len();
    (width / 2, height / 2)
}

pub fn random(map: &Vec<Vec<bool>>, count: usize) -> Vec<(usize, usize)> {
    let mut rng = rand::rng();
    let width = map[0].len();
    let height = map.len();
    (0..count).map(|_| (rng.random_range(0..width), rng.random_range(0..height))).collect()
}

pub fn circle(map: &Vec<Vec<bool>>, thickness: usize, radius: usize, center: (usize, usize)) -> Vec<(usize, usize)> {
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

/// Draw a filled square of seeds centered on the passed point
pub fn square_fill(map: &mut Vec<Vec<bool>>, half_size: usize, center: (usize, usize)) {
    let width = map[0].len();
    let height = map.len();
    let center_x = center.0;
    let center_y = center.1;

    for y in center_y.saturating_sub(half_size)..=(center_y + half_size).min(height - 1) {
        for x in center_x.saturating_sub(half_size)..=(center_x + half_size).min(width - 1) {
            map[y][x] = true;
        }
    }
}

pub fn diamond(map: &Vec<Vec<bool>>, thickness: usize, half_size: usize, center: (usize, usize)) -> Vec<(usize, usize)> {
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

pub fn line(start: (usize, usize), end: (usize, usize), width: usize) -> Vec<(usize, usize)> {
    let mut points = Vec::new();
    let (mut x0, mut y0) = (start.0 as isize, start.1 as isize);
    let (x1, y1) = (end.0 as isize, end.1 as isize);
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;

    let half_width = width as isize / 2;

    loop {
        // Draw a square of the specified width centered on the current point
        for wx in -half_width..=half_width {
            for wy in -half_width..=half_width {
                points.push(((x0 + wx) as usize, (y0 + wy) as usize));
            }
        }

        if x0 == x1 && y0 == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x0 += sx;
        }
        if e2 <= dx {
            err += dx;
            y0 += sy;
        }
    }

    points
}

pub fn line_fill(map: &mut Vec<Vec<bool>>, start: (usize, usize), end: (usize, usize), width: usize) {
    let line_points = line(start, end, width);
    let height = map.len();
    let map_width = map[0].len();

    for (x, y) in line_points {
        if x < map_width && y < height {
            map[y][x] = true;
        }
    }
}
