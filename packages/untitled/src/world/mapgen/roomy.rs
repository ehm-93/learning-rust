use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use rand::seq::SliceRandom;

use super::*;

const ROOM_COUNT_DIVISOR: i32 = 48;
const ROOM_COUNT_RANGE: i32 = 8;
const ROOM_COUNT_MIN: i32 = 8;
const ROOM_COUNT_MAX: i32 = 128;
const MIN_ROOM_SIZE: usize = 8;
const MAX_ROOM_SIZE: usize = 24;
const MIN_CORRIDOR_WIDTH: usize = 2;
const MAX_CORRIDOR_WIDTH: usize = 4;
const WALK_BIAS_CHANCE: f64 = 0.04;
const WALK_LEN_COEFF: f64 = 1.5;
const WALK_STEER_STRENGTH: f64 = 0.25;
const MAX_LOOPS: i32 = 8;

/// Generate a roomy map
///
/// The map will consist of a central spawn room and many rooms of
/// various sizes connected by wide corridors.
pub fn roomy(size: usize, seed: u64) -> Vec<Vec<bool>> {
    let mut rng = StdRng::seed_from_u64(seed);
    let width = size / 2;
    let height = width;
    let mut map = vec![vec![false; width]; height];
    let (cx, cy) = center(&map);

    // Spawn room
    square_fill(&mut map, 8, (cx, cy));

    // Add some random rooms
    let mut room_count = (size as i32) / ROOM_COUNT_DIVISOR;
    room_count = rng.random_range(
        (room_count - ROOM_COUNT_RANGE / 2)..(room_count + ROOM_COUNT_RANGE / 2)
    )
        .max(ROOM_COUNT_MIN)
        .min(ROOM_COUNT_MAX);
    let room_positions = random(&map, room_count as usize);
    for room_pos in room_positions.iter() {
        let room_size = rng.random_range(MIN_ROOM_SIZE..MAX_ROOM_SIZE);
        square_fill(&mut map, room_size / 2, *room_pos);
    }

    // create a minimum spanning tree to connect all rooms including the spawn room
    // and fill in corridors
    let mut all_positions = vec![(cx, cy)]; // start with center spawn room
    all_positions.extend(room_positions.iter());
    let total_rooms = all_positions.len();

    let mut graph = vec![vec![false; total_rooms]; total_rooms];
    let mut connected = vec![false; total_rooms];
    connected[0] = true; // start with the center spawn room
    let mut edges = Vec::new();
    while connected.iter().any(|&c| !c) {
        let mut possible_edges = Vec::new();
        for i in 0..total_rooms {
            if connected[i] {
                for j in 0..total_rooms {
                    if !connected[j] {
                        let dist = (((all_positions[i].0 as isize - all_positions[j].0 as isize).pow(2)
                            + (all_positions[i].1 as isize - all_positions[j].1 as isize).pow(2)) as f64).sqrt() * 1.1;
                        possible_edges.push((dist, i, j));
                    }
                }
            }
        }
        if let Some(&(_, i, j)) = possible_edges.iter().min_by(|a, b| a.0.partial_cmp(&b.0).unwrap()) {
            graph[i][j] = true;
            graph[j][i] = true;
            connected[j] = true;
            edges.push((i, j));
        } else {
            break; // no more edges to add
        }
    }

    // create corridors
    for (i, j) in edges {
        let corridor_width = rng.random_range(MIN_CORRIDOR_WIDTH..MAX_CORRIDOR_WIDTH);
        let dist = (((all_positions[i].0 as isize - all_positions[j].0 as isize).pow(2)
            + (all_positions[i].1 as isize - all_positions[j].1 as isize).pow(2)) as f64).sqrt();
        random_walk_fill(
            &mut map,
            &mut rng,
            &all_positions[i],
            &all_positions[j],
            WALK_BIAS_CHANCE,
            (dist * WALK_LEN_COEFF) as usize,
            corridor_width,
            WALK_STEER_STRENGTH,
        );
    }

    // add some loops for flavor
    // For each node in a random set, try to connect to closest nodes that don't create intersecting edges
    let extra_edge_attempts = (room_count / 4).max(MAX_LOOPS) as usize;
    let mut random_nodes: Vec<usize> = (0..total_rooms).collect();
    random_nodes.shuffle(&mut rng);
    random_nodes.truncate(extra_edge_attempts);

    let max_distance = (size / 4) as f64;
    let max_distance_sq = max_distance * max_distance;

    for &a in &random_nodes {
        // Sort other nodes by distance to A (largest to smallest for bigger loops)
        // Filter to only nodes within size/4 distance
        let mut candidates: Vec<(f64, usize)> = (0..total_rooms)
            .filter(|&b| b != a && !graph[a][b])
            .map(|b| {
                let dist_sq = ((all_positions[a].0 as isize - all_positions[b].0 as isize).pow(2)
                    + (all_positions[a].1 as isize - all_positions[b].1 as isize).pow(2)) as f64;
                (dist_sq, b)
            })
            .filter(|(dist_sq, _)| *dist_sq <= max_distance_sq)
            .collect();
        candidates.sort_by(|x, y| y.0.partial_cmp(&x.0).unwrap()); // Reversed for largest first

        // Try to add edge to farthest node that doesn't intersect existing edges
        for (_dist, b) in candidates {
            let mut intersects = false;

            // Check if edge AB intersects any existing edge in graph
            for i in 0..total_rooms {
                for j in (i+1)..total_rooms {
                    if graph[i][j] && edges_intersect(
                        all_positions[a], all_positions[b],
                        all_positions[i], all_positions[j]
                    ) {
                        intersects = true;
                        break;
                    }
                }
                if intersects { break; }
            }

            if !intersects {
                let corridor_width = rng.random_range(MIN_CORRIDOR_WIDTH..MAX_CORRIDOR_WIDTH);
                let dist = (((all_positions[a].0 as isize - all_positions[b].0 as isize).pow(2)
                    + (all_positions[a].1 as isize - all_positions[b].1 as isize).pow(2)) as f64).sqrt();
                random_walk_fill(
                    &mut map,
                    &mut rng,
                    &all_positions[a],
                    &all_positions[b],
                    WALK_BIAS_CHANCE,
                    (dist * WALK_LEN_COEFF) as usize,
                    corridor_width,
                    WALK_STEER_STRENGTH,
                );
                graph[a][b] = true;
                graph[b][a] = true;
                break; // Only add one edge per node A
            }
        }
    }

    // scale out
    let inner_size = map.len();
    resize(&mut map, 3 * size / 4, 3 * size / 4);
    let outer_size = map.len();

    // Add one room to each of the 8 peripheral rectangles created by the expansion
    let border = (outer_size - inner_size) / 2;
    let mut outer_rooms = Vec::new();

    // Define the 8 peripheral regions: top-left, top, top-right, right, bottom-right, bottom, bottom-left, left
    let regions = [
        // (x_min, x_max, y_min, y_max)
        (0, border, 0, border),                                      // top-left corner
        (border, border + inner_size, 0, border),                    // top edge
        (border + inner_size, outer_size, 0, border),                // top-right corner
        (border + inner_size, outer_size, border, border + inner_size), // right edge
        (border + inner_size, outer_size, border + inner_size, outer_size), // bottom-right corner
        (border, border + inner_size, border + inner_size, outer_size), // bottom edge
        (0, border, border + inner_size, outer_size),                // bottom-left corner
        (0, border, border, border + inner_size),                    // left edge
    ];

    // Place one random room in each peripheral region
    for &(x_min, x_max, y_min, y_max) in &regions {
        if x_max > x_min && y_max > y_min {
            let x = rng.random_range(x_min..x_max);
            let y = rng.random_range(y_min..y_max);
            outer_rooms.push((x, y));
        }
    }

    // Create rooms in outer region
    for &room_pos in &outer_rooms {
        let room_size = rng.random_range(MIN_ROOM_SIZE..MAX_ROOM_SIZE);
        square_fill(&mut map, room_size / 2, room_pos);
    }

    // Connect each outer room to its nearest inner room
    for &outer_pos in &outer_rooms {
        // Find nearest inner room
        let border = (outer_size - inner_size) / 2;
        let mut min_dist = f64::MAX;
        let mut nearest_room = all_positions[0];

        for &inner_pos in &all_positions {
            // Adjust inner position to account for border
            let adjusted_inner = (inner_pos.0 + border, inner_pos.1 + border);
            let dist_sq = ((outer_pos.0 as isize - adjusted_inner.0 as isize).pow(2)
                + (outer_pos.1 as isize - adjusted_inner.1 as isize).pow(2)) as f64;
            if dist_sq < min_dist {
                min_dist = dist_sq;
                nearest_room = adjusted_inner;
            }
        }

        // Create random walk corridor to nearest room
        let corridor_width = rng.random_range(MIN_CORRIDOR_WIDTH..MAX_CORRIDOR_WIDTH);
        let dist = min_dist.sqrt();
        random_walk_fill(
            &mut map,
            &mut rng,
            &outer_pos,
            &nearest_room,
            WALK_BIAS_CHANCE,
            (dist * WALK_LEN_COEFF) as usize,
            corridor_width,
            WALK_STEER_STRENGTH,
        );
    }

    // scale out to guarantee solid boarder
    resize(&mut map, size, size);

    // Add some noise to make it less uniform
    simplex::generate_simplex_noise(&mut map, &mut rng, 0.045, 0.8);
    simplex::generate_simplex_noise(&mut map, &mut rng, 0.1, 0.7);

    // Smoothing
    ca::cellular_automata(&mut map, 5, |_, _| { });

    return map;
}

/// Check if two line segments intersect (not counting endpoint touches)
fn edges_intersect(p1: (usize, usize), p2: (usize, usize), p3: (usize, usize), p4: (usize, usize)) -> bool {
    let (x1, y1) = (p1.0 as f64, p1.1 as f64);
    let (x2, y2) = (p2.0 as f64, p2.1 as f64);
    let (x3, y3) = (p3.0 as f64, p3.1 as f64);
    let (x4, y4) = (p4.0 as f64, p4.1 as f64);

    let denom = (x1 - x2) * (y3 - y4) - (y1 - y2) * (x3 - x4);
    if denom.abs() < 1e-10 {
        return false; // parallel or coincident
    }

    let t = ((x1 - x3) * (y3 - y4) - (y1 - y3) * (x3 - x4)) / denom;
    let u = -((x1 - x2) * (y1 - y3) - (y1 - y2) * (x1 - x3)) / denom;

    // Check if intersection point is within both line segments (excluding endpoints)
    t > 0.0 && t < 1.0 && u > 0.0 && u < 1.0
}
