use bevy::prelude::*;

/// Game state resource to track if game is over
#[derive(Resource, Default, PartialEq, Eq, Debug)]
pub enum GameState {
    #[default]
    Playing,
    GameOver,
}
