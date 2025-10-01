use bevy::prelude::*;
use crate::world::states::WorldState;

/// Dungeon plugin that manages dungeon scenes and gameplay
pub struct DungeonPlugin;

impl Plugin for DungeonPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<DungeonSceneState>()
            .init_resource::<crate::world::states::DungeonConfig>()
            .add_systems(
                Update,
                (
                    detect_dungeon_state_changes,
                    handle_dungeon_exit.run_if(in_dungeon_state),
                )
            );
    }
}

// Helper condition function
fn in_dungeon_state(state: Res<State<WorldState>>) -> bool {
    matches!(state.get(), WorldState::Dungeon)
}

// Marker component for dungeon entities
#[derive(Component)]
pub struct DungeonEntity;

// Track dungeon scene state
#[derive(Resource, Default)]
pub struct DungeonSceneState {
    pub is_setup: bool,
    pub was_in_dungeon: bool,
}

// System to detect when we enter/exit dungeon state and setup/cleanup accordingly
fn detect_dungeon_state_changes(
    mut commands: Commands,
    current_state: Res<State<WorldState>>,
    mut scene_state: ResMut<DungeonSceneState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    dungeon_config: Res<crate::world::states::DungeonConfig>,
    dungeon_entities: Query<Entity, With<DungeonEntity>>,
) {
    let in_dungeon = matches!(current_state.get(), WorldState::Dungeon);

    // Entering dungeon state
    if in_dungeon && !scene_state.was_in_dungeon {
        info!("Entering Dungeon state - setting up scene");

        setup_dungeon_entities(
            &mut commands,
            &mut meshes,
            &mut materials,
            dungeon_config.depth,
            &dungeon_config.modifiers,
        );
        scene_state.is_setup = true;
    }
    // Leaving dungeon state
    else if !in_dungeon && scene_state.was_in_dungeon && scene_state.is_setup {
        info!("Leaving Dungeon state - cleaning up scene");

        for entity in dungeon_entities.iter() {
            commands.entity(entity).despawn();
        }
        scene_state.is_setup = false;
    }

    scene_state.was_in_dungeon = in_dungeon;
}



// Handle dungeon exit (escape key or death)
fn handle_dungeon_exit(
    keyboard: Res<ButtonInput<KeyCode>>,
    current_state: Res<State<WorldState>>,
    mut next_state: ResMut<NextState<WorldState>>,
    dungeon_config: Res<crate::world::states::DungeonConfig>,
    mut cathedral_config: ResMut<crate::world::states::CathedralConfig>,
) {
    use crate::world::states::WorldState;

    if keyboard.just_pressed(KeyCode::Escape) {
        if matches!(current_state.get(), WorldState::Dungeon) {
            info!("Dungeon: Player escaped from depth {} -> returning directly to cathedral", dungeon_config.depth);

            // Update progress based on dungeon completion
            cathedral_config.player_progress.update_depth_reached(dungeon_config.depth);

            // Transition directly to cathedral state
            next_state.set(WorldState::Cathedral);
        }
    }
}

// Helper function to set up dungeon entities
fn setup_dungeon_entities(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    depth: u32,
    modifiers: &[String],
) {
    // Create some basic dungeon environment for now
    // TODO: Integrate with the legacy dungeon generator

    // Add a simple floor
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(1600.0, 1200.0))),
        MeshMaterial2d(materials.add(Color::srgb(0.2, 0.15, 0.1))),
        Transform::from_translation(Vec3::new(0.0, 0.0, -2.0)),
        DungeonEntity,
    ));

    // Add some walls for visual reference
    for i in 0..4 {
        let angle = i as f32 * std::f32::consts::PI / 2.0;
        let distance = 600.0;
        let x = angle.cos() * distance;
        let y = angle.sin() * distance;

        commands.spawn((
            Mesh2d(meshes.add(Rectangle::new(40.0, 200.0))),
            MeshMaterial2d(materials.add(Color::srgb(0.4, 0.4, 0.4))),
            Transform::from_translation(Vec3::new(x, y, 0.0)),
            DungeonEntity,
        ));
    }

    // Add depth indicator text
    commands.spawn((
        Text2d::new(format!("Dungeon Depth: {}", depth)),
        TextFont {
            font_size: 24.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Transform::from_translation(Vec3::new(0.0, 400.0, 1.0)),
        DungeonEntity,
    ));

    // Add modifier info if any
    if !modifiers.is_empty() {
        commands.spawn((
            Text2d::new(format!("Modifiers: {}", modifiers.join(", "))),
            TextFont {
                font_size: 18.0,
                ..default()
            },
            TextColor(Color::srgb(1.0, 1.0, 0.0)),
            Transform::from_translation(Vec3::new(0.0, 350.0, 1.0)),
            DungeonEntity,
        ));
    }

    // Add exit instruction
    commands.spawn((
        Text2d::new("Press ESC to return to Cathedral"),
        TextFont {
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::srgb(0.5, 0.5, 0.5)),
        Transform::from_translation(Vec3::new(0.0, -400.0, 1.0)),
        DungeonEntity,
    ));

    info!("Dungeon entities spawned for depth {}", depth);
}
