//! Object placement system
//!
//! This module handles the interactive placement of new objects in the editor viewport.
//! Users can select a primitive or GLB model from the asset browser, and a semi-transparent
//! preview follows their cursor. Clicking places the object at that location.
//!
//! # Features
//!
//! - **Preview entity**: Semi-transparent ghost of object follows cursor
//! - **Grid snapping**: Automatically aligns placement to grid when enabled
//! - **Ray-plane intersection**: Uses viewport raycasting to determine placement position
//! - **Multiple asset types**: Supports both primitive shapes and GLB models
//!
//! # Workflow
//!
//! 1. User selects asset from asset browser/hierarchy panel
//! 2. `start_placement()` creates a preview entity
//! 3. `update_preview_position()` runs each frame to position preview under cursor
//! 4. `place_object()` creates final entity on click, removes preview
//! 5. ESC cancels placement mode

use bevy::prelude::*;
use bevy::asset::LoadState;
use std::path::PathBuf;

use crate::editor::core::types::{EditorEntity, PlayerSpawn, GlbModel, EditorLight, LightType, EditorVisualization, MissingAsset};
use crate::editor::viewport::{camera::EditorCamera, grid::{snap_to_grid, GridConfig}, raycasting::ray_plane_intersection};
use crate::editor::objects::primitives::{PrimitiveDefinition, PrimitiveType};

/// Type of asset being placed
#[derive(Clone, Debug)]
pub enum PlacementAsset {
    Primitive(PrimitiveDefinition),
    GlbModel {
        name: String,
        path: PathBuf,
    },
}

/// Resource tracking the current placement state
#[derive(Resource, Default)]
pub struct PlacementState {
    pub active: bool,
    pub preview_entity: Option<Entity>,
    pub selected_primitive: Option<PrimitiveDefinition>,
    pub selected_asset: Option<PlacementAsset>,
}

/// Marker component for preview entities
#[derive(Component)]
pub struct PreviewEntity;

/// Start placing a primitive
pub fn start_placement(
    placement_state: &mut PlacementState,
    primitive: PrimitiveDefinition,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    start_placement_asset(
        placement_state,
        PlacementAsset::Primitive(primitive),
        commands,
        meshes,
        materials,
        None,
    );
}

/// Start placing any asset (primitive or GLB)
pub fn start_placement_asset(
    placement_state: &mut PlacementState,
    asset: PlacementAsset,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    asset_server: Option<&Res<AssetServer>>,
) {
    // Clean up any existing preview
    if let Some(entity) = placement_state.preview_entity {
        commands.entity(entity).despawn();
    }

    let preview = match &asset {
        PlacementAsset::Primitive(primitive) => {
            // Create preview entity for primitive
            let mesh = primitive.primitive_type.create_mesh(primitive.default_size);
            let mut material_color = primitive.color;
            material_color.set_alpha(0.5); // Semi-transparent

            commands
                .spawn((
                    PreviewEntity,
                    Mesh3d(meshes.add(mesh)),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: material_color,
                        alpha_mode: AlphaMode::Blend,
                        ..default()
                    })),
                    Transform::from_xyz(0.0, 0.5, 0.0),
                ))
                .id()
        }
        PlacementAsset::GlbModel { path, .. } => {
            // Create preview entity for GLB model
            if let Some(asset_server) = asset_server {
                let glb_path_str = path.to_string_lossy().to_string();
                let scene_handle = asset_server.load(format!("{}#Scene0", glb_path_str));

                commands
                    .spawn((
                        PreviewEntity,
                        SceneRoot(scene_handle),
                        Transform::from_xyz(0.0, 0.5, 0.0),
                        Visibility::Inherited,
                    ))
                    .id()
            } else {
                warn!("AssetServer not provided for GLB preview");
                return;
            }
        }
    };

    placement_state.preview_entity = Some(preview);

    // Preserve backward compatibility
    if let PlacementAsset::Primitive(primitive) = &asset {
        placement_state.selected_primitive = Some(primitive.clone());
    } else {
        placement_state.selected_primitive = None;
    }

    placement_state.selected_asset = Some(asset);
    placement_state.active = true;
}

/// Update preview position to follow mouse cursor
pub fn update_preview_position(
    camera_query: Query<(&Camera, &GlobalTransform), With<EditorCamera>>,
    windows: Query<&Window>,
    mut preview_query: Query<&mut Transform, With<PreviewEntity>>,
    placement_state: Res<PlacementState>,
    grid_config: Res<GridConfig>,
) {
    if !placement_state.active {
        return;
    }

    let Ok((camera, camera_transform)) = camera_query.single() else {
        return;
    };

    let Ok(window) = windows.single() else {
        return;
    };

    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    // Cast ray from camera through cursor
    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
        return;
    };

    // Intersect with ground plane (Y=0)
    if let Some(distance) = ray_plane_intersection(ray.origin, ray.direction.as_vec3(), Vec3::ZERO, Vec3::Y) {
        let mut point = ray.origin + ray.direction.as_vec3() * distance;

        // Apply grid snapping if enabled
        if grid_config.snap_enabled {
            point = snap_to_grid(point, grid_config.spacing);
        }

        // Update preview position
        for mut transform in preview_query.iter_mut() {
            transform.translation = point;
        }
    }
}

/// Place the object on mouse click
pub fn place_object(
    mut commands: Commands,
    mouse_input: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut placement_state: ResMut<PlacementState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    preview_query: Query<&Transform, With<PreviewEntity>>,
    camera_query: Query<&EditorCamera>,
) {
    if !placement_state.active {
        return;
    }

    // Check if camera mouse is locked (if so, we shouldn't be placing)
    if let Ok(camera) = camera_query.single() {
        if camera.mouse_locked {
            return;
        }
    }

    // Cancel placement on ESC
    if keyboard.just_pressed(KeyCode::Escape) {
        if let Some(entity) = placement_state.preview_entity {
            commands.entity(entity).despawn();
        }
        placement_state.active = false;
        placement_state.preview_entity = None;
        placement_state.selected_primitive = None;
        placement_state.selected_asset = None;
        return;
    }

    // Place object on left click
    if mouse_input.just_pressed(MouseButton::Left) {
        if let Some(asset) = &placement_state.selected_asset {
            if let Ok(preview_transform) = preview_query.single() {
                match asset {
                    PlacementAsset::Primitive(primitive) => {
                        // Check if this is a light primitive - lights need special handling
                        let is_light = matches!(primitive.primitive_type, PrimitiveType::PointLight | PrimitiveType::SpotLight);

                        if is_light {
                            // For lights, spawn without mesh/material initially
                            let light_entity = match primitive.primitive_type {
                                PrimitiveType::PointLight => {
                                    commands.spawn((
                                        EditorEntity,
                                        EditorLight { light_type: LightType::Point },
                                        PointLight {
                                            intensity: 1000.0,
                                            color: Color::srgb(1.0, 1.0, 0.9),
                                            shadows_enabled: true,
                                            ..default()
                                        },
                                        Name::new("Point Light"),
                                        *preview_transform,
                                    )).id()
                                }
                                PrimitiveType::SpotLight => {
                                    commands.spawn((
                                        EditorEntity,
                                        EditorLight { light_type: LightType::Spot },
                                        SpotLight {
                                            intensity: 1000.0,
                                            color: Color::srgb(1.0, 1.0, 0.9),
                                            shadows_enabled: true,
                                            inner_angle: 0.6,
                                            outer_angle: 0.8,
                                            ..default()
                                        },
                                        Name::new("Spot Light"),
                                        *preview_transform,
                                    )).id()
                                }
                                _ => Entity::PLACEHOLDER,
                            };

                            // Spawn visualization mesh as a separate child entity
                            let viz_mesh = primitive.primitive_type.create_mesh(primitive.default_size);
                            let viz_entity = commands.spawn((
                                EditorVisualization,
                                Mesh3d(meshes.add(viz_mesh)),
                                MeshMaterial3d(materials.add(StandardMaterial {
                                    base_color: primitive.color,
                                    ..default()
                                })),
                                Transform::default(),
                            )).id();

                            // Make visualization a child of the light
                            commands.entity(light_entity).add_child(viz_entity);

                            info!("Placed {} at {:?}", primitive.name, preview_transform.translation);
                        } else {
                            // Non-light primitives: spawn normally with mesh
                            let mesh = primitive.primitive_type.create_mesh(primitive.default_size);
                            let mut entity_commands = commands.spawn((
                                EditorEntity,
                                Mesh3d(meshes.add(mesh)),
                                MeshMaterial3d(materials.add(StandardMaterial {
                                    base_color: primitive.color,
                                    ..default()
                                })),
                                *preview_transform,
                            ));

                            // Add PlayerSpawn component if this is a player spawn marker
                            if primitive.primitive_type == PrimitiveType::PlayerSpawn {
                                entity_commands.insert(PlayerSpawn);
                                entity_commands.insert(Name::new("Player Spawn"));
                            }

                            info!("Placed {} at {:?}", primitive.name, preview_transform.translation);
                        }
                    }
                    PlacementAsset::GlbModel { name, path } => {
                        // Check if asset exists before spawning
                        let glb_path_str = path.to_string_lossy().to_string();
                        let glb_handle: Handle<Scene> = asset_server.load(format!("{}#Scene0", glb_path_str));
                        
                        match asset_server.get_load_state(&glb_handle) {
                            Some(LoadState::Failed(_)) => {
                                // Asset failed to load - spawn red error cube
                                error!("Failed to load GLB asset: {}", glb_path_str);
                                
                                let error_mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
                                let error_material = materials.add(StandardMaterial {
                                    base_color: Color::srgb(1.0, 0.0, 0.0),
                                    emissive: Color::srgb(1.0, 0.0, 0.0).into(),
                                    ..default()
                                });
                                
                                commands.spawn((
                                    EditorEntity,
                                    MissingAsset { path: path.clone() },
                                    Name::new(format!("MISSING: {}", name)),
                                    Mesh3d(error_mesh),
                                    MeshMaterial3d(error_material),
                                    *preview_transform,
                                ));
                                
                                info!("Spawned error placeholder for missing asset: {}", glb_path_str);
                            }
                            _ => {
                                // Asset exists or is loading - spawn normally
                                commands.spawn((
                                    EditorEntity,
                                    GlbModel { path: path.clone() },
                                    Name::new(name.clone()),
                                    SceneRoot(glb_handle),
                                    *preview_transform,
                                    Visibility::Inherited,
                                ));
                                
                                info!("Placed GLB {} at {:?}", name, preview_transform.translation);
                            }
                        }
                    }
                }
            }
        }

        // Continue placement mode for multiple placements
        // User needs to press ESC to exit
    }
}
