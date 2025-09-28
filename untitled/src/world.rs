use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use rand::{rng, Rng};
use serde::{Deserialize, Serialize};

use crate::{
    components::*,
    constants::*,
};

/// Disables gravity for the 2D physics world
pub fn disable_gravity(mut query: Query<&mut RapierConfiguration>) {
    for mut config in &mut query {
        config.gravity = Vec2::ZERO;
    }
}

/// Sets up the initial game world with player, obstacles, and boundaries
pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Spawn 2D camera with following component
    commands.spawn((
        Camera2d,
        MainCamera,
    ));

    // Spawn player as a white circle
    commands.spawn((
        Mesh2d(meshes.add(Circle::new(PLAYER_RADIUS))),
        MeshMaterial2d(materials.add(Color::WHITE)),
        Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        Player,
        Team::Player,
        Health::new(PLAYER_MAX_HEALTH),
        Dash::new(),
        GrenadeThrower::new(),
        RigidBody::Dynamic,
        Collider::ball(PLAYER_RADIUS),
        // Lock rotation so the player doesn't spin
        LockedAxes::ROTATION_LOCKED,
        // Add Velocity component for movement
        Velocity::zero(),
        // Enable collision events for damage detection
        ActiveEvents::COLLISION_EVENTS,
    ));

    // Generate procedural dungeon rooms
    generate_dungeon_rooms(&mut commands, &mut meshes, &mut materials, 1);
}

/// Represents connections between adjacent cells
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
struct CellConnections {
    visited: bool,
    is_room: bool,
    room_id: u32, // 0 = not a room, >0 = room ID
    north: bool,
    south: bool,
    east: bool,
    west: bool,
}

#[derive(Clone, Copy)]
enum Direction {
    North,
    South,
    East,
    West,
}

/// Grid-based dungeon structure with cell connections
#[derive(Serialize, Deserialize)]
struct DungeonGrid {
    cells: Vec<Vec<CellConnections>>,
    width: usize,
    height: usize,
}

impl DungeonGrid {
    fn new(width: usize, height: usize) -> Self {
        Self {
            cells: vec![vec![CellConnections { visited: false, is_room: false, room_id: 0, north: false, south: false, east: false, west: false }; height]; width],
            width,
            height,
        }
    }

    fn set_visited(&mut self, x: usize, y: usize) {
        if x < self.width && y < self.height {
            self.cells[x][y].visited = true;
        }
    }

    fn is_visited(&self, x: usize, y: usize) -> bool {
        if x < self.width && y < self.height {
            self.cells[x][y].visited
        } else {
            false
        }
    }

    fn set_room_with_id(&mut self, x: usize, y: usize, room_id: u32) {
        if x < self.width && y < self.height {
            self.cells[x][y].is_room = true;
            self.cells[x][y].room_id = room_id;
        }
    }

    fn get_room_id(&self, x: usize, y: usize) -> u32 {
        if x < self.width && y < self.height {
            self.cells[x][y].room_id
        } else {
            0
        }
    }

    fn is_room(&self, x: usize, y: usize) -> bool {
        if x < self.width && y < self.height {
            self.cells[x][y].is_room
        } else {
            false
        }
    }

    fn connect_cells(&mut self, x: usize, y: usize, dir: Direction) {
        match dir {
            Direction::North => {
                if y + 1 < self.height {
                    self.cells[x][y].north = true;
                    self.cells[x][y + 1].south = true;
                }
            }
            Direction::South => {
                if y > 0 {
                    self.cells[x][y].south = true;
                    self.cells[x][y - 1].north = true;
                }
            }
            Direction::East => {
                if x + 1 < self.width {
                    self.cells[x][y].east = true;
                    self.cells[x + 1][y].west = true;
                }
            }
            Direction::West => {
                if x > 0 {
                    self.cells[x][y].west = true;
                    self.cells[x - 1][y].east = true;
                }
            }
        }
    }

    fn get_connections(&self, x: usize, y: usize) -> CellConnections {
        if x < self.width && y < self.height {
            self.cells[x][y]
        } else {
            CellConnections { visited: false, is_room: false, room_id: 0, north: false, south: false, east: false, west: false }
        }
    }

    fn get_unvisited_directions(&self, x: usize, y: usize) -> Vec<Direction> {
        let mut directions = Vec::new();

        // Check all 4 directions
        if x > 0 && !self.is_visited(x - 1, y) {
            directions.push(Direction::West);
        }
        if x + 1 < self.width && !self.is_visited(x + 1, y) {
            directions.push(Direction::East);
        }
        if y > 0 && !self.is_visited(x, y - 1) {
            directions.push(Direction::South);
        }
        if y + 1 < self.height && !self.is_visited(x, y + 1) {
            directions.push(Direction::North);
        }

        directions
    }

    fn get_neighbor(&self, x: usize, y: usize, dir: Direction) -> Option<(usize, usize)> {
        match dir {
            Direction::North => {
                if y + 1 < self.height {
                    Some((x, y + 1))
                } else {
                    None
                }
            }
            Direction::South => {
                if y > 0 {
                    Some((x, y - 1))
                } else {
                    None
                }
            }
            Direction::East => {
                if x + 1 < self.width {
                    Some((x + 1, y))
                } else {
                    None
                }
            }
            Direction::West => {
                if x > 0 {
                    Some((x - 1, y))
                } else {
                    None
                }
            }
        }
    }
}

/// Generate dungeon using random walk algorithm
fn generate_dungeon_rooms(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    _depth: u32,
) {
    let mut grid = DungeonGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut rng = rng();

    // Start random walk from center
    let start_x = GRID_WIDTH / 2;
    let start_y = GRID_HEIGHT / 2;

    // Random walk to connect cells
    random_walk_connect(&mut grid, &mut rng, start_x, start_y, 1.0);

    // Place rooms in some connected cells
    place_rooms(&mut grid, &mut rng);

    // Convert grid to physical entities
    spawn_grid_as_entities(commands, meshes, materials, &grid);
}

/// Random walk algorithm to connect cells
fn random_walk_connect(grid: &mut DungeonGrid, rng: &mut impl Rng, start_x: usize, start_y: usize, fill_ratio: f32) {
    let target_cells = (grid.width * grid.height) as f32 * fill_ratio;
    let mut connected_cells = 0;
    let mut stack = Vec::new();

    // Start the walk
    grid.set_visited(start_x, start_y);
    connected_cells += 1;
    stack.push((start_x, start_y));

    while connected_cells < target_cells as usize && !stack.is_empty() {
        // Choose a random position from our stack (not always the last - more branching)
        let stack_idx = if stack.len() == 1 { 0 } else { rng.random_range(0..stack.len()) };
        let (current_x, current_y) = stack[stack_idx];

        let unvisited_directions = grid.get_unvisited_directions(current_x, current_y);

        if !unvisited_directions.is_empty() {
            // Pick a random unvisited direction
            let direction = unvisited_directions[rng.random_range(0..unvisited_directions.len())];

            // Get the neighbor coordinates in that direction
            if let Some((next_x, next_y)) = grid.get_neighbor(current_x, current_y, direction) {
                // Connect the cells
                grid.connect_cells(current_x, current_y, direction);
                grid.set_visited(next_x, next_y);
                connected_cells += 1;

                // Add the new cell to our stack
                stack.push((next_x, next_y));
            }
        } else {
            // No more neighbors, remove this cell from the stack
            stack.remove(stack_idx);
        }
    }
}

/// Try to create a rectangular room by checking all cells in the defined room size area
fn try_create_rectangular_room(grid: &DungeonGrid, start_x: usize, start_y: usize) -> Vec<(usize, usize)> {
    let mut room_cells = Vec::new();

    // Check all cells in the room size area (ROOM_SIZE_X x ROOM_SIZE_Y)
    for dx in 0..ROOM_SIZE_X {
        for dy in 0..ROOM_SIZE_Y {
            let x = start_x + dx;
            let y = start_y + dy;

            // Check bounds
            if x >= grid.width || y >= grid.height {
                continue; // Skip cells outside grid
            }

            // Only include cells that are visited and not already in a room
            if grid.is_visited(x, y) && !grid.is_room(x, y) {
                room_cells.push((x, y));
            }
        }
    }

    room_cells
}

/// Connect all cells within a room to their adjacent room cells
fn connect_room_cells(grid: &mut DungeonGrid, room_cells: &[(usize, usize)]) {
    for &(x, y) in room_cells {
        // Check all 4 directions and connect to adjacent room cells

        // North
        if let Some((nx, ny)) = grid.get_neighbor(x, y, Direction::North) {
            if room_cells.contains(&(nx, ny)) {
                grid.connect_cells(x, y, Direction::North);
            }
        }

        // South
        if let Some((nx, ny)) = grid.get_neighbor(x, y, Direction::South) {
            if room_cells.contains(&(nx, ny)) {
                grid.connect_cells(x, y, Direction::South);
            }
        }

        // East
        if let Some((nx, ny)) = grid.get_neighbor(x, y, Direction::East) {
            if room_cells.contains(&(nx, ny)) {
                grid.connect_cells(x, y, Direction::East);
            }
        }

        // West
        if let Some((nx, ny)) = grid.get_neighbor(x, y, Direction::West) {
            if room_cells.contains(&(nx, ny)) {
                grid.connect_cells(x, y, Direction::West);
            }
        }
    }
}

/// Place rooms in some of the connected cells
fn place_rooms(grid: &mut DungeonGrid, rng: &mut impl Rng) {
    let room_attempts = ROOM_COUNT;

    for current_room_id in 1..room_attempts + 1 {
        // Make 3 attempts to find a viable room starting position
        for _ in 0..3 {
            // Find a random visited cell that could become a room
            let room_x = rng.random_range(1..grid.width - 1);
            let room_y = rng.random_range(1..grid.height - 1);

            // Try to create a rectangular room starting from this position
            let room_cells = try_create_rectangular_room(grid, room_x, room_y);

            // Only create a room if we found enough eligible cells
            if room_cells.len() >= 4 {  // Minimum room size (at least 4 cells)
                // Convert all eligible cells to room cells
                for &(rx, ry) in &room_cells {
                    grid.set_room_with_id(rx, ry, current_room_id as u32);
                }

                // Connect all cells within the room to each other
                connect_room_cells(grid, &room_cells);

                break; // Successfully placed room, no need for more attempts
            }
        }
    }
}

/// Convert the logical grid into physical entities
fn spawn_grid_as_entities(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    grid: &DungeonGrid,
) {
    // Calculate the physical world bounds
    let total_width = GRID_WIDTH as f32 * CELL_SIZE;
    let total_height = GRID_HEIGHT as f32 * CELL_SIZE;
    let start_x = -total_width / 2.0;
    let start_y = -total_height / 2.0;

    // For each cell in the grid, spawn walls for unvisited cells or floor tiles for visited cells
    for x in 0..grid.width {
        for y in 0..grid.height {
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
}

/// Spawn walls between all adjacent unconnected cells, avoiding duplicates
fn spawn_all_walls(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    grid: &DungeonGrid,
) {
    let total_width = GRID_WIDTH as f32 * CELL_SIZE;
    let total_height = GRID_HEIGHT as f32 * CELL_SIZE;
    let start_x = -total_width / 2.0;
    let start_y = -total_height / 2.0;
    let wall_thickness = WALL_THICKNESS;
    let half_cell = CELL_SIZE / 2.0;

    // For each cell, check right and down neighbors to avoid duplicate walls
    for x in 0..grid.width {
        for y in 0..grid.height {
            let world_x = start_x + (x as f32 + 0.5) * CELL_SIZE;
            let world_y = start_y + (y as f32 + 0.5) * CELL_SIZE;

            let current_visited = grid.is_visited(x, y);
            let current_room_id = grid.get_room_id(x, y);
            let connections = grid.get_connections(x, y);

            // Check east neighbor (right)
            if x + 1 < grid.width {
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
                    commands.spawn((
                        Mesh2d(meshes.add(Rectangle::new(wall_thickness, CELL_SIZE))),
                        MeshMaterial2d(materials.add(Color::srgb(WALL_COLOR[0], WALL_COLOR[1], WALL_COLOR[2]))),
                        Transform::from_translation(Vec3::new(world_x + half_cell, world_y, 0.0)),
                        DungeonWall,
                        RigidBody::Fixed,
                        Collider::cuboid(wall_thickness / 2.0, CELL_SIZE / 2.0),
                    ));
                }
            } else if current_visited {
                // Edge of grid = wall for visited cells
                commands.spawn((
                    Mesh2d(meshes.add(Rectangle::new(wall_thickness, CELL_SIZE))),
                    MeshMaterial2d(materials.add(Color::srgb(WALL_COLOR[0], WALL_COLOR[1], WALL_COLOR[2]))),
                    Transform::from_translation(Vec3::new(world_x + half_cell, world_y, 0.0)),
                    DungeonWall,
                    RigidBody::Fixed,
                    Collider::cuboid(wall_thickness / 2.0, CELL_SIZE / 2.0),
                ));
            }

            // Check north neighbor (up)
            if y + 1 < grid.height {
                let neighbor_visited = grid.is_visited(x, y + 1);
                let neighbor_room_id = grid.get_room_id(x, y + 1);

                // Same logic as east wall
                let should_spawn_wall = !connections.north ||
                    (current_visited != neighbor_visited) ||
                    (current_room_id > 0 && neighbor_room_id > 0 && current_room_id != neighbor_room_id);

                if should_spawn_wall {
                    commands.spawn((
                        Mesh2d(meshes.add(Rectangle::new(CELL_SIZE, wall_thickness))),
                        MeshMaterial2d(materials.add(Color::srgb(WALL_COLOR[0], WALL_COLOR[1], WALL_COLOR[2]))),
                        Transform::from_translation(Vec3::new(world_x, world_y + half_cell, 0.0)),
                        DungeonWall,
                        RigidBody::Fixed,
                        Collider::cuboid(CELL_SIZE / 2.0, wall_thickness / 2.0),
                    ));
                }
            } else if current_visited {
                // Edge of grid = wall for visited cells
                commands.spawn((
                    Mesh2d(meshes.add(Rectangle::new(CELL_SIZE, wall_thickness))),
                    MeshMaterial2d(materials.add(Color::srgb(WALL_COLOR[0], WALL_COLOR[1], WALL_COLOR[2]))),
                    Transform::from_translation(Vec3::new(world_x, world_y + half_cell, 0.0)),
                    DungeonWall,
                    RigidBody::Fixed,
                    Collider::cuboid(CELL_SIZE / 2.0, wall_thickness / 2.0),
                ));
            }

            // Handle left and bottom edges for visited cells
            if x == 0 && current_visited {
                // Left edge wall
                commands.spawn((
                    Mesh2d(meshes.add(Rectangle::new(wall_thickness, CELL_SIZE))),
                    MeshMaterial2d(materials.add(Color::srgb(WALL_COLOR[0], WALL_COLOR[1], WALL_COLOR[2]))),
                    Transform::from_translation(Vec3::new(world_x - half_cell, world_y, 0.0)),
                    DungeonWall,
                    RigidBody::Fixed,
                    Collider::cuboid(wall_thickness / 2.0, CELL_SIZE / 2.0),
                ));
            }

            if y == 0 && current_visited {
                // Bottom edge wall
                commands.spawn((
                    Mesh2d(meshes.add(Rectangle::new(CELL_SIZE, wall_thickness))),
                    MeshMaterial2d(materials.add(Color::srgb(WALL_COLOR[0], WALL_COLOR[1], WALL_COLOR[2]))),
                    Transform::from_translation(Vec3::new(world_x, world_y - half_cell, 0.0)),
                    DungeonWall,
                    RigidBody::Fixed,
                    Collider::cuboid(CELL_SIZE / 2.0, wall_thickness / 2.0),
                ));
            }
        }
    }
}
