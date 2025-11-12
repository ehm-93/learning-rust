//! Scene data structures and serialization logic

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::editor::core::types::{EditorEntity, PlayerSpawn, RigidBodyType, GlbModel};
use crate::editor::objects::primitives::PrimitiveType;

/// Root scene data structure
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SceneData {
    /// Scene metadata
    pub metadata: SceneMetadata,
    /// Global scene properties (lighting, etc.)
    #[serde(default)]
    pub global: GlobalData,
    /// List of entities in the scene
    pub entities: Vec<EntityData>,
}

/// Scene metadata
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SceneMetadata {
    /// Scene version for compatibility tracking
    pub version: u32,
    /// Optional scene name
    pub name: Option<String>,
    /// Optional description
    pub description: Option<String>,
}

impl Default for SceneMetadata {
    fn default() -> Self {
        Self {
            version: 1,
            name: None,
            description: None,
        }
    }
}

/// Global scene properties (lighting, fog, etc.)
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct GlobalData {
    /// Lighting settings
    pub lighting: LightingData,
}

impl Default for GlobalData {
    fn default() -> Self {
        Self {
            lighting: LightingData::default(),
        }
    }
}

/// Global lighting data for the scene
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct LightingData {
    /// Directional light settings
    pub directional: DirectionalLightData,
    /// Ambient light settings
    pub ambient: AmbientLightData,
}

impl Default for LightingData {
    fn default() -> Self {
        Self {
            directional: DirectionalLightData::default(),
            ambient: AmbientLightData::default(),
        }
    }
}

/// Directional light data
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct DirectionalLightData {
    /// Light illuminance in lux
    pub illuminance: f32,
    /// Light color (RGBA)
    pub color: [f32; 4],
    /// Direction (as transform: position and look-at)
    pub position: [f32; 3],
    pub look_at: [f32; 3],
}

impl Default for DirectionalLightData {
    fn default() -> Self {
        Self {
            illuminance: 10000.0,
            color: [1.0, 1.0, 1.0, 1.0],
            position: [4.0, 8.0, 4.0],
            look_at: [0.0, 0.0, 0.0],
        }
    }
}

/// Ambient light data
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct AmbientLightData {
    /// Ambient light color (RGBA)
    pub color: [f32; 4],
    /// Ambient light brightness
    pub brightness: f32,
}

impl Default for AmbientLightData {
    fn default() -> Self {
        Self {
            color: [1.0, 1.0, 1.0, 1.0],
            brightness: 400.0,
        }
    }
}

/// Serializable entity representation
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EntityData {
    /// Optional entity name for identification
    pub name: Option<String>,
    /// Transform component data
    pub transform: TransformData,
    /// Components attached to this entity
    pub components: Vec<ComponentData>,
}

/// Serializable transform data
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct TransformData {
    pub position: [f32; 3],
    pub rotation: [f32; 4], // Quaternion (x, y, z, w)
    pub scale: [f32; 3],
}

impl From<&Transform> for TransformData {
    fn from(transform: &Transform) -> Self {
        Self {
            position: transform.translation.to_array(),
            rotation: [
                transform.rotation.x,
                transform.rotation.y,
                transform.rotation.z,
                transform.rotation.w,
            ],
            scale: transform.scale.to_array(),
        }
    }
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

/// Extensible component data enum
/// New component types can be added here as the editor evolves
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum ComponentData {
    /// Mesh component data
    Mesh { primitive_type: PrimitiveTypeSerde },
    /// Material component data (base color only for now)
    Material { base_color: [f32; 4] },
    /// Player spawn marker
    PlayerSpawn,
    /// Rigid body physics type
    RigidBody { body_type: RigidBodyTypeSerde },
    /// GLB/GLTF model component
    GlbModel { path: String },
}

/// Serializable primitive type
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum PrimitiveTypeSerde {
    Cube,
    Sphere,
    Plane,
    Cylinder,
    Capsule,
    PlayerSpawn,
}

/// Serializable rigid body type
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum RigidBodyTypeSerde {
    Fixed,
    Dynamic,
}

impl From<RigidBodyType> for RigidBodyTypeSerde {
    fn from(rb: RigidBodyType) -> Self {
        match rb {
            RigidBodyType::Fixed => RigidBodyTypeSerde::Fixed,
            RigidBodyType::Dynamic => RigidBodyTypeSerde::Dynamic,
        }
    }
}

impl From<RigidBodyTypeSerde> for RigidBodyType {
    fn from(rbs: RigidBodyTypeSerde) -> Self {
        match rbs {
            RigidBodyTypeSerde::Fixed => RigidBodyType::Fixed,
            RigidBodyTypeSerde::Dynamic => RigidBodyType::Dynamic,
        }
    }
}

impl From<PrimitiveType> for PrimitiveTypeSerde {
    fn from(pt: PrimitiveType) -> Self {
        match pt {
            PrimitiveType::Cube => PrimitiveTypeSerde::Cube,
            PrimitiveType::Sphere => PrimitiveTypeSerde::Sphere,
            PrimitiveType::Plane => PrimitiveTypeSerde::Plane,
            PrimitiveType::Cylinder => PrimitiveTypeSerde::Cylinder,
            PrimitiveType::Capsule => PrimitiveTypeSerde::Capsule,
            PrimitiveType::PlayerSpawn => PrimitiveTypeSerde::PlayerSpawn,
        }
    }
}

impl From<PrimitiveTypeSerde> for PrimitiveType {
    fn from(pts: PrimitiveTypeSerde) -> Self {
        match pts {
            PrimitiveTypeSerde::Cube => PrimitiveType::Cube,
            PrimitiveTypeSerde::Sphere => PrimitiveType::Sphere,
            PrimitiveTypeSerde::Plane => PrimitiveType::Plane,
            PrimitiveTypeSerde::Cylinder => PrimitiveType::Cylinder,
            PrimitiveTypeSerde::Capsule => PrimitiveType::Capsule,
            PrimitiveTypeSerde::PlayerSpawn => PrimitiveType::PlayerSpawn,
        }
    }
}

/// Save the current scene to a YAML file
pub fn save_scene(
    path: impl AsRef<Path>,
    editor_entities: Query<(
        Entity,
        &Transform,
        Option<&Name>,
        Option<&Mesh3d>,
        Option<&MeshMaterial3d<StandardMaterial>>,
        Option<&PlayerSpawn>,
        Option<&RigidBodyType>,
        Option<&GlbModel>,
    ), With<EditorEntity>>,
    meshes: Res<Assets<Mesh>>,
    materials: Res<Assets<StandardMaterial>>,
    directional_light: Query<(&DirectionalLight, &Transform), Without<EditorEntity>>,
    ambient_light: Res<AmbientLight>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut entities = Vec::new();

    for (_entity, transform, name, mesh_handle, material_handle, player_spawn, rigid_body, glb_model) in editor_entities.iter() {
        let mut components = Vec::new();

        // Serialize mesh if present
        if let Some(mesh3d) = mesh_handle {
            if let Some(mesh) = meshes.get(&mesh3d.0) {
                // Try to identify the primitive type from the mesh
                // For MVP, we'll use a simple heuristic based on vertex count
                // In the future, we should store this metadata on the entity
                let primitive_type = identify_primitive_type(mesh);
                components.push(ComponentData::Mesh {
                    primitive_type: primitive_type.into(),
                });
            }
        }

        // Serialize material if present
        if let Some(material3d) = material_handle {
            if let Some(material) = materials.get(&material3d.0) {
                components.push(ComponentData::Material {
                    base_color: material.base_color.to_srgba().to_f32_array(),
                });
            }
        }

        // Serialize player spawn marker if present
        if player_spawn.is_some() {
            components.push(ComponentData::PlayerSpawn);
        }

        // Serialize rigid body type if present
        if let Some(&rb_type) = rigid_body {
            components.push(ComponentData::RigidBody {
                body_type: rb_type.into(),
            });
        }

        // Serialize GLB model if present
        if let Some(glb) = glb_model {
            components.push(ComponentData::GlbModel {
                path: glb.path.to_string_lossy().to_string(),
            });
        }

        entities.push(EntityData {
            name: name.map(|n| n.to_string()),
            transform: transform.into(),
            components,
        });
    }

    // Capture lighting state (always saves the custom lighting values, not the toggle state)
    let lighting = if let Ok((dir_light, dir_transform)) = directional_light.single() {
        LightingData {
            directional: DirectionalLightData {
                illuminance: dir_light.illuminance,
                color: dir_light.color.to_srgba().to_f32_array(),
                position: dir_transform.translation.to_array(),
                look_at: (dir_transform.translation + dir_transform.forward() * 10.0).to_array(),
            },
            ambient: AmbientLightData {
                color: ambient_light.color.to_srgba().to_f32_array(),
                brightness: ambient_light.brightness,
            },
        }
    } else {
        // Use defaults if no directional light exists
        LightingData::default()
    };

    let scene_data = SceneData {
        metadata: SceneMetadata::default(),
        global: GlobalData { lighting },
        entities,
    };

    let yaml = serde_yaml::to_string(&scene_data)?;
    fs::write(path, yaml)?;

    Ok(())
}

/// Load a scene from a YAML file
pub fn load_scene(
    path: impl AsRef<Path>,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    asset_server: &Res<AssetServer>,
) -> Result<(), Box<dyn std::error::Error>> {
    let yaml = fs::read_to_string(path)?;
    let scene_data: SceneData = serde_yaml::from_str(&yaml)?;

    // Apply lighting settings
    // Spawn or update directional light
    commands.spawn((
        DirectionalLight {
            illuminance: scene_data.global.lighting.directional.illuminance,
            color: Color::srgba(
                scene_data.global.lighting.directional.color[0],
                scene_data.global.lighting.directional.color[1],
                scene_data.global.lighting.directional.color[2],
                scene_data.global.lighting.directional.color[3],
            ),
            shadows_enabled: true,
            ..default()
        },
        Transform::from_translation(Vec3::from_array(scene_data.global.lighting.directional.position))
            .looking_at(Vec3::from_array(scene_data.global.lighting.directional.look_at), Vec3::Y),
    ));

    // Update ambient light resource
    commands.insert_resource(AmbientLight {
        color: Color::srgba(
            scene_data.global.lighting.ambient.color[0],
            scene_data.global.lighting.ambient.color[1],
            scene_data.global.lighting.ambient.color[2],
            scene_data.global.lighting.ambient.color[3],
        ),
        brightness: scene_data.global.lighting.ambient.brightness,
        ..default()
    });

    for entity_data in scene_data.entities {
        let mut entity_commands = commands.spawn(EditorEntity);

        // Add transform
        entity_commands.insert(Transform::from(entity_data.transform));

        // Add name if present
        if let Some(name) = entity_data.name {
            entity_commands.insert(Name::new(name));
        }

        // Process components
        let mut mesh_type: Option<PrimitiveType> = None;
        let mut base_color: Option<Color> = None;
        let mut is_player_spawn = false;
        let mut rigid_body_type: Option<RigidBodyType> = None;
        let mut glb_model_path: Option<String> = None;

        for component in entity_data.components {
            match component {
                ComponentData::Mesh { primitive_type } => {
                    mesh_type = Some(primitive_type.into());
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
                    rigid_body_type = Some(body_type.into());
                }
                ComponentData::GlbModel { path } => {
                    glb_model_path = Some(path);
                }
            }
        }

        // Spawn mesh and material if we have the data
        if let Some(prim_type) = mesh_type {
            // Create mesh at the primitive's default size - Transform.scale handles any scaling
            let mesh = prim_type.create_mesh(prim_type.default_size());
            entity_commands.insert(Mesh3d(meshes.add(mesh)));

            // Add material
            let color = base_color.unwrap_or(Color::srgb(0.7, 0.7, 0.7));
            entity_commands.insert(MeshMaterial3d(materials.add(StandardMaterial {
                base_color: color,
                ..default()
            })));
        }

        // Add PlayerSpawn component if marked
        if is_player_spawn {
            entity_commands.insert(PlayerSpawn);
        }

        // Add RigidBodyType component if present
        if let Some(rb_type) = rigid_body_type {
            entity_commands.insert(rb_type);
        }

        // Add GLB model if present
        if let Some(glb_path) = glb_model_path {
            let scene_handle = asset_server.load(format!("{}#Scene0", glb_path));
            entity_commands.insert((
                GlbModel { path: std::path::PathBuf::from(&glb_path) },
                SceneRoot(scene_handle),
                Visibility::Inherited,
            ));
        }
    }

    Ok(())
}

/// Heuristic to identify primitive type from a mesh
/// This is a temporary solution - ideally we'd store this metadata on the entity
fn identify_primitive_type(mesh: &Mesh) -> PrimitiveType {
    // Count vertices to identify the primitive
    let vertex_count = if let Some(bevy::render::mesh::VertexAttributeValues::Float32x3(positions)) =
        mesh.attribute(Mesh::ATTRIBUTE_POSITION)
    {
        positions.len()
    } else {
        0
    };

    // Rough heuristic based on typical vertex counts
    match vertex_count {
        24 => PrimitiveType::Cube,       // 4 vertices per face * 6 faces
        4 => PrimitiveType::Plane,        // Simple plane
        _ if vertex_count > 100 => PrimitiveType::Sphere, // Ico sphere has many vertices
        _ if vertex_count > 40 => PrimitiveType::Cylinder, // Cylinder has circular ends
        _ => PrimitiveType::Capsule,     // Fallback
    }
}
