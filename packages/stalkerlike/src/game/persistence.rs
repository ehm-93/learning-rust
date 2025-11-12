use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task};
use bevy_rapier3d::prelude::*;
use bevy_egui::PrimaryEguiContext;
use rusqlite::Connection;
use futures_lite::future;

use super::components::*;
use super::resources::*;

pub struct PersistencePlugin;

impl Plugin for PersistencePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<SaveGameEvent>()
            .add_event::<LoadGameEvent>()

            // Database initialization
            .add_systems(Startup, init_database)

            // Save/load systems - run continuously to process events
            .add_systems(
                Update,
                (
                    handle_save_game,
                    handle_load_request,
                    poll_load_task,
                    check_loading_complete,
                )
            );
    }
}

#[derive(Component)]
struct LoadTask(Task<Option<SavedGameState>>);

struct SavedGameState {
    player: Option<PlayerData>,
    physics_objects: Vec<PhysicsObjectData>,
}

struct PlayerData {
    position: Vec3,
    health: f32,
    pitch: f32,
    yaw: f32,
}

struct PhysicsObjectData {
    position: Vec3,
    rotation: Quat,
    velocity: Vec3,
}

fn init_database(save_path: Res<SavePath>) {
    if let Ok(conn) = Connection::open(&save_path.0) {
        // Player save table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS player_save (
                slot INTEGER PRIMARY KEY,
                position_x REAL,
                position_y REAL,
                position_z REAL,
                health REAL,
                pitch REAL,
                yaw REAL
            )",
            [],
        ).ok();

        // Physics objects save table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS physics_objects (
                slot INTEGER,
                entity_id INTEGER,
                position_x REAL,
                position_y REAL,
                position_z REAL,
                velocity_x REAL,
                velocity_y REAL,
                velocity_z REAL,
                rotation_x REAL,
                rotation_y REAL,
                rotation_z REAL,
                rotation_w REAL,
                PRIMARY KEY (slot, entity_id)
            )",
            [],
        ).ok();
    }
}

/// Centralized save system - saves everything in one transaction
fn handle_save_game(
    mut events: EventReader<SaveGameEvent>,
    save_path: Res<SavePath>,
    world: &World,
    saveable_query: Query<Entity, With<Saveable>>,
) {
    for event in events.read() {
        if let Ok(mut conn) = Connection::open(&save_path.0) {
            // Start transaction for entire save operation
            let tx = match conn.transaction() {
                Ok(tx) => tx,
                Err(e) => {
                    error!("Failed to start save transaction: {}", e);
                    continue;
                }
            };

            let mut success = true;
            let mut saved_physics_count = 0;

            // Clear physics objects table for this slot
            if let Err(e) = tx.execute(
                "DELETE FROM physics_objects WHERE slot = ?1",
                [event.slot],
            ) {
                error!("Failed to clear physics objects: {}", e);
                continue;
            }

            // Save all saveable entities
            for entity in saveable_query.iter() {
                let entity_ref = world.entity(entity);

                info!("Saving entity {:?}", entity);

                // Is this the player?
                if entity_ref.contains::<Player>() {
                    info!("Saving player entity");
                    let transform = match entity_ref.get::<Transform>() {
                        Some(t) => t,
                        None => {
                            error!("Player entity missing Transform component");
                            success = false;
                            break;
                        }
                    };
                    let health = match entity_ref.get::<Health>() {
                        Some(h) => h,
                        None => {
                            error!("Player entity missing Health component");
                            success = false;
                            break;
                        }
                    };
                    let camera = match entity_ref.get::<PlayerCamera>() {
                        Some(c) => c,
                        None => {
                            error!("Player entity missing PlayerCamera component");
                            success = false;
                            break;
                        }
                    };

                    if let Err(e) = tx.execute(
                        "INSERT OR REPLACE INTO player_save
                            (slot, position_x, position_y, position_z, health, pitch, yaw)
                            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                        (
                            event.slot,
                            transform.translation.x,
                            transform.translation.y,
                            transform.translation.z,
                            health.current,
                            camera.pitch,
                            camera.yaw,
                        ),
                    ) {
                        error!("Failed to save player: {}", e);
                        success = false;
                        break;
                    }
                }
                // Is this a physics object?
                else if entity_ref.contains::<RigidBody>() {
                    let rigid_body = match entity_ref.get::<RigidBody>() {
                        Some(rb) => rb,
                        None => {
                            error!("Physics entity missing RigidBody component");
                            success = false;
                            break;
                        }
                    };

                    if *rigid_body != RigidBody::Dynamic {
                        error!("Only dynamic physics objects are saved");
                        continue;
                    }

                    let transform = match entity_ref.get::<Transform>() {
                        Some(t) => t,
                        None => {
                            error!("Physics entity missing Transform component");
                            success = false;
                            break;
                        }
                    };

                    let velocity = match entity_ref.get::<Velocity>() {
                        Some(v) => v,
                        None => {
                            &Velocity::default()
                        }
                    };

                    if let Err(e) = tx.execute(
                        "INSERT INTO physics_objects
                         (slot, entity_id, position_x, position_y, position_z,
                          velocity_x, velocity_y, velocity_z,
                          rotation_x, rotation_y, rotation_z, rotation_w)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
                        (
                            event.slot,
                            saved_physics_count,
                            transform.translation.x,
                            transform.translation.y,
                            transform.translation.z,
                            velocity.linvel.x,
                            velocity.linvel.y,
                            velocity.linvel.z,
                            transform.rotation.x,
                            transform.rotation.y,
                            transform.rotation.z,
                            transform.rotation.w,
                        ),
                    ) {
                        error!("Failed to save physics object {}: {}", saved_physics_count, e);
                        success = false;
                        break;
                    }
                    saved_physics_count += 1;
                } else {
                    error!("Saveable entity is neither Player nor Physics Object");
                    success = false;
                }
            }

            if success && saved_physics_count > 0 {
                info!("Saved {} physics objects", saved_physics_count);
            }

            // Commit or rollback transaction
            if success {
                match tx.commit() {
                    Ok(_) => info!("Game saved to slot {}", event.slot),
                    Err(e) => error!("Failed to commit save: {}", e),
                }
            } else {
                error!("Save failed, transaction rolled back");
            }
        }
    }
}

/// System to handle LoadGameEvent and transition to Loading state
fn handle_load_request(
    mut events: EventReader<LoadGameEvent>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    existing_task: Query<Entity, With<LoadTask>>,
    save_path: Res<SavePath>,
) {
    for event in events.read() {
        // Don't start a new load if one is already in progress
        if !existing_task.is_empty() {
            continue;
        }

        let slot = event.slot;
        let path = save_path.0.clone();

        // Initialize load progress
        let mut progress = LoadProgress::default();
        progress.register("Game Data");
        commands.insert_resource(progress);

        // Transition to Loading state
        next_state.set(GameState::Loading);

        // Spawn async load task
        let task_pool = AsyncComputeTaskPool::get();
        let task = task_pool.spawn(async move {
            match Connection::open(&path) {
                Ok(conn) => {
                    let mut game_state = SavedGameState {
                        player: None,
                        physics_objects: Vec::new(),
                    };

                    // Load player
                    if let Ok(mut stmt) = conn.prepare(
                        "SELECT position_x, position_y, position_z, health, pitch, yaw
                         FROM player_save WHERE slot = ?1"
                    ) {
                        game_state.player = stmt.query_row([slot], |row| {
                            Ok(PlayerData {
                                position: Vec3::new(
                                    row.get(0)?,
                                    row.get(1)?,
                                    row.get(2)?,
                                ),
                                health: row.get(3)?,
                                pitch: row.get(4)?,
                                yaw: row.get(5)?,
                            })
                        }).ok();
                    }

                    // Load physics objects
                    if let Ok(mut stmt) = conn.prepare(
                        "SELECT position_x, position_y, position_z,
                                velocity_x, velocity_y, velocity_z,
                                rotation_x, rotation_y, rotation_z, rotation_w
                         FROM physics_objects WHERE slot = ?1 ORDER BY entity_id"
                    ) {
                        if let Ok(objects_iter) = stmt.query_map([slot], |row| {
                            Ok(PhysicsObjectData {
                                position: Vec3::new(
                                    row.get(0)?,
                                    row.get(1)?,
                                    row.get(2)?,
                                ),
                                velocity: Vec3::new(
                                    row.get(3)?,
                                    row.get(4)?,
                                    row.get(5)?,
                                ),
                                rotation: Quat::from_xyzw(
                                    row.get(6)?,
                                    row.get(7)?,
                                    row.get(8)?,
                                    row.get(9)?,
                                ),
                            })
                        }) {
                            for obj_result in objects_iter {
                                if let Ok(obj) = obj_result {
                                    game_state.physics_objects.push(obj);
                                }
                            }
                        }
                    }

                    Some(game_state)
                }
                Err(_) => None,
            }
        });

        commands.spawn(LoadTask(task));
        info!("Started loading game from slot {}", slot);
    }
}

/// System to poll the load task and spawn entities when complete
fn poll_load_task(
    mut commands: Commands,
    mut task_query: Query<(Entity, &mut LoadTask)>,
    mut progress: Option<ResMut<LoadProgress>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, mut task) in task_query.iter_mut() {
        if let Some(result) = future::block_on(future::poll_once(&mut task.0)) {
            // Task is complete, remove task entity
            commands.entity(entity).despawn();

            if let Some(game_state) = result {
                // Spawn player if loaded
                if let Some(player_data) = game_state.player {
                    commands.spawn((
                        GameEntity,
                        Player,
                        Transform::from_translation(player_data.position),
                        PlayerCamera {
                            sensitivity: 0.002,
                            pitch: player_data.pitch,
                            yaw: player_data.yaw,
                        },
                        Health {
                            maximum: 100.0,
                            current: player_data.health,
                        },
                        RigidBody::Dynamic,
                        Collider::capsule_y(0.65, 0.25),
                        Velocity::default(),
                        LockedAxes::ROTATION_LOCKED,
                    ))
                    .with_children(|parent| {
                        parent.spawn((
                            Camera3d::default(),
                            PrimaryEguiContext,
                            Transform::from_xyz(0.0, 0.75, 0.0),
                        ))
                        .with_children(|camera_parent| {
                            camera_parent.spawn((
                                Flashlight::default(),
                                SpotLight {
                                    intensity: 0.0,
                                    range: 20.0,
                                    radius: 0.50,
                                    shadows_enabled: true,
                                    outer_angle: 0.3,
                                    inner_angle: 0.2,
                                    ..default()
                                },
                                Transform::from_xyz(0.3, -0.2, 0.0),
                            ));
                        });
                    });
                    info!("Player loaded successfully");
                }

                // Spawn physics objects
                for obj_data in game_state.physics_objects {
                    commands.spawn((
                        GameEntity,
                        Saveable,
                        Mesh3d(meshes.add(Sphere::new(0.5))),
                        MeshMaterial3d(materials.add(Color::srgb(0.8, 0.2, 0.2))),
                        Transform::from_translation(obj_data.position)
                            .with_rotation(obj_data.rotation),
                        RigidBody::Dynamic,
                        Collider::ball(0.5),
                        Restitution::coefficient(0.7),
                        Velocity {
                            linvel: obj_data.velocity,
                            angvel: Vec3::ZERO,
                        },
                    ));
                }

                info!("Game loaded successfully");
            } else {
                error!("Failed to load game data");
            }

            // Mark loading as complete
            if let Some(ref mut progress) = progress {
                progress.complete("Game Data");
            }
        }
    }
}

/// System to check if loading is complete and transition to InGame
fn check_loading_complete(
    progress: Option<Res<LoadProgress>>,
    mut next_state: ResMut<NextState<GameState>>,
    current_state: Res<State<GameState>>,
) {
    if *current_state.get() != GameState::Loading {
        return;
    }

    if let Some(progress) = progress {
        if progress.is_complete() && !progress.registered.is_empty() {
            info!("Loading complete! Transitioning to InGame");
            next_state.set(GameState::InGame);
        }
    }
}
