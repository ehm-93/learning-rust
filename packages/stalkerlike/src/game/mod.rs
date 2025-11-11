use bevy::prelude::*;
use bevy_egui::{EguiPlugin, PrimaryEguiContext};
use bevy_rapier3d::prelude::*;

mod components;
mod persistence;
mod player;
mod resources;
mod ui;

use components::{Saveable, GameEntity};
use persistence::PersistencePlugin;
use player::PlayerPlugin;
use resources::*;
use ui::UiPlugin;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app
            // Bevy default plugins
            .add_plugins(DefaultPlugins)

            // Third-party plugins
            .add_plugins(EguiPlugin::default())
            .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
            // .add_plugins(RapierDebugRenderPlugin::default())

            // Game plugins
            .add_plugins(PlayerPlugin)
            .add_plugins(UiPlugin)
            .add_plugins(PersistencePlugin)

            // Game state
            .init_state::<GameState>()

            // Resources
            .insert_resource(SavePath::default())

            // MainMenu state
            .add_systems(OnEnter(GameState::MainMenu), (
                cleanup_game_scene,
                setup_menu_camera,
            ).chain())
            .add_systems(OnExit(GameState::MainMenu), cleanup_menu_camera)

            // NewGame state
            .add_systems(OnEnter(GameState::NewGame), setup_world)

            // Loading state
            .add_systems(OnEnter(GameState::Loading), (
                setup_static_world,
                cleanup_game_scene,
            ))

            // InGame state - handled by PlayerPlugin

            // Paused state
            .add_systems(OnEnter(GameState::Paused), pause_physics)
            .add_systems(OnExit(GameState::Paused), resume_physics)

            // Add colliders to loaded scene meshes
            .add_systems(Update, add_colliders_to_scene_children);
    }
}

/// Spawns static world content (ground, lights, fixed objects)
/// Called on both NewGame and Loading states
fn setup_static_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    spawn_static_content(&mut commands, &mut meshes, &mut materials, &asset_server);
}

/// Helper function to spawn static content
fn spawn_static_content(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    asset_server: &Res<AssetServer>,
) {
    // Ground plane with physics collider
    commands.spawn((
        GameEntity,
        Mesh3d(meshes.add(Plane3d::default().mesh().size(50.0, 50.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.3, 0.3, 0.3))),
        Transform::from_xyz(0.0, 0.0, 0.0),
        Collider::cuboid(25.0, 0.1, 25.0),
    ));

    // Load pipe model from GLB file
    commands.spawn((
        GameEntity,
        NeedsCollider, // Marker to add colliders to child meshes
        SceneRoot(asset_server.load("models/pipes/pipe_2m_8m_hollow.glb#Scene0")),
        Transform::from_xyz(3.0, 1.0, 0.0),
        RigidBody::Fixed,
    ));

    // Load pipe model from GLB file
    commands.spawn((
        GameEntity,
        NeedsCollider, // Marker to add colliders to child meshes
        SceneRoot(asset_server.load("models/pipes/pipe_2m_8m_hollow_elbow_90.glb#Scene0")),
        Transform::from_xyz(-10.0, 1.0, -10.0),
        RigidBody::Fixed,
    ));

    // Static object (cube) with physics
    commands.spawn((
        GameEntity,
        Mesh3d(meshes.add(Cuboid::new(2.0, 2.0, 2.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.8, 0.3, 0.3))),
        Transform::from_xyz(0.0, 1.0, -5.0),
        RigidBody::Fixed,
        Collider::cuboid(1.0, 1.0, 1.0),
    ));

    // Directional light
    commands.spawn((
        GameEntity,
        DirectionalLight {
            illuminance: 0.0001,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Ambient light (resource, not an entity)
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.0,
        affects_lightmapped_meshes: true,
    });
}

/// Spawns dynamic world content for a new game
/// Only called on NewGame state
fn setup_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // Setup static world first
    spawn_static_content(&mut commands, &mut meshes, &mut materials, &asset_server);

    // Add some dynamic physics objects for testing
    for i in 0..5 {
        commands.spawn((
            GameEntity,
            Saveable,
            Mesh3d(meshes.add(Sphere::new(0.5))),
            MeshMaterial3d(materials.add(Color::srgb(0.2, 0.7, 0.9))),
            Transform::from_xyz(i as f32 * 2.0 - 4.0, i as f32 * 5.0, 0.0),
            RigidBody::Dynamic,
            Collider::ball(0.5),
            Restitution::coefficient(0.85),
            Damping { linear_damping: 0.2, angular_damping: 0.2 },
        ));
    }
}#[derive(Component)]
struct MenuCamera;

fn setup_menu_camera(mut commands: Commands) {
    // Spawn a 2D camera for the main menu
    commands.spawn((
        Camera2d,
        MenuCamera,
        PrimaryEguiContext,
    ));
}

fn cleanup_menu_camera(
    mut commands: Commands,
    query: Query<Entity, With<MenuCamera>>,
) {
    // Remove the menu camera when leaving main menu
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

fn cleanup_game_scene(
    mut commands: Commands,
    query: Query<Entity, With<GameEntity>>,
) {
    // Clean up all entities marked with GameEntity
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

fn pause_physics(mut query: Query<&mut RapierConfiguration>) {
    if let Ok(mut config) = query.single_mut() {
        config.physics_pipeline_active = false;
    }
}

fn resume_physics(mut query: Query<&mut RapierConfiguration>) {
    if let Ok(mut config) = query.single_mut() {
        config.physics_pipeline_active = true;
    }
}

/// Marker component for scene roots that need colliders on their children
/// Used because AsyncCollider can't be added directly to SceneRoot -
/// it needs to be on the actual mesh entities which spawn as children
#[derive(Component)]
struct NeedsCollider;

/// Add trimesh colliders to all mesh children of scenes marked with NeedsCollider
fn add_colliders_to_scene_children(
    mut commands: Commands,
    needs_collider_query: Query<(Entity, &Children), With<NeedsCollider>>,
    children_query: Query<&Children>,
    mesh_query: Query<Entity, (With<Mesh3d>, Without<Collider>)>,
) {
    for (parent_entity, children) in needs_collider_query.iter() {
        let mut found_any = false;

        // Recursively check all descendants for meshes
        let mut to_check = children.to_vec();

        while let Some(entity) = to_check.pop() {
            // Check if this entity has a mesh
            if mesh_query.get(entity).is_ok() {
                commands.entity(entity).insert(
                    AsyncCollider(ComputedColliderShape::TriMesh(TriMeshFlags::default()))
                );
                found_any = true;
            }

            // Add this entity's children to the check list
            if let Ok(grandchildren) = children_query.get(entity) {
                to_check.extend(grandchildren.iter());
            }
        }

        // Remove marker once we've processed all children
        if found_any {
            commands.entity(parent_entity).remove::<NeedsCollider>();
        }
    }
}
