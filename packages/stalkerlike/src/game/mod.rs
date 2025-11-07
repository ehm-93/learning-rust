use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_rapier3d::prelude::*;

mod components;
mod persistence;
mod player;
mod resources;
mod ui;

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
            .add_plugins(RapierDebugRenderPlugin::default())

            // Game plugins
            .add_plugins(PlayerPlugin)
            .add_plugins(UiPlugin)
            .add_plugins(PersistencePlugin)

            // Game state
            .init_state::<GameState>()

            // Resources
            .insert_resource(SavePath::default())

            // Startup systems
            .add_systems(Startup, setup_world)

            // State transitions
            .add_systems(OnEnter(GameState::MainMenu), setup_menu_camera)
            .add_systems(OnExit(GameState::MainMenu), cleanup_menu_camera);
    }
}

#[derive(Component)]
struct MenuCamera;

fn setup_menu_camera(mut commands: Commands) {
    // Spawn a 2D camera for the main menu
    commands.spawn((
        Camera2d,
        MenuCamera,
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

fn setup_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Ground plane with physics collider
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(50.0, 50.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.3, 0.3, 0.3))),
        Transform::from_xyz(0.0, 0.0, 0.0),
        Collider::cuboid(25.0, 0.1, 25.0),
        Restitution::coefficient(1.0),
    ));

    // Static object (cube) with physics
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(2.0, 2.0, 2.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.8, 0.3, 0.3))),
        Transform::from_xyz(0.0, 1.0, -5.0),
        RigidBody::Fixed,
        Collider::cuboid(1.0, 1.0, 1.0),
    ));

    // Add some dynamic physics objects for testing
    for i in 0..5 {
        commands.spawn((
            Mesh3d(meshes.add(Sphere::new(0.5))),
            MeshMaterial3d(materials.add(Color::srgb(0.2, 0.7, 0.9))),
            Transform::from_xyz(i as f32 * 2.0 - 4.0, i as f32 * 5.0, 0.0),
            RigidBody::Dynamic,
            Collider::ball(0.5),
            Restitution::coefficient(0.85),
            Damping { linear_damping: 0.2, angular_damping: 0.2 },
        ));
    }

    // Ambient light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.0,
        affects_lightmapped_meshes: true,
    });

    // Directional light
    commands.spawn((
        DirectionalLight {
            illuminance: 0.0001,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}
