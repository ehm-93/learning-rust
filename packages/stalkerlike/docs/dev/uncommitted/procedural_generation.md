# Procedural Generation System

## Overview
Hybrid approach combining hand-authored setpieces with procedural connective tissue. True 10km depth with seamless streaming.

## Architecture

### Chunk-Based World
```rust
// World divided into 32x32x32 meter chunks
#[derive(Hash, Eq, PartialEq, Clone, Copy)]
struct ChunkCoord {
    x: i32,  // Horizontal
    y: i32,  // Depth (negative = deeper)
    z: i32,  // Horizontal
}

// Only ~27 chunks loaded at once (3x3x3 around player)
const CHUNK_SIZE: f32 = 32.0;
const LOAD_RADIUS: i32 = 1;  // Chunks in each direction
```

### Generation Pipeline
1. Check if chunk is hand-authored (database)
2. Check if previously generated (cache)
3. Generate based on depth and constraints
4. Cache result in dynamic database

## Wave Function Collapse (WFC) for 3D

### Module Templates
```yaml
templates:
  tunnel_straight:
    id: tunnel_straight
    connections:
      north: tunnel_socket
      south: tunnel_socket
    weight: 10.0
    depth_range: [0, -10000]
    tags: [narrow, industrial]
    
  mining_chamber:
    id: mining_chamber
    connections:
      north: tunnel_socket
      south: tunnel_socket
      east: tunnel_socket
      west: tunnel_socket
      up: shaft_socket
      down: shaft_socket
    weight: 2.0
    depth_range: [-100, -5000]
    tags: [large, industrial, lit]
    
  collapsed_section:
    id: collapsed_section
    connections:
      north: tunnel_socket
      south: blocked
    weight: 5.0
    depth_range: [-500, -10000]
    tags: [hazard, impassable]
```

### Socket System
```rust
// Connections between modules
#[derive(Clone, PartialEq)]
enum SocketType {
    TunnelSmall,
    TunnelLarge,
    Shaft,
    Blocked,
    Open,
}

struct ModuleTemplate {
    id: String,
    sockets: HashMap<Direction, SocketType>,
    weight: f32,
    depth_range: (f32, f32),
    tags: Vec<String>,
}
```

### Constraint Propagation
```rust
fn pick_module_wfc(
    position: ChunkCoord,
    constraints: Vec<Constraint>,
    templates: &[ModuleTemplate],
    rng: &mut StdRng,
) -> ModuleTemplate {
    // Filter compatible modules
    let valid: Vec<_> = templates.iter()
        .filter(|t| {
            // Check depth
            let depth = position.y as f32 * CHUNK_SIZE;
            depth >= t.depth_range.0 && depth <= t.depth_range.1
        })
        .filter(|t| {
            // Check socket compatibility with neighbors
            constraints.iter().all(|c| {
                match c {
                    Constraint::Socket(dir, socket) => {
                        t.sockets.get(dir) == Some(socket)
                    },
                    Constraint::Tag(tag) => t.tags.contains(tag),
                    Constraint::NotTag(tag) => !t.tags.contains(tag),
                }
            })
        })
        .collect();
    
    // Weighted random selection
    valid.choose_weighted(rng, |t| t.weight)
        .expect("No valid modules")
        .clone()
}
```

## Depth-Based Generation

### Depth Zones
```rust
enum DepthZone {
    Surface,        // 0 to -100m
    Corporate,      // -100 to -500m
    Mining,         // -500 to -2000m
    Frontier,       // -2000 to -5000m
    Deep,           // -5000 to -8000m
    Abyss,         // -8000m and below
}

impl DepthZone {
    fn from_depth(y: f32) -> Self {
        match y {
            y if y > -100.0 => Self::Surface,
            y if y > -500.0 => Self::Corporate,
            y if y > -2000.0 => Self::Mining,
            y if y > -5000.0 => Self::Frontier,
            y if y > -8000.0 => Self::Deep,
            _ => Self::Abyss,
        }
    }
    
    fn get_templates(&self) -> Vec<&'static str> {
        match self {
            Self::Surface => vec!["entrance", "security", "transit"],
            Self::Corporate => vec!["office", "quarters", "cafeteria"],
            Self::Mining => vec!["shaft", "processing", "storage"],
            Self::Frontier => vec!["makeshift", "abandoned", "damaged"],
            Self::Deep => vec!["crystal", "unstable", "flooded"],
            Self::Abyss => vec!["void", "impossible", "dissolved"],
        }
    }
}
```

### Environmental Parameters
```rust
struct EnvironmentParams {
    depth: f32,
    temperature: f32,      // Increases with depth
    pressure: f32,         // Increases with depth
    radiation: f32,        // Varies by zone
    c7_concentration: f32, // Increases with depth
    structural_integrity: f32, // Decreases with depth
}

fn calculate_environment(coord: ChunkCoord) -> EnvironmentParams {
    let depth = coord.y as f32 * CHUNK_SIZE;
    
    EnvironmentParams {
        depth,
        temperature: 20.0 + (depth.abs() / 100.0) * 2.0,  // +2°C per 100m
        pressure: 1.0 + (depth.abs() / 1000.0),
        radiation: if depth < -5000.0 { 
            (depth.abs() - 5000.0) / 1000.0 
        } else { 0.0 },
        c7_concentration: (depth.abs() / 10000.0).min(1.0),
        structural_integrity: (1.0 - depth.abs() / 15000.0).max(0.1),
    }
}
```

## Content Population

### Prop Density
```rust
fn calculate_prop_density(zone: DepthZone, faction: Faction) -> f32 {
    match (zone, faction) {
        (DepthZone::Corporate, Faction::Corporate) => 0.8,
        (DepthZone::Mining, Faction::Union) => 0.6,
        (DepthZone::Frontier, Faction::Frontier) => 0.4,
        (DepthZone::Deep, _) => 0.2,
        (DepthZone::Abyss, _) => 0.1,
        _ => 0.3,
    }
}
```

### Prop Selection
```rust
fn select_props(
    zone: DepthZone,
    template: &ModuleTemplate,
    rng: &mut StdRng,
) -> Vec<PropPlacement> {
    let mut props = Vec::new();
    let density = calculate_prop_density(zone, template.faction);
    
    // Number of props based on density
    let num_props = (template.volume * density * 0.001) as usize;
    
    for _ in 0..num_props {
        let prop_type = match zone {
            DepthZone::Surface => {
                weighted_choice(&[
                    ("bench", 5.0),
                    ("plant", 3.0),
                    ("sign", 4.0),
                    ("terminal", 2.0),
                ], rng)
            },
            DepthZone::Mining => {
                weighted_choice(&[
                    ("drill", 3.0),
                    ("ore_cart", 4.0),
                    ("tool_rack", 2.0),
                    ("helmet", 5.0),
                ], rng)
            },
            DepthZone::Deep => {
                weighted_choice(&[
                    ("crystal_small", 5.0),
                    ("crystal_large", 1.0),
                    ("dissolved_equipment", 3.0),
                    ("anomaly", 0.5),
                ], rng)
            },
            // ... etc
        };
        
        props.push(PropPlacement {
            prop_type,
            position: random_valid_position(&template, rng),
            rotation: random_rotation(rng),
        });
    }
    
    props
}
```

## Faction Territory

### Territory System
```rust
struct TerritoryMap {
    control: HashMap<ChunkCoord, FactionControl>,
}

struct FactionControl {
    primary: Faction,
    influence: f32,  // 0.0 to 1.0
    contested: bool,
}

fn generate_faction_control(
    coord: ChunkCoord,
    seed: u64,
) -> FactionControl {
    let depth = coord.y as f32 * CHUNK_SIZE;
    
    // Base faction by depth
    let primary = match depth {
        d if d > -500.0 => Faction::Corporate,
        d if d > -2000.0 => Faction::Union,
        d if d > -5000.0 => Faction::Frontier,
        _ => Faction::Crystalline,
    };
    
    // Noise for organic borders
    let noise = simplex_noise_3d(
        coord.x as f64 * 0.1,
        coord.y as f64 * 0.1,
        coord.z as f64 * 0.1,
        seed,
    );
    
    FactionControl {
        primary,
        influence: (0.5 + noise * 0.5).clamp(0.0, 1.0),
        contested: noise.abs() < 0.2,  // Border zones
    }
}
```

## Special Generation Cases

### Connection Points
```yaml
# Hand-authored chunks specify connection points
chunk:
  id: corporate_hub
  connections:
    - position: [32, 0, 16]
      direction: east
      socket: tunnel_large
      tags: [main_path]
    - position: [16, -10, 0]
      direction: down
      socket: shaft
      tags: [maintenance]
```

### Path Guarantees
```rust
// Ensure critical paths remain connected
fn validate_critical_path(
    start: ChunkCoord,
    end: ChunkCoord,
    chunks: &HashMap<ChunkCoord, ChunkData>,
) -> bool {
    // A* pathfinding through chunks
    pathfind(start, end, chunks).is_some()
}

// Regenerate if critical path broken
fn ensure_connectivity(world: &mut World) {
    let critical_paths = vec![
        (SURFACE_ENTRANCE, CORPORATE_HUB),
        (CORPORATE_HUB, DEEP_MINES),
        // ...
    ];
    
    for (start, end) in critical_paths {
        if !validate_critical_path(start, end, &world.chunks) {
            regenerate_path(start, end, world);
        }
    }
}
```

## Optimization

### Chunk Caching
```sql
-- Store generated chunks in database
CREATE TABLE generated_chunks (
    x INTEGER, y INTEGER, z INTEGER,
    seed INTEGER,
    template_id TEXT,
    props TEXT,  -- JSON array
    generated_at TIMESTAMP,
    PRIMARY KEY (x, y, z)
);
```

### LOD Generation
```rust
enum ChunkLOD {
    Full,     // Everything
    Reduced,  // Geometry only, no props
    Distant,  // Simplified geometry
    Unloaded, // Nothing
}

fn determine_lod(player_pos: Vec3, chunk_coord: ChunkCoord) -> ChunkLOD {
    let chunk_center = chunk_coord.to_world_pos();
    let distance = (player_pos - chunk_center).length();
    
    match distance {
        d if d < CHUNK_SIZE => ChunkLOD::Full,
        d if d < CHUNK_SIZE * 3.0 => ChunkLOD::Reduced,
        d if d < CHUNK_SIZE * 6.0 => ChunkLOD::Distant,
        _ => ChunkLOD::Unloaded,
    }
}
```

## Streaming System

### Chunk Loading
```rust
fn stream_world_system(
    player: Query<&Transform, With<Player>>,
    mut chunk_manager: ResMut<ChunkManager>,
    mut commands: Commands,
) {
    let player_pos = player.single().translation;
    let player_chunk = ChunkCoord::from_world_pos(player_pos);
    
    // Calculate chunks to load
    let mut should_load = HashSet::new();
    for dx in -LOAD_RADIUS..=LOAD_RADIUS {
        for dy in -LOAD_RADIUS..=LOAD_RADIUS {
            for dz in -LOAD_RADIUS..=LOAD_RADIUS {
                let coord = ChunkCoord {
                    x: player_chunk.x + dx,
                    y: player_chunk.y + dy,
                    z: player_chunk.z + dz,
                };
                should_load.insert(coord);
            }
        }
    }
    
    // Load new chunks
    for coord in &should_load {
        if !chunk_manager.loaded.contains_key(coord) {
            let chunk_data = generate_or_load_chunk(*coord);
            spawn_chunk(chunk_data, &mut commands);
            chunk_manager.loaded.insert(*coord, chunk_data);
        }
    }
    
    // Unload distant chunks
    chunk_manager.loaded.retain(|coord, _| {
        should_load.contains(coord)
    });
}
```

## Seed Management

```rust
struct WorldSeed {
    master: u64,
    
    // Derived seeds for different systems
    layout: u64,      // Chunk templates
    props: u64,       // Prop placement
    faction: u64,     // Territory control
    narrative: u64,   // Story elements
}

impl WorldSeed {
    fn new(master: u64) -> Self {
        Self {
            master,
            layout: hash(master, "layout"),
            props: hash(master, "props"),
            faction: hash(master, "faction"),
            narrative: hash(master, "narrative"),
        }
    }
    
    fn for_chunk(&self, coord: ChunkCoord) -> u64 {
        let mut hasher = DefaultHasher::new();
        hasher.write_u64(self.master);
        coord.hash(&mut hasher);
        hasher.finish()
    }
}
```

## Integration with Hand-Authored Content

### Mixing Strategies
1. **Islands**: Hand-authored chunks surrounded by procedural
2. **Highways**: Procedural connects hand-authored setpieces
3. **Layers**: Hand-authored main path, procedural side areas
4. **Templates**: Hand-authored used as templates for procedural

### Priority System
```rust
fn get_chunk_source(coord: ChunkCoord) -> ChunkSource {
    // Check priority
    if is_critical_story_chunk(coord) {
        ChunkSource::Handcrafted
    } else if is_near_critical_path(coord, 2) {
        ChunkSource::SemiProcedural  // Use templates
    } else {
        ChunkSource::FullyProcedural
    }
}
```
