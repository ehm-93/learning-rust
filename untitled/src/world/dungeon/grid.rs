use serde::{Deserialize, Serialize};

/// Represents connections between adjacent cells
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct CellConnections {
    pub visited: bool,
    pub is_room: bool,
    pub room_id: u32, // 0 = not a room, >0 = room ID
    pub north: bool,
    pub south: bool,
    pub east: bool,
    pub west: bool,
}

impl Default for CellConnections {
    fn default() -> Self {
        Self {
            visited: false,
            is_room: false,
            room_id: 0,
            north: false,
            south: false,
            east: false,
            west: false,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Direction {
    North,
    South,
    East,
    West,
}

/// Grid-based dungeon structure with cell connections
#[derive(Serialize, Deserialize)]
pub struct DungeonGrid {
    cells: Vec<Vec<CellConnections>>,
    width: usize,
    height: usize,
}

impl DungeonGrid {
    /// Creates a new empty dungeon grid
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            cells: vec![vec![CellConnections::default(); height]; width],
            width,
            height,
        }
    }

    /// Marks a cell as visited
    pub fn set_visited(&mut self, x: usize, y: usize) {
        if x < self.width && y < self.height {
            self.cells[x][y].visited = true;
        }
    }

    /// Checks if a cell has been visited
    pub fn is_visited(&self, x: usize, y: usize) -> bool {
        if x < self.width && y < self.height {
            self.cells[x][y].visited
        } else {
            false
        }
    }

    /// Marks a cell as a room with the given ID
    pub fn set_room_with_id(&mut self, x: usize, y: usize, room_id: u32) {
        if x < self.width && y < self.height {
            self.cells[x][y].is_room = true;
            self.cells[x][y].room_id = room_id;
        }
    }

    /// Gets the room ID for a cell (0 if not a room)
    pub fn get_room_id(&self, x: usize, y: usize) -> u32 {
        if x < self.width && y < self.height {
            self.cells[x][y].room_id
        } else {
            0
        }
    }

    /// Checks if a cell is part of a room
    pub fn is_room(&self, x: usize, y: usize) -> bool {
        if x < self.width && y < self.height {
            self.cells[x][y].is_room
        } else {
            false
        }
    }

    /// Connects two adjacent cells in the specified direction
    pub fn connect_cells(&mut self, x: usize, y: usize, dir: Direction) {
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

    /// Gets the connections for a cell
    pub fn get_connections(&self, x: usize, y: usize) -> CellConnections {
        if x < self.width && y < self.height {
            self.cells[x][y]
        } else {
            CellConnections::default()
        }
    }

    /// Gets all unvisited directions from a cell
    pub fn get_unvisited_directions(&self, x: usize, y: usize) -> Vec<Direction> {
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

    /// Gets the coordinates of a neighbor in the specified direction
    pub fn get_neighbor(&self, x: usize, y: usize, dir: Direction) -> Option<(usize, usize)> {
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

    /// Gets the grid dimensions
    pub fn dimensions(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    /// Gets the grid width
    pub fn width(&self) -> usize {
        self.width
    }

    /// Gets the grid height
    pub fn height(&self) -> usize {
        self.height
    }
}
