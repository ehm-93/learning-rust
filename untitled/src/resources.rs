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
