use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

use super::*;

/// Generate a freeform map using random walk and cellular automata
///
/// The map will consist of a large central room with random paths leading
/// outwards radially with some noise added to make it less uniform.
pub fn freeform(size: usize, seed: u64) -> Vec<Vec<bool>> {
    let mut rng = StdRng::seed_from_u64(seed);
    let width = 3 * size / 4;
    let height = width;
    let mut map = vec![vec![false; width]; height];
    let (cx, cy) = center(&map);

    // Spawn room
    square_fill(&mut map, 8, (cx, cy));

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
