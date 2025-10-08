use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use bevy_ecs_tilemap::prelude::*;

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
mod inventory;
mod debug;
mod persistence;

// Import everything we need
use events::*;
use resources::*;
use enemy::*;
use world::*;
use ui::*;
use sounds::*;
use inventory::InventoryPlugin;
use world::WorldPlugin;
use player::PlayerPlugin;
use debug::DebugOverlayPlugin;

#[cfg(feature = "mapgen-test")]
fn main() {
    use std::time::{SystemTime, UNIX_EPOCH};

    let size = 2048;
    let seed = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let arg = std::env::args().into_iter().nth(1).unwrap_or_default();
    // if first arg is "roomy" then generate a roomy map and save to out/roomy_test.png
    if arg == "roomy" {
        let map = crate::world::mapgen::roomy::roomy(size, seed);
        crate::world::mapgen::save_png(&map, "out/roomy_test.png");
        let fill = crate::world::mapgen::operators::flood_fill_bool(&map, (map[0].len() / 2, map.len() / 2));
        crate::world::mapgen::save_png(&fill, "out/roomy_test_filled.png");
        return;
    }
    // otherwise generate a freeform map and save to out/freeform_test.png
    let map = crate::world::mapgen::freeform::freeform(size, seed);
    crate::world::mapgen::save_png(&map, "out/freeform_test.png");
    let fill = crate::world::mapgen::operators::flood_fill_bool(&map, (map[0].len() / 2, map.len() / 2));
    crate::world::mapgen::save_png(&fill, "out/freeform_test_filled.png");
}

#[cfg(not(feature = "mapgen-test"))]
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

        // Set fixed timestep to 20 Hz for more consistent behavior updates
        .insert_resource(Time::<Fixed>::from_hz(20.0))

        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add_plugins(TilemapPlugin)
        .add_plugins(persistence::PersistencePlugin)
        .add_plugins(PlayerPlugin)
        .add_plugins(InventoryPlugin)
        .add_plugins(WorldPlugin)
        .add_plugins(DebugOverlayPlugin)
        .add_plugins(combat::FowPlugin)

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


