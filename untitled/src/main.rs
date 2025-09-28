use bevy::prelude::*;

use avian2d::prelude::*;

#[derive(Component)]
struct Player;

const PLAYER_SPEED: f32 = 300.0;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Combat Sandbox - Player Movement".to_string(),
                    resolution: (800.0, 600.0).into(),
                    ..default()
                }),
                ..default()
            }),
            PhysicsPlugins::default(),
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, player_movement)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Spawn 2D camera
    commands.spawn(Camera2d);

    // Spawn player as a white circle
    commands.spawn((
        Mesh2d(meshes.add(Circle::new(10.0))),
        MeshMaterial2d(materials.add(Color::WHITE)),
        Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        Player,
        RigidBody::Dynamic,
        Collider::circle(10.0),
        // Lock rotation so the player doesn't spin
        LockedAxes::ROTATION_LOCKED,
    ));
}

fn player_movement(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut LinearVelocity, With<Player>>,
) {
    for mut velocity in query.iter_mut() {
        let mut direction = Vec2::ZERO;

        // Check WASD input
        if keyboard_input.pressed(KeyCode::KeyW) {
            direction.y += 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyS) {
            direction.y -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyA) {
            direction.x -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyD) {
            direction.x += 1.0;
        }

        // Normalize diagonal movement
        if direction != Vec2::ZERO {
            direction = direction.normalize();
        }

        // Apply velocity
        velocity.0 = direction * PLAYER_SPEED;
    }
}
