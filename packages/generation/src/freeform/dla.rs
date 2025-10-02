use rand::Rng;
use std::collections::HashSet;

/// Perform diffusion-limited aggregation to grow structures on the map
///
/// # Parameters
/// - `map`: 2D boolean grid representing the map (true = occupied)
/// - `target_tiles`: Number of tiles to fill before stopping
/// - `max_particles`: Maximum number of particles to spawn
/// - `spawn_points`: List of (x, y) coordinates where particles can spawn
/// - `bias_points`: List of (x, y) coordinates that particles may be biased towards
/// - `bias_chance`: Probability (0.0 to 1.0) of biasing towards a bias point each step
/// - `progress_callback`: Function called after each tile is filled, with current count, target, and map
pub fn dla_generation<F>(
    map: &mut Vec<Vec<bool>>,
    target_tiles: usize,
    max_particles: usize,
    spawn_points: &Vec<(usize, usize)>,
    bias_points: &Vec<(usize, usize)>,
    bias_chance: f64,
    mut progress_callback: F,
) where F: FnMut(usize, usize, &Vec<Vec<bool>>),
{
    let width = map[0].len();
    let height = map.len();
    let mut occupied = HashSet::new();
    for y in 0..height {
        for x in 0..width {
            if map[y][x] {
                occupied.insert((x, y));
            }
        }
    }
    let seed_count = occupied.len();

    let mut rng = rand::rng();
    let mut particles_spawned = 0;

    while (occupied.len() - seed_count) < target_tiles && particles_spawned < max_particles {
        // Spawn particle near existing structure
        let spawn_idx = rng.random_range(0..spawn_points.len());
        let (mut x, mut y) = spawn_points[spawn_idx];

        // Random walk until stuck or out of bounds (scale with map size)
        let max_steps = (width * height / 100).max(10000);
        for _ in 0..max_steps {
            // Check if adjacent to occupied cell
            if is_adjacent_to_occupied(x, y, &occupied) {
                map[y][x] = true;
                occupied.insert((x, y));
                progress_callback(occupied.len(), target_tiles, &map);
                break;
            }

            if rng.random::<f64>() < bias_chance {
                let bias_idx = rng.random_range(0..bias_points.len());
                let (bias_x, bias_y) = bias_points[bias_idx];

                if bias_x > x && x < width - 1 {
                    x += 1;
                } else if bias_x < x && x > 0 {
                    x -= 1;
                }
                if bias_y > y && y < height - 1 {
                    y += 1;
                } else if bias_y < y && y > 0 {
                    y -= 1;
                }
            }
            // Else random step
            else {
                let dir = rng.random_range(0..4);
                match dir {
                    0 if x > 0 => x -= 1,
                    1 if x < width - 1 => x += 1,
                    2 if y > 0 => y -= 1,
                    3 if y < height - 1 => y += 1,
                    _ => continue,
                }
            }
        }

        particles_spawned += 1;
    }
}

fn is_adjacent_to_occupied(x: usize, y: usize, occupied: &HashSet<(usize, usize)>) -> bool {
    for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
        let nx = x as i32 + dx;
        let ny = y as i32 + dy;
        if nx >= 0 && ny >= 0 && occupied.contains(&(nx as usize, ny as usize)) {
            return true;
        }
    }
    false
}
