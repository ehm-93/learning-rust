use bevy::prelude::*;
use rusqlite::{Connection, Result};

use crate::components::*;
use crate::resources::*;

pub struct PersistencePlugin;

impl Plugin for PersistencePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, init_database)
            .add_systems(Update, save_game_system.run_if(in_state(GameState::Paused)))
            .add_systems(Update, load_game_system.run_if(in_state(GameState::MainMenu)));
    }
}

fn init_database(save_path: Res<SavePath>) {
    if let Ok(conn) = Connection::open(&save_path.0) {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS save_data (
                id INTEGER PRIMARY KEY,
                position_x REAL,
                position_y REAL,
                position_z REAL,
                health REAL
            )",
            [],
        ).ok();
    }
}

fn save_game_system(
    _save_path: Res<SavePath>,
    _query: Query<(&Transform, &Health), (With<Player>, With<Saveable>)>,
) {
    // This will be triggered by button press in future
    // For now, it's just a placeholder
}

fn load_game_system(
    _save_path: Res<SavePath>,
    _query: Query<(&mut Transform, &mut Health), (With<Player>, With<Saveable>)>,
) {
    // This will be triggered by button press in future
    // For now, it's just a placeholder
}

pub fn save_game(save_path: &SavePath, position: Vec3, health: f32) -> Result<()> {
    let conn = Connection::open(&save_path.0)?;

    conn.execute(
        "INSERT OR REPLACE INTO save_data (id, position_x, position_y, position_z, health)
         VALUES (1, ?1, ?2, ?3, ?4)",
        (position.x, position.y, position.z, health),
    )?;

    Ok(())
}

pub fn load_game(save_path: &SavePath) -> Result<SaveData> {
    let conn = Connection::open(&save_path.0)?;

    let mut stmt = conn.prepare(
        "SELECT position_x, position_y, position_z, health FROM save_data WHERE id = 1"
    )?;

    let save_data = stmt.query_row([], |row| {
        Ok(SaveData {
            position: Vec3::new(
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
            ),
            health: row.get(3)?,
        })
    })?;

    Ok(save_data)
}
