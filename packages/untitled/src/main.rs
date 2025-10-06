use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use bevy_ecs_tilemap::prelude::*;

// Module declarations
mod components;
mod constants;
mod events;
mod resources;
// mod combat; // Temporarily disabled for Phase 0
mod enemy;
mod player;
mod world;
mod ui;
mod sounds;
mod line_of_sight;
mod inventory;
mod debug;
mod packages;

// Import everything we need
use events::*;
use resources::*;
// use combat::*; // Temporarily disabled for Phase 0
use enemy::*;
use world::*;
use ui::*;
use sounds::*;
use inventory::InventoryPlugin;
use world::WorldPlugin;
use player::PlayerPlugin;
use debug::DebugOverlayPlugin;
use packages::*;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Combat Sandbox - Enemy Archetypes".to_string(),
                resolution: (1280.0, 720.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add_plugins(TilemapPlugin)
        .add_plugins(PlayerPlugin)
        .add_plugins(InventoryPlugin)
        .add_plugins(WorldPlugin)
        .add_plugins(DebugOverlayPlugin)
        .add_event::<ProjectileImpactEvent>()
        .add_event::<DamageEvent>()
        .add_event::<HitFlashEvent>()
        .add_event::<GrenadeExplosionEvent>()
        .add_event::<PortalActivationEvent>()
        .insert_resource(GameState::default())
        .insert_resource(ui::tooltip::TooltipState::default())
        .add_systems(Startup, (disable_gravity, setup_health_bar, load_sounds, setup_homing_barrage_test))
        .add_systems(Update, (
            // Phase 1 Effect systems
            effect_update_system.run_if(resource_equals(GameState::Playing)),

            // Enemy systems
            enemy_ai.run_if(resource_equals(GameState::Playing)),
            laser_sight_system.run_if(resource_equals(GameState::Playing)),

            // UI systems
            update_health_bar,
            update_health_bar_color,
            show_game_over_overlay,
            handle_restart_button,

            // Tooltip systems
            ui::tooltip::handle_tooltip_hover.run_if(resource_equals(GameState::Playing)),
            ui::tooltip::cleanup_orphaned_tooltips,
        ));
        // Combat systems temporarily disabled for Phase 0
        // .add_systems(Update, (
        //     // Combat systems
        //     detect_projectile_collisions.run_if(resource_equals(GameState::Playing)),
        //     handle_projectile_impacts.run_if(resource_equals(GameState::Playing)),
        //     detect_enemy_player_collisions.run_if(resource_equals(GameState::Playing)),
        //     handle_grenade_explosions.run_if(resource_equals(GameState::Playing)),
        //     process_grenade_explosions.run_if(resource_equals(GameState::Playing)),
        //     update_explosion_effects.run_if(resource_equals(GameState::Playing)),
        //     manage_grenade_speed.run_if(resource_equals(GameState::Playing)),
        //     process_damage.run_if(resource_equals(GameState::Playing)),
        //     handle_hit_flash.run_if(resource_equals(GameState::Playing)), // Run before cleanup
        //     cleanup_dead_entities, // Run after hit flash
        //     update_hit_flash,
        //     cleanup_projectiles.run_if(resource_equals(GameState::Playing)),
        // ));

    #[cfg(feature = "debug-physics")]
    app.add_plugins(RapierDebugRenderPlugin::default());

    app.run();
}


