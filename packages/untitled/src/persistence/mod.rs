//! World persistence using SQLite
//!
//! This module handles saving and loading of chunk data (terrain tiles and FOW masks)
//! to/from a SQLite database for seamless chunk unload/reload cycles.

use bevy::prelude::*;
use rusqlite::{Connection, Result as SqlResult};
use std::sync::{Arc, Mutex};

use crate::world::chunks::{ChunkCoord, CHUNK_SIZE};
use crate::world::tiles::TileType;

/// Resource wrapping a SQLite connection for chunk persistence
#[derive(Resource, Clone)]
pub struct ChunkDatabase {
    /// Thread-safe database connection
    connection: Arc<Mutex<Connection>>,
}

impl ChunkDatabase {
    /// Create a new database connection and initialize schema
    pub fn new(db_path: &str) -> SqlResult<Self> {
        let conn = Connection::open(db_path)?;

        // Create terrain chunks table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS terrain_chunks (
                chunk_x INTEGER NOT NULL,
                chunk_y INTEGER NOT NULL,
                tiles BLOB NOT NULL,
                PRIMARY KEY (chunk_x, chunk_y)
            )",
            [],
        )?;

        // Create FOW chunks table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS fow_chunks (
                chunk_x INTEGER NOT NULL,
                chunk_y INTEGER NOT NULL,
                vision BLOB NOT NULL,
                PRIMARY KEY (chunk_x, chunk_y)
            )",
            [],
        )?;

        Ok(Self {
            connection: Arc::new(Mutex::new(conn)),
        })
    }

    /// Save terrain chunk data to database
    pub fn save_terrain_chunk(
        &self,
        chunk_coord: ChunkCoord,
        tiles: &[[TileType; CHUNK_SIZE as usize]; CHUNK_SIZE as usize],
    ) -> SqlResult<()> {
        let conn = self.connection.lock().unwrap();

        // Serialize tiles to bytes (simple binary format)
        let mut bytes = Vec::with_capacity(CHUNK_SIZE as usize * CHUNK_SIZE as usize);
        for row in tiles.iter() {
            for tile in row.iter() {
                bytes.push(*tile as u8);
            }
        }

        conn.execute(
            "INSERT OR REPLACE INTO terrain_chunks (chunk_x, chunk_y, tiles) VALUES (?1, ?2, ?3)",
            rusqlite::params![chunk_coord.x, chunk_coord.y, &bytes],
        )?;

        Ok(())
    }

    /// Load terrain chunk data from database
    pub fn load_terrain_chunk(
        &self,
        chunk_coord: ChunkCoord,
    ) -> SqlResult<Option<[[TileType; CHUNK_SIZE as usize]; CHUNK_SIZE as usize]>> {
        let conn = self.connection.lock().unwrap();

        let mut stmt = conn.prepare(
            "SELECT tiles FROM terrain_chunks WHERE chunk_x = ?1 AND chunk_y = ?2"
        )?;

        let result = stmt.query_row(
            rusqlite::params![chunk_coord.x, chunk_coord.y],
            |row| {
                let bytes: Vec<u8> = row.get(0)?;

                // Deserialize bytes back to tiles array
                let mut tiles = [[TileType::Floor; CHUNK_SIZE as usize]; CHUNK_SIZE as usize];
                for (i, byte) in bytes.iter().enumerate() {
                    let y = i / CHUNK_SIZE as usize;
                    let x = i % CHUNK_SIZE as usize;
                    tiles[y][x] = TileType::from_u8(*byte);
                }

                Ok(tiles)
            },
        );

        match result {
            Ok(tiles) => Ok(Some(tiles)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// Save FOW chunk vision data to database
    pub fn save_fow_chunk(
        &self,
        chunk_coord: ChunkCoord,
        vision: &Vec<Vec<u8>>,
    ) -> SqlResult<()> {
        let conn = self.connection.lock().unwrap();

        // Flatten vision data to bytes
        let mut bytes = Vec::with_capacity(vision.len() * vision[0].len());
        for row in vision.iter() {
            bytes.extend_from_slice(row);
        }

        conn.execute(
            "INSERT OR REPLACE INTO fow_chunks (chunk_x, chunk_y, vision) VALUES (?1, ?2, ?3)",
            rusqlite::params![chunk_coord.x, chunk_coord.y, &bytes],
        )?;

        Ok(())
    }

    /// Load FOW chunk vision data from database
    pub fn load_fow_chunk(
        &self,
        chunk_coord: ChunkCoord,
    ) -> SqlResult<Option<Vec<Vec<u8>>>> {
        let conn = self.connection.lock().unwrap();

        let mut stmt = conn.prepare(
            "SELECT vision FROM fow_chunks WHERE chunk_x = ?1 AND chunk_y = ?2"
        )?;

        let result = stmt.query_row(
            rusqlite::params![chunk_coord.x, chunk_coord.y],
            |row| {
                let bytes: Vec<u8> = row.get(0)?;

                // Reconstruct 2D vision array
                let size = (bytes.len() as f32).sqrt() as usize;
                let mut vision = vec![vec![0u8; size]; size];
                for (i, byte) in bytes.iter().enumerate() {
                    let y = i / size;
                    let x = i % size;
                    vision[y][x] = *byte;
                }

                Ok(vision)
            },
        );

        match result {
            Ok(vision) => Ok(Some(vision)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// Delete terrain chunk from database (optional cleanup)
    pub fn delete_terrain_chunk(&self, chunk_coord: ChunkCoord) -> SqlResult<()> {
        let conn = self.connection.lock().unwrap();
        conn.execute(
            "DELETE FROM terrain_chunks WHERE chunk_x = ?1 AND chunk_y = ?2",
            rusqlite::params![chunk_coord.x, chunk_coord.y],
        )?;
        Ok(())
    }

    /// Delete FOW chunk from database (optional cleanup)
    pub fn delete_fow_chunk(&self, chunk_coord: ChunkCoord) -> SqlResult<()> {
        let conn = self.connection.lock().unwrap();
        conn.execute(
            "DELETE FROM fow_chunks WHERE chunk_x = ?1 AND chunk_y = ?2",
            rusqlite::params![chunk_coord.x, chunk_coord.y],
        )?;
        Ok(())
    }

    /// Get count of saved terrain chunks (for debugging)
    pub fn terrain_chunk_count(&self) -> SqlResult<i32> {
        let conn = self.connection.lock().unwrap();
        let count: i32 = conn.query_row(
            "SELECT COUNT(*) FROM terrain_chunks",
            [],
            |row| row.get(0),
        )?;
        Ok(count)
    }

    /// Get count of saved FOW chunks (for debugging)
    pub fn fow_chunk_count(&self) -> SqlResult<i32> {
        let conn = self.connection.lock().unwrap();
        let count: i32 = conn.query_row(
            "SELECT COUNT(*) FROM fow_chunks",
            [],
            |row| row.get(0),
        )?;
        Ok(count)
    }
}

impl TileType {
    /// Convert u8 back to TileType (must match the as u8 conversion)
    fn from_u8(value: u8) -> Self {
        match value {
            0 => TileType::Floor,
            1 => TileType::Wall,
            _ => TileType::Floor, // Default fallback
        }
    }
}

/// System to initialize the chunk database
fn initialize_chunk_database(mut commands: Commands) {
    // Store database in the current directory as "chunks.db"
    match ChunkDatabase::new("chunks.db") {
        Ok(db) => {
            info!("Chunk database initialized successfully");
            commands.insert_resource(db);
        }
        Err(e) => {
            error!("Failed to initialize chunk database: {}", e);
        }
    }
}

/// Plugin for chunk persistence using SQLite
///
/// This plugin initializes the SQLite database connection on startup
/// and provides the ChunkDatabase resource for other systems to use.
pub struct PersistencePlugin;

impl Plugin for PersistencePlugin {
    fn build(&self, app: &mut App) {
        app
            // Initialize the database on startup
            .add_systems(Startup, initialize_chunk_database);
    }
}
