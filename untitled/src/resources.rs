use bevy::prelude::*;

/// Timer resource for controlling player fire rate
#[derive(Resource)]
pub struct FireTimer {
    pub timer: Timer,
}

/// Timer resource for controlling enemy spawning
#[derive(Resource)]
pub struct EnemySpawnTimer {
    pub timer: Timer,
}

/// Score tracking resource
#[derive(Resource, Default)]
pub struct Score {
    pub current: u32,
}

/// Game state resource to track if game is over
#[derive(Resource, Default, PartialEq, Eq)]
pub enum GameState {
    #[default]
    Playing,
    GameOver,
}
