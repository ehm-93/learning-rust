use bevy::prelude::*;

/// Resource containing all game sound assets
#[derive(Resource)]
pub struct GameSounds {
    pub gun_01: Handle<AudioSource>,
    pub gun_02: Handle<AudioSource>,
    pub gun_03: Handle<AudioSource>,
    pub explosion_01: Handle<AudioSource>,
}

/// Load all sound assets at startup
pub fn load_sounds(mut commands: Commands, asset_server: Res<AssetServer>) {
    let sounds = GameSounds {
        gun_01: asset_server.load("sound/gun_01.wav"),
        gun_02: asset_server.load("sound/gun_02.wav"),
        gun_03: asset_server.load("sound/gun_03.wav"),
        explosion_01: asset_server.load("sound/explosion_01.wav"),
    };

    commands.insert_resource(sounds);
}

/// Play a sound effect with specified volume
/// Note: Volume control will be implemented later - currently plays at default volume
pub fn play_sound(commands: &mut Commands, sound: Handle<AudioSource>, _volume: f32) {
    commands.spawn(AudioPlayer::new(sound.clone()));
}
