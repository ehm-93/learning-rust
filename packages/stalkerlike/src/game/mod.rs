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
            .add_systems(OnEnter(GameState::NewGame), (
                setup_world,
                spawn_player_at_marker,
            ).chain())

            // Loading state
            .add_systems(OnEnter(GameState::Loading), (
                setup_static_world,
                cleanup_game_scene,
                spawn_player_at_marker,
            ).chain())

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
    _asset_server: Res<AssetServer>,
) {
    // Load the test scene from YAML
    // Path is relative to the Cargo workspace root when running with cargo run
    let scene_path = "packages/stalkerlike/assets/scenes/test_scene.yaml";

    info!("Attempting to load scene from: {}", scene_path);
    info!("Current directory: {:?}", std::env::current_dir());

    match load_scene_from_yaml(scene_path, &mut commands, &mut meshes, &mut materials) {
        Ok(_) => {
            info!("✅ Successfully loaded scene from {}", scene_path);
        }
        Err(e) => {
            error!("❌ Failed to load scene from {}: {}", scene_path, e);
        }
    }
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
    _asset_server: Res<AssetServer>,
) {
    // Load the test scene from YAML
    let scene_path = "packages/stalkerlike/assets/scenes/test_scene.yaml";

    info!("Attempting to load scene from: {}", scene_path);
    info!("Current directory: {:?}", std::env::current_dir());

    match load_scene_from_yaml(scene_path, &mut commands, &mut meshes, &mut materials) {
        Ok(_) => {
            info!("✅ Successfully loaded scene from {}", scene_path);
        }
        Err(e) => {
            error!("❌ Failed to load scene from {}: {}", scene_path, e);
        }
    }
}

/// System that spawns the player at a PlayerSpawnMarker position
fn spawn_player_at_marker(
    commands: Commands,
    spawn_query: Query<&Transform, With<PlayerSpawnMarker>>,
) {
    // Find player spawn position from the scene
    let spawn_position = if let Ok(spawn_transform) = spawn_query.single() {
        // Use the spawn marker's position, adjusted for player capsule height
        spawn_transform.translation + Vec3::new(0.0, 0.9, 0.0)
    } else {
        // Fallback to default position if no spawn marker found
        warn!("No PlayerSpawn marker found in scene, using default position");
        Vec3::new(0.0, 0.9, 0.0)
    };

    info!("Spawning player at position: {:?}", spawn_position);
    player::setup_player(commands, spawn_position);
}

/// Load a scene from a YAML file for the game mode
/// Similar to the editor's load_scene but uses GameEntity instead of EditorEntity
fn load_scene_from_yaml(
    path: impl AsRef<std::path::Path>,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) -> Result<(), Box<dyn std::error::Error>> {
    use serde::{Deserialize, Serialize};

    // Import scene data structures from editor
    // We'll define minimal versions here to avoid coupling to editor internals
    #[derive(Serialize, Deserialize, Debug, Clone)]
    struct SceneData {
        metadata: SceneMetadata,
        entities: Vec<EntityData>,
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    struct SceneMetadata {
        version: u32,
        name: Option<String>,
        description: Option<String>,
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    struct EntityData {
        name: Option<String>,
        transform: TransformData,
        components: Vec<ComponentData>,
    }

    #[derive(Serialize, Deserialize, Debug, Clone, Copy)]
    struct TransformData {
        position: [f32; 3],
        rotation: [f32; 4],
        scale: [f32; 3],
    }

    impl From<TransformData> for Transform {
        fn from(data: TransformData) -> Self {
            Transform {
                translation: Vec3::from_array(data.position),
                rotation: Quat::from_xyzw(
                    data.rotation[0],
                    data.rotation[1],
                    data.rotation[2],
                    data.rotation[3],
                ),
                scale: Vec3::from_array(data.scale),
            }
        }
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    #[serde(tag = "type")]
    enum ComponentData {
        Mesh { primitive_type: PrimitiveTypeSerde },
        Material { base_color: [f32; 4] },
        PlayerSpawn,
        RigidBody { body_type: RigidBodyTypeSerde },
    }

    #[derive(Serialize, Deserialize, Debug, Clone, Copy)]
    enum PrimitiveTypeSerde {
        Cube,
        Sphere,
        Plane,
        Cylinder,
        Capsule,
        PlayerSpawn,
    }

    #[derive(Serialize, Deserialize, Debug, Clone, Copy)]
    enum RigidBodyTypeSerde {
        Fixed,
        Dynamic,
    }

    // Helper to create meshes for each primitive type
    fn create_primitive_mesh(prim_type: PrimitiveTypeSerde, size: Vec3) -> Mesh {
        match prim_type {
            PrimitiveTypeSerde::Cube => Cuboid::new(size.x, size.y, size.z).into(),
            PrimitiveTypeSerde::Sphere => Sphere::new(size.x / 2.0).mesh().ico(32).unwrap().into(),
            PrimitiveTypeSerde::Plane => Plane3d::default().mesh().size(size.x, size.z).into(),
            PrimitiveTypeSerde::Cylinder => Cylinder::new(size.x / 2.0, size.y).into(),
            PrimitiveTypeSerde::Capsule => {
                let radius = size.x / 2.0;
                let half_height = (size.y / 2.0) - radius;
                Capsule3d::new(radius, half_height.max(0.001)).into()
            }
            PrimitiveTypeSerde::PlayerSpawn => {
                // Create an upward-pointing cone for player spawn marker
                Cone {
                    radius: size.x / 2.0,
                    height: size.y,
                }.into()
            }
        }
    }

    fn get_default_size(prim_type: PrimitiveTypeSerde) -> Vec3 {
        match prim_type {
            PrimitiveTypeSerde::Cube => Vec3::ONE,
            PrimitiveTypeSerde::Sphere => Vec3::splat(1.0),
            PrimitiveTypeSerde::Plane => Vec3::new(10.0, 0.1, 10.0),
            PrimitiveTypeSerde::Cylinder => Vec3::new(1.0, 2.0, 1.0),
            PrimitiveTypeSerde::Capsule => Vec3::new(0.5, 2.0, 0.5),
            PrimitiveTypeSerde::PlayerSpawn => Vec3::new(0.5, 2.0, 0.5),
        }
    }

    // Read and parse the YAML file
    let yaml = std::fs::read_to_string(path)?;
    let scene_data: SceneData = serde_yaml::from_str(&yaml)?;

    // Spawn entities from the scene
    for entity_data in scene_data.entities {
        let mut entity_commands = commands.spawn(GameEntity);

        // Add transform
        entity_commands.insert(Transform::from(entity_data.transform));

        // Add name if present
        if let Some(name) = entity_data.name {
            entity_commands.insert(Name::new(name));
        }

        // Process components
        let mut mesh_type: Option<PrimitiveTypeSerde> = None;
        let mut base_color: Option<Color> = None;
        let mut is_player_spawn = false;
        let mut rigid_body_type: Option<RigidBodyTypeSerde> = None;

        for component in entity_data.components {
            match component {
                ComponentData::Mesh { primitive_type } => {
                    mesh_type = Some(primitive_type);
                }
                ComponentData::Material { base_color: color } => {
                    base_color = Some(Color::srgba(
                        color[0],
                        color[1],
                        color[2],
                        color[3],
                    ));
                }
                ComponentData::PlayerSpawn => {
                    is_player_spawn = true;
                }
                ComponentData::RigidBody { body_type } => {
                    rigid_body_type = Some(body_type);
                }
            }
        }

        // Handle player spawn differently - don't render the cone, just mark the spawn point
        if is_player_spawn {
            // Only add the marker component, no mesh or collider
            entity_commands.insert(PlayerSpawnMarker);
        } else if let Some(prim_type) = mesh_type {
            // Create mesh at the primitive's default size - Transform.scale handles any scaling
            let default_size = get_default_size(prim_type);
            let mesh = create_primitive_mesh(prim_type, default_size);
            entity_commands.insert(Mesh3d(meshes.add(mesh)));

            // Add material
            let color = base_color.unwrap_or(Color::srgb(0.7, 0.7, 0.7));
            entity_commands.insert(MeshMaterial3d(materials.add(StandardMaterial {
                base_color: color,
                ..default()
            })));

            // Add physics colliders for scene geometry
            // Default to Fixed if no rigid body type specified
            let rb = match rigid_body_type.unwrap_or(RigidBodyTypeSerde::Fixed) {
                RigidBodyTypeSerde::Fixed => RigidBody::Fixed,
                RigidBodyTypeSerde::Dynamic => RigidBody::Dynamic,
            };

            match prim_type {
                PrimitiveTypeSerde::Cube => {
                    entity_commands.insert((
                        rb,
                        Collider::cuboid(default_size.x / 2.0, default_size.y / 2.0, default_size.z / 2.0),
                    ));
                }
                PrimitiveTypeSerde::Sphere => {
                    entity_commands.insert((
                        rb,
                        Collider::ball(default_size.x / 2.0),
                    ));
                }
                PrimitiveTypeSerde::Plane => {
                    entity_commands.insert((
                        rb,
                        Collider::cuboid(default_size.x / 2.0, 0.1, default_size.z / 2.0),
                    ));
                }
                PrimitiveTypeSerde::Cylinder => {
                    entity_commands.insert((
                        rb,
                        Collider::cylinder(default_size.y / 2.0, default_size.x / 2.0),
                    ));
                }
                PrimitiveTypeSerde::Capsule => {
                    let radius = default_size.x / 2.0;
                    let half_height = (default_size.y / 2.0) - radius;
                    entity_commands.insert((
                        rb,
                        Collider::capsule_y(half_height.max(0.001), radius),
                    ));
                }
                PrimitiveTypeSerde::PlayerSpawn => {
                    // This shouldn't happen since we check is_player_spawn first
                }
            }
        }
    }

    Ok(())
}

/// Marker component for player spawn points in game mode
#[derive(Component)]
pub struct PlayerSpawnMarker;

#[derive(Component)]
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
