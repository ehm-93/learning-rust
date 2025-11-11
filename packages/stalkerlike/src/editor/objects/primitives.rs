use bevy::prelude::*;
use bevy::render::mesh::VertexAttributeValues;

/// Catalog of primitive meshes available in the editor
#[derive(Resource)]
pub struct AssetCatalog {
    pub primitives: Vec<PrimitiveDefinition>,
}

impl Default for AssetCatalog {
    fn default() -> Self {
        Self {
            primitives: vec![
                PrimitiveDefinition {
                    name: "Cube".to_string(),
                    primitive_type: PrimitiveType::Cube,
                    default_size: Vec3::ONE,
                    color: Color::srgb(0.7, 0.7, 0.7),
                },
                PrimitiveDefinition {
                    name: "Sphere".to_string(),
                    primitive_type: PrimitiveType::Sphere,
                    default_size: Vec3::splat(1.0), // 1m diameter
                    color: Color::srgb(0.6, 0.7, 0.8),
                },
                PrimitiveDefinition {
                    name: "Plane".to_string(),
                    primitive_type: PrimitiveType::Plane,
                    default_size: Vec3::new(10.0, 0.1, 10.0), // 10x10m floor section
                    color: Color::srgb(0.5, 0.6, 0.5),
                },
                PrimitiveDefinition {
                    name: "Cylinder".to_string(),
                    primitive_type: PrimitiveType::Cylinder,
                    default_size: Vec3::new(1.0, 2.0, 1.0), // 1m diameter x 2m height
                    color: Color::srgb(0.7, 0.6, 0.5),
                },
                PrimitiveDefinition {
                    name: "Capsule".to_string(),
                    primitive_type: PrimitiveType::Capsule,
                    default_size: Vec3::new(0.5, 2.0, 0.5), // 0.5m diameter x 2m height
                    color: Color::srgb(0.6, 0.6, 0.7),
                },
                PrimitiveDefinition {
                    name: "Player Spawn".to_string(),
                    primitive_type: PrimitiveType::PlayerSpawn,
                    default_size: Vec3::new(0.5, 2.0, 0.5), // Arrow: 0.5m wide x 2m tall
                    color: Color::srgb(0.2, 1.0, 0.2), // Bright green for visibility
                },
            ],
        }
    }
}

#[derive(Clone)]
pub struct PrimitiveDefinition {
    pub name: String,
    pub primitive_type: PrimitiveType,
    pub default_size: Vec3,
    pub color: Color,
}

#[derive(Clone, Copy, PartialEq)]
pub enum PrimitiveType {
    Cube,
    Sphere,
    Plane,
    Cylinder,
    Capsule,
    PlayerSpawn,
}

impl PrimitiveType {
    /// Get the default size for this primitive type
    pub fn default_size(&self) -> Vec3 {
        match self {
            PrimitiveType::Cube => Vec3::ONE,
            PrimitiveType::Sphere => Vec3::splat(1.0),
            PrimitiveType::Plane => Vec3::new(10.0, 0.1, 10.0),
            PrimitiveType::Cylinder => Vec3::new(1.0, 2.0, 1.0),
            PrimitiveType::Capsule => Vec3::new(0.5, 2.0, 0.5),
            PrimitiveType::PlayerSpawn => Vec3::new(0.5, 2.0, 0.5),
        }
    }

    pub fn create_mesh(&self, size: Vec3) -> Mesh {
        let mut mesh = match self {
            PrimitiveType::Cube => Cuboid::new(size.x, size.y, size.z).into(),
            PrimitiveType::Sphere => {
                // Use radius (half of diameter)
                Sphere::new(size.x / 2.0).mesh().ico(32).unwrap().into()
            }
            PrimitiveType::Plane => {
                // Plane mesh sized by X and Z
                Plane3d::default().mesh().size(size.x, size.z).into()
            }
            PrimitiveType::Cylinder => {
                // Cylinder with radius (half of diameter) and height
                Cylinder::new(size.x / 2.0, size.y).into()
            }
            PrimitiveType::Capsule => {
                // Capsule with radius (half of diameter) and total height
                // The height includes the hemispheres, so we need to adjust
                let radius = size.x / 2.0;
                let half_height = (size.y / 2.0) - radius;
                Capsule3d::new(radius, half_height.max(0.001)).into()
            }
            PrimitiveType::PlayerSpawn => {
                // Create a distinctive arrow pointing up (composed of cone + cylinder)
                // Use a cone for the arrow head and a thin cylinder for the shaft
                Self::create_arrow_mesh(size)
            }
        };

        // Add vertex colors (white by default, can be modified per-vertex later)
        Self::add_vertex_colors(&mut mesh);
        mesh
    }

    /// Add vertex color attribute to a mesh
    fn add_vertex_colors(mesh: &mut Mesh) {
        // Get the number of vertices
        let vertex_count = if let Some(VertexAttributeValues::Float32x3(positions)) =
            mesh.attribute(Mesh::ATTRIBUTE_POSITION)
        {
            positions.len()
        } else {
            return; // No positions, can't add colors
        };

        // Create white vertex colors for all vertices
        let colors: Vec<[f32; 4]> = vec![[1.0, 1.0, 1.0, 1.0]; vertex_count];
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    }

    /// Create an arrow mesh pointing upward (for player spawn marker)
    fn create_arrow_mesh(size: Vec3) -> Mesh {
        // Simple cylinder for the shaft with cone on top
        // We'll use a composite approach: cylinder base + cone tip
        // For simplicity in MVP, just use a cone pointing up
        let radius = size.x / 2.0;
        let height = size.y;

        // Create an upward-pointing cone
        Cone {
            radius,
            height,
        }.into()
    }
}
