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
mod sounds;
mod line_of_sight;

// Import everything we need
use constants::*;
use events::*;
use resources::*;
use combat::*;
use enemy::*;
use player::*;
use world::*;
use ui::*;
use sounds::*;

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
        .add_event::<HitFlashEvent>()
        .add_event::<GrenadeExplosionEvent>()
        .insert_resource(FireTimer {
            timer: Timer::from_seconds(FIRE_RATE, TimerMode::Repeating),
        })

        .insert_resource(Score::default())
        .insert_resource(GameState::default())
        .insert_resource(DungeonParams::default())
        .add_systems(Startup, (disable_gravity, setup, setup_health_bar, setup_score_display, load_sounds))
        .add_systems(Update, (
            // Player systems
            player_movement.run_if(resource_equals(GameState::Playing)),
            shoot_projectiles.run_if(resource_equals(GameState::Playing)),
            throw_grenades.run_if(resource_equals(GameState::Playing)),
            camera_follow,

            // Enemy systems
            enemy_ai.run_if(resource_equals(GameState::Playing)),
            laser_sight_system.run_if(resource_equals(GameState::Playing)),

            // UI systems
            update_health_bar,
            update_health_bar_color,
            update_score_display,
            show_game_over_overlay,
            handle_restart_button,
        ))
        .add_systems(Update, (
            // Combat systems
            detect_projectile_collisions.run_if(resource_equals(GameState::Playing)),
            handle_projectile_impacts.run_if(resource_equals(GameState::Playing)),
            detect_enemy_player_collisions.run_if(resource_equals(GameState::Playing)),
            handle_grenade_explosions.run_if(resource_equals(GameState::Playing)),
            process_grenade_explosions.run_if(resource_equals(GameState::Playing)),
            update_explosion_effects.run_if(resource_equals(GameState::Playing)),
            manage_grenade_speed.run_if(resource_equals(GameState::Playing)),
            process_damage.run_if(resource_equals(GameState::Playing)),
            handle_hit_flash.run_if(resource_equals(GameState::Playing)), // Run before cleanup
            cleanup_dead_entities, // Run after hit flash
            update_hit_flash,
            cleanup_projectiles.run_if(resource_equals(GameState::Playing)),
        ))
        .run();
}
