use bevy::prelude::*;

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
}

impl PrimitiveType {
    pub fn create_mesh(&self, size: Vec3) -> Mesh {
        match self {
            PrimitiveType::Cube => Cuboid::new(size.x, size.y, size.z).into(),
        }
    }
}
