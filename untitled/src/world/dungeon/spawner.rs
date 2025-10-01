use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use rand::Rng;

use crate::{
    components::*,
    constants::*,
    enemy::{spawn_single_enemy, spawn_enemy_group},
    world::scenes::manager::Scene,
};
use super::grid::DungeonGrid;

/// Convert the logical grid into physical entities
pub fn spawn_grid_as_entities(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    grid: &DungeonGrid,
) {
    let (width, height) = grid.dimensions();

    // Calculate the physical world bounds
    let total_width = width as f32 * CELL_SIZE;
    let total_height = height as f32 * CELL_SIZE;
    let start_x = -total_width / 2.0;
    let start_y = -total_height / 2.0;

    // For each cell in the grid, spawn walls for unvisited cells or floor tiles for visited cells
    for x in 0..width {
        for y in 0..height {
            let world_x = start_x + (x as f32 + 0.5) * CELL_SIZE;
            let world_y = start_y + (y as f32 + 0.5) * CELL_SIZE;

            if !grid.is_visited(x, y) {
                // Spawn solid wall for unvisited cells
                commands.spawn((
                    Mesh2d(meshes.add(Rectangle::new(CELL_SIZE, CELL_SIZE))),
                    MeshMaterial2d(materials.add(Color::srgb(WALL_COLOR[0], WALL_COLOR[1], WALL_COLOR[2]))),
                    Transform::from_translation(Vec3::new(world_x, world_y, 0.0)),
                    DungeonWall,
                    RigidBody::Fixed,
                    Collider::cuboid(CELL_SIZE / 2.0, CELL_SIZE / 2.0),
                ));
            } else {
                // Spawn floor tile for visited cells
                let floor_color = if grid.is_room(x, y) {
                    Color::srgb(ROOM_FLOOR_COLOR[0], ROOM_FLOOR_COLOR[1], ROOM_FLOOR_COLOR[2])
                } else {
                    Color::srgb(CORRIDOR_FLOOR_COLOR[0], CORRIDOR_FLOOR_COLOR[1], CORRIDOR_FLOOR_COLOR[2])
                };

                commands.spawn((
                    Mesh2d(meshes.add(Rectangle::new(CELL_SIZE, CELL_SIZE))),
                    MeshMaterial2d(materials.add(floor_color)),
                    Transform::from_translation(Vec3::new(world_x, world_y, -1.0)), // Slightly behind walls
                ));
            }
        }
    }

    // Spawn walls between all adjacent unconnected cells (avoiding duplicates)
    spawn_all_walls(commands, meshes, materials, grid);

    // Spawn enemies based on dungeon layout
    spawn_enemies_in_dungeon(commands, meshes, materials, grid);
}

/// Spawn walls between all adjacent unconnected cells, avoiding duplicates
pub fn spawn_all_walls(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    grid: &DungeonGrid,
) {
    let (width, height) = grid.dimensions();
    let total_width = width as f32 * CELL_SIZE;
    let total_height = height as f32 * CELL_SIZE;
    let start_x = -total_width / 2.0;
    let start_y = -total_height / 2.0;
    let wall_thickness = WALL_THICKNESS;
    let half_cell = CELL_SIZE / 2.0;

    // For each cell, check right and down neighbors to avoid duplicate walls
    for x in 0..width {
        for y in 0..height {
            let world_x = start_x + (x as f32 + 0.5) * CELL_SIZE;
            let world_y = start_y + (y as f32 + 0.5) * CELL_SIZE;

            let current_visited = grid.is_visited(x, y);
            let current_room_id = grid.get_room_id(x, y);
            let connections = grid.get_connections(x, y);

            // Check east neighbor (right)
            if x + 1 < width {
                let neighbor_visited = grid.is_visited(x + 1, y);
                let neighbor_room_id = grid.get_room_id(x + 1, y);

                // Spawn wall if:
                // 1. No connection between cells, OR
                // 2. Different rooms (both are rooms), OR
                // 3. One cell is visited and the other isn't
                let should_spawn_wall = !connections.east ||
                    (current_visited != neighbor_visited) ||
                    (current_room_id > 0 && neighbor_room_id > 0 && current_room_id != neighbor_room_id);

                if should_spawn_wall {
                    spawn_wall(
                        commands,
                        meshes,
                        materials,
                        world_x + half_cell,
                        world_y,
                        wall_thickness,
                        CELL_SIZE,
                        true, // vertical wall
                    );
                }
            } else if current_visited {
                // Edge of grid = wall for visited cells
                spawn_wall(
                    commands,
                    meshes,
                    materials,
                    world_x + half_cell,
                    world_y,
                    wall_thickness,
                    CELL_SIZE,
                    true, // vertical wall
                );
            }

            // Check north neighbor (up)
            if y + 1 < height {
                let neighbor_visited = grid.is_visited(x, y + 1);
                let neighbor_room_id = grid.get_room_id(x, y + 1);

                // Same logic as east wall
                let should_spawn_wall = !connections.north ||
                    (current_visited != neighbor_visited) ||
                    (current_room_id > 0 && neighbor_room_id > 0 && current_room_id != neighbor_room_id);

                if should_spawn_wall {
                    spawn_wall(
                        commands,
                        meshes,
                        materials,
                        world_x,
                        world_y + half_cell,
                        CELL_SIZE,
                        wall_thickness,
                        false, // horizontal wall
                    );
                }
            } else if current_visited {
                // Edge of grid = wall for visited cells
                spawn_wall(
                    commands,
                    meshes,
                    materials,
                    world_x,
                    world_y + half_cell,
                    CELL_SIZE,
                    wall_thickness,
                    false, // horizontal wall
                );
            }

            // Handle left and bottom edges for visited cells
            if x == 0 && current_visited {
                // Left edge wall
                spawn_wall(
                    commands,
                    meshes,
                    materials,
                    world_x - half_cell,
                    world_y,
                    wall_thickness,
                    CELL_SIZE,
                    true, // vertical wall
                );
            }

            if y == 0 && current_visited {
                // Bottom edge wall
                spawn_wall(
                    commands,
                    meshes,
                    materials,
                    world_x,
                    world_y - half_cell,
                    CELL_SIZE,
                    wall_thickness,
                    false, // horizontal wall
                );
            }
        }
    }
}

/// Spawns enemies intelligently based on dungeon layout
pub fn spawn_enemies_in_dungeon(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    grid: &DungeonGrid,
) {
    let mut rng = rand::rng();
    let (width, height) = grid.dimensions();
    let total_width = width as f32 * CELL_SIZE;
    let total_height = height as f32 * CELL_SIZE;
    let start_x = -total_width / 2.0;
    let start_y = -total_height / 2.0;

    // Collect all room centers and corridor positions
    let mut room_centers = Vec::new();
    let mut corridor_positions = Vec::new();

    // Find room centers (group cells by room_id)
    let mut room_cells: std::collections::HashMap<u32, Vec<(usize, usize)>> = std::collections::HashMap::new();

    for x in 0..width {
        for y in 0..height {
            if grid.is_visited(x, y) {
                let world_x = start_x + (x as f32 + 0.5) * CELL_SIZE;
                let world_y = start_y + (y as f32 + 0.5) * CELL_SIZE;

                if grid.is_room(x, y) {
                    let room_id = grid.get_room_id(x, y);
                    room_cells.entry(room_id).or_insert_with(Vec::new).push((x, y));
                } else {
                    // Corridor cell
                    corridor_positions.push(Vec2::new(world_x, world_y));
                }
            }
        }
    }

    // Calculate room centers
    for (_room_id, cells) in room_cells.iter() {
        if cells.len() >= 4 { // Only consider rooms with decent size
            let sum_x: f32 = cells.iter().map(|(x, _)| *x as f32).sum();
            let sum_y: f32 = cells.iter().map(|(_, y)| *y as f32).sum();
            let center_x = start_x + (sum_x / cells.len() as f32 + 0.5) * CELL_SIZE;
            let center_y = start_y + (sum_y / cells.len() as f32 + 0.5) * CELL_SIZE;
            room_centers.push(Vec2::new(center_x, center_y));
        }
    }

    println!("Found {} rooms and {} corridor positions", room_centers.len(), corridor_positions.len());

    // Define enemy types for room clusters (prefer room-suitable enemies)
    let room_enemy_types = [
        (EnemyArchetype::BigMelee, 0.25),        // 25% - needs space
        (EnemyArchetype::Sniper, 0.3),           // 30% - long sightlines
        (EnemyArchetype::MachineGunner, 0.25),   // 25% - positioning
        (EnemyArchetype::Shotgunner, 0.2),       // 20% - close combat
    ];

    // Define enemy types for corridors and additional spawns
    let corridor_enemy_types = [
        (EnemyArchetype::SmallMelee, 0.4),       // 40% - ambush enemies
        (EnemyArchetype::Shotgunner, 0.3),       // 30% - close quarters
        (EnemyArchetype::BigMelee, 0.2),         // 20% - patrol enemies
        (EnemyArchetype::MachineGunner, 0.1),    // 10% - rare in corridors
    ];

    let mut total_enemies_spawned = 0;

    // PHASE 1: Guaranteed clusters in every room (2-4 enemies per room)
    for room_center in &room_centers {
        let cluster_size = rng.random_range(2..=4);
        println!("Spawning cluster of {} enemies in room at {:?}", cluster_size, room_center);

        for i in 0..cluster_size {
            let archetype = select_weighted_archetype_simple(&room_enemy_types, &mut rng);

            // Position enemies around the room center with some spread
            let angle = (i as f32 / cluster_size as f32) * std::f32::consts::TAU;
            let radius = rng.random_range(20.0..60.0); // Spread within room
            let offset = Vec2::new(
                angle.cos() * radius + (rng.random::<f32>() - 0.5) * 40.0,
                angle.sin() * radius + (rng.random::<f32>() - 0.5) * 40.0,
            );
            let spawn_pos = *room_center + offset;

            // 50% chance for small groups within the cluster
            if matches!(archetype, EnemyArchetype::SmallMelee) && rng.random_bool(0.5) {
                spawn_enemy_group(commands, meshes, materials, archetype, spawn_pos, 2);
                total_enemies_spawned += 2;
            } else {
                spawn_single_enemy(commands, meshes, materials, archetype, spawn_pos);
                total_enemies_spawned += 1;
            }
        }
    }

    // PHASE 2: Additional enemies in corridors and extra room spawns
    let additional_enemies = rng.random_range(15..30) + corridor_positions.len() / 15;
    println!("Spawning {} additional enemies in corridors and rooms", additional_enemies);

    for _ in 0..additional_enemies {
        let archetype = select_weighted_archetype_simple(&corridor_enemy_types, &mut rng);

        // 60% chance to spawn in corridors, 40% chance for extra room spawns
        let spawn_pos = if !corridor_positions.is_empty() && rng.random_bool(0.6) {
            // Spawn in corridor
            corridor_positions[rng.random_range(0..corridor_positions.len())]
        } else if !room_centers.is_empty() {
            // Extra spawn in a random room
            let room_center = room_centers[rng.random_range(0..room_centers.len())];
            let offset = Vec2::new(
                (rng.random::<f32>() - 0.5) * CELL_SIZE * 0.8,
                (rng.random::<f32>() - 0.5) * CELL_SIZE * 0.8,
            );
            room_center + offset
        } else {
            Vec2::ZERO // Fallback
        };

        // Spawn with groups for small melee
        if matches!(archetype, EnemyArchetype::SmallMelee) && rng.random_bool(0.4) {
            let group_size = rng.random_range(2..=3);
            spawn_enemy_group(commands, meshes, materials, archetype, spawn_pos, group_size);
            total_enemies_spawned += group_size;
        } else {
            spawn_single_enemy(commands, meshes, materials, archetype, spawn_pos);
            total_enemies_spawned += 1;
        }
    }

    println!("Total enemies spawned: {} (guaranteed clusters in {} rooms)",
             total_enemies_spawned, room_centers.len());
}

/// Helper function to select enemy archetype based on simple weights (archetype, weight)
fn select_weighted_archetype_simple(
    configs: &[(EnemyArchetype, f32)],
    rng: &mut impl Rng,
) -> EnemyArchetype {
    let total_weight: f32 = configs.iter().map(|(_, weight)| weight).sum();
    let mut random_value = rng.random::<f32>() * total_weight;

    for (archetype, weight) in configs {
        random_value -= weight;
        if random_value <= 0.0 {
            return *archetype;
        }
    }

    configs[0].0 // Fallback
}



/// Helper function to spawn a single wall entity
fn spawn_wall(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    _is_vertical: bool,
) {
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(width, height))),
        MeshMaterial2d(materials.add(Color::srgb(WALL_COLOR[0], WALL_COLOR[1], WALL_COLOR[2]))),
        Transform::from_translation(Vec3::new(x, y, 0.0)),
        DungeonWall,
        RigidBody::Fixed,
        Collider::cuboid(width / 2.0, height / 2.0),
    ));
}

/// Scene-aware version of spawn_grid_as_entities
/// For now, this is a simplified version that just wraps the original function
/// TODO: Make the original spawn_grid_as_entities scene-aware and use that
pub fn spawn_grid_as_entities_in_scene<T: Scene>(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    grid: &DungeonGrid,
) {
    // For now, just call the original function
    // TODO: Refactor the original function to be scene-aware
    spawn_grid_as_entities(commands, meshes, materials, grid);
}
