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
mod behavior;

// Import everything we need
use events::*;
use resources::*;
// use combat::*; // Temporarily disabled for Phase 0
use enemy::*;
use world::*;
use ui::*;
use sounds::*;
use inventory::InventoryPlugin;
use behavior::{BehaviorPlugin, BehaviorTestScenePlugin};
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

        // Set fixed timestep to 20 Hz for more consistent physics and behavior updates
        .insert_resource(Time::<Fixed>::from_hz(20.0))

        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add_plugins(TilemapPlugin)
        .add_plugins(PlayerPlugin)
        .add_plugins(InventoryPlugin)
        .add_plugins(WorldPlugin)
        .add_plugins(DebugOverlayPlugin)
        .add_plugins(BehaviorPlugin)
        .add_plugins(PackagePlugin)
        .add_plugins(BehaviorTestScenePlugin) // Behavior stress test

        .add_event::<ProjectileImpactEvent>()
        .add_event::<DamageEvent>()
        .add_event::<HitFlashEvent>()
        .add_event::<GrenadeExplosionEvent>()
        .add_event::<PortalActivationEvent>()

        .insert_resource(GameState::default())
        .insert_resource(ui::tooltip::TooltipState::default())

        .add_systems(Startup, (
            disable_gravity,
            setup_health_bar,
            load_sounds,
        ))
        .add_systems(Update, (
            handle_restart_button,

            // Tooltip systems
            ui::tooltip::cleanup_orphaned_tooltips,
            ui::tooltip::handle_tooltip_hover.run_if(resource_equals(GameState::Playing)),
        ))
        .add_systems(FixedUpdate, (
            // Phase 1 Effect systems
            effect_update_system.run_if(resource_equals(GameState::Playing)),

            // Enemy systems
            enemy_ai.run_if(resource_equals(GameState::Playing)),
            laser_sight_system.run_if(resource_equals(GameState::Playing)),

            // UI systems
            update_health_bar,
            update_health_bar_color,
            show_game_over_overlay,
        ));

    #[cfg(feature = "debug-physics")]
    app.add_plugins(RapierDebugRenderPlugin::default());

    app.run();
}


