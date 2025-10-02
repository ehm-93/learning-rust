use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use std::collections::HashMap;
use image::{ImageBuffer, Rgb};

pub const CHUNK_SIZE: usize = 256;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PathType {
    Main,
    Side,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EdgeSide {
    North,
    South,
    East,
    West,
}

impl EdgeSide {
    pub fn opposite(&self) -> EdgeSide {
        match self {
            EdgeSide::North => EdgeSide::South,
            EdgeSide::South => EdgeSide::North,
            EdgeSide::East => EdgeSide::West,
            EdgeSide::West => EdgeSide::East,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Connector {
    pub edge: EdgeSide,
    pub position: usize,
    pub path_type: PathType,
}

impl Connector {
    pub fn matches(&self, other: &Connector) -> bool {
        self.edge.opposite() == other.edge
            && self.position == other.position
            && self.path_type == other.path_type
    }
}

#[derive(Debug, Clone)]
pub struct Template {
    pub id: usize,
    pub tiles: Vec<Vec<bool>>, // true = walkable, false = solid
    pub connectors: Vec<Connector>,
}

impl Template {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            tiles: vec![vec![false; CHUNK_SIZE]; CHUNK_SIZE],
            connectors: Vec::new(),
        }
    }
}

pub struct TilesetGenerator {
    templates: Vec<Template>,
}

impl TilesetGenerator {
    pub fn new() -> Self {
        Self {
            templates: Vec::new(),
        }
    }

    pub fn generate_chunk(&self, x: i32, y: i32) -> Template {
        self.generate_chunk_with_constraints(x, y, &HashMap::new())
    }

    pub fn generate_chunk_with_constraints(&self, x: i32, y: i32, neighbors: &HashMap<EdgeSide, Vec<Connector>>) -> Template {
        // Use coordinates as seed for deterministic generation
        let seed = ((x as u64) << 32) | (y as u64);
        let mut rng = StdRng::seed_from_u64(seed);

        // Filter templates that satisfy neighbor constraints
        let valid_templates: Vec<&Template> = self.templates.iter()
            .filter(|template| self.template_satisfies_constraints(template, neighbors))
            .collect();

        if valid_templates.is_empty() {
            // If no template satisfies constraints, generate a simple one
            self.generate_simple_template(0, &mut rng)
        } else {
            let idx = rng.gen_range(0..valid_templates.len());
            valid_templates[idx].clone()
        }
    }

    fn template_satisfies_constraints(&self, template: &Template, neighbors: &HashMap<EdgeSide, Vec<Connector>>) -> bool {
        for (neighbor_edge, neighbor_connectors) in neighbors {
            let our_connectors: Vec<&Connector> = template.connectors.iter()
                .filter(|c| c.edge == *neighbor_edge)
                .collect();

            // Check if all neighbor connectors have matching connectors on our side
            for neighbor_connector in neighbor_connectors {
                let has_match = our_connectors.iter().any(|our_connector| {
                    our_connector.position == neighbor_connector.position &&
                    our_connector.path_type == neighbor_connector.path_type
                });

                if !has_match {
                    return false;
                }
            }
        }
        true
    }

    fn generate_simple_template(&self, id: usize, _rng: &mut StdRng) -> Template {
        let mut template = Template::new(id);

        // Create a simple room with some corridors
        // Fill center area as walkable
        for y in 64..192 {
            for x in 64..192 {
                template.tiles[y][x] = true;
            }
        }

        // Define 1 entrypoint per side at center position (128)
        let entrypoint_position = 128;
        let corridor_width = 4;

        // Extract configuration from template ID
        // Each configuration represents which sides have entrypoints (4 bits)
        let config = id % 16; // 16 possible configurations (2^4)
        let has_north = (config & 0b1000) != 0;
        let has_south = (config & 0b0100) != 0;
        let has_east = (config & 0b0010) != 0;
        let has_west = (config & 0b0001) != 0;

        // North corridor
        if has_north {
            for y in 0..64 {
                for x in (entrypoint_position - corridor_width/2)..=(entrypoint_position + corridor_width/2) {
                    if x < CHUNK_SIZE {
                        template.tiles[y][x] = true;
                    }
                }
            }
            template.connectors.push(Connector {
                edge: EdgeSide::North,
                position: entrypoint_position,
                path_type: PathType::Main,
            });
        }

        // South corridor
        if has_south {
            for y in 192..CHUNK_SIZE {
                for x in (entrypoint_position - corridor_width/2)..=(entrypoint_position + corridor_width/2) {
                    if x < CHUNK_SIZE {
                        template.tiles[y][x] = true;
                    }
                }
            }
            template.connectors.push(Connector {
                edge: EdgeSide::South,
                position: entrypoint_position,
                path_type: PathType::Main,
            });
        }

        // East corridor
        if has_east {
            for y in (entrypoint_position - corridor_width/2)..=(entrypoint_position + corridor_width/2) {
                if y < CHUNK_SIZE {
                    for x in 192..CHUNK_SIZE {
                        template.tiles[y][x] = true;
                    }
                }
            }
            template.connectors.push(Connector {
                edge: EdgeSide::East,
                position: entrypoint_position,
                path_type: PathType::Main,
            });
        }

        // West corridor
        if has_west {
            for y in (entrypoint_position - corridor_width/2)..=(entrypoint_position + corridor_width/2) {
                if y < CHUNK_SIZE {
                    for x in 0..64 {
                        template.tiles[y][x] = true;
                    }
                }
            }
            template.connectors.push(Connector {
                edge: EdgeSide::West,
                position: entrypoint_position,
                path_type: PathType::Main,
            });
        }

        template
    }

    pub fn generate_template_library(&mut self, variations_per_config: usize) {
        let mut rng = StdRng::seed_from_u64(42); // Fixed seed for reproducible templates

        // Generate variations_per_config templates for each of the 16 possible configurations
        for config in 0..16 {
            for variation in 0..variations_per_config {
                let template_id = config * variations_per_config + variation;
                let template = self.generate_simple_template(template_id, &mut rng);
                self.templates.push(template);
            }
        }

        println!("Generated {} templates: {} variations of {} configurations",
                 self.templates.len(), variations_per_config, 16);
    }

    pub fn get_neighbor_connectors(&self, chunk: &Template, edge: EdgeSide) -> Vec<Connector> {
        chunk.connectors.iter()
            .filter(|c| c.edge == edge)
            .map(|c| Connector {
                edge: c.edge.opposite(),
                position: c.position,
                path_type: c.path_type,
            })
            .collect()
    }

    pub fn export_chunk_as_bitmap(&self, chunk: &Template, x: i32, y: i32, output_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut img = ImageBuffer::new(CHUNK_SIZE as u32, CHUNK_SIZE as u32);

        for (y_pos, row) in chunk.tiles.iter().enumerate() {
            for (x_pos, &is_walkable) in row.iter().enumerate() {
                let color = if is_walkable {
                    Rgb([255u8, 255u8, 255u8]) // White for walkable
                } else {
                    Rgb([0u8, 0u8, 0u8]) // Black for solid
                };
                img.put_pixel(x_pos as u32, y_pos as u32, color);
            }
        }

        // Draw connectors as colored pixels (4px wide)
        for connector in &chunk.connectors {
            let color = match connector.path_type {
                PathType::Main => Rgb([255u8, 0u8, 0u8]), // Red for main paths
                PathType::Side => Rgb([0u8, 255u8, 0u8]), // Green for side paths
            };

            let corridor_width = 4;
            let start_pos = connector.position - corridor_width/2;
            let end_pos = connector.position + corridor_width/2;

            match connector.edge {
                EdgeSide::North => {
                    for x in start_pos..=end_pos {
                        if x < CHUNK_SIZE {
                            img.put_pixel(x as u32, 0, color);
                        }
                    }
                },
                EdgeSide::South => {
                    for x in start_pos..=end_pos {
                        if x < CHUNK_SIZE {
                            img.put_pixel(x as u32, (CHUNK_SIZE - 1) as u32, color);
                        }
                    }
                },
                EdgeSide::East => {
                    for y in start_pos..=end_pos {
                        if y < CHUNK_SIZE {
                            img.put_pixel((CHUNK_SIZE - 1) as u32, y as u32, color);
                        }
                    }
                },
                EdgeSide::West => {
                    for y in start_pos..=end_pos {
                        if y < CHUNK_SIZE {
                            img.put_pixel(0, y as u32, color);
                        }
                    }
                },
            }
        }

        let filename = format!("{}/chunk_{}_{}.bmp", output_dir, x, y);
        img.save(&filename)?;
        println!("Exported chunk ({}, {}) to {}", x, y, filename);
        Ok(())
    }
}
