use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

// Module declarations
mod components;
mod constants;
mod events;
mod resources;
mod combat;
mod enemy;
mod player;
mod world;
mod ui;

// Import everything we need
use components::*;
use constants::*;
use events::*;
use resources::*;
use combat::*;
use enemy::*;
use player::*;
use world::*;
use ui::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Combat Sandbox - Enemy Archetypes".to_string(),
                resolution: (800.0, 600.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        // .add_plugins(RapierDebugRenderPlugin::default())
        .add_event::<ProjectileImpactEvent>()
        .add_event::<DamageEvent>()
        .insert_resource(FireTimer {
            timer: Timer::from_seconds(FIRE_RATE, TimerMode::Repeating),
        })
        .insert_resource(EnemySpawnTimer {
            timer: Timer::from_seconds(ENEMY_SPAWN_RATE, TimerMode::Repeating),
        })
        .add_systems(Startup, (disable_gravity, setup, setup_health_bar))
        .add_systems(Update, (
            // Player systems
            player_movement,
            shoot_projectiles,
            camera_follow,

            // Enemy systems
            spawn_enemies,
            enemy_ai,
            laser_sight_system,

            // UI systems
            update_health_bar,
            update_health_bar_color,

            // Combat systems
            detect_projectile_collisions,
            handle_projectile_impacts,
            detect_enemy_player_collisions,
            process_damage,
            cleanup_dead_entities,
            cleanup_projectiles,
        ))
        .run();
}
