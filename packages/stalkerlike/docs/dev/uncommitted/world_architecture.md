# World Architecture & Chunking

## Overview
True-scale 10km deep mining colony with seamless streaming, combining hand-authored narrative spaces with procedural connective tissue.

## World Structure

### Coordinate System
```rust
// World space uses standard 3D coordinates
// Y-axis represents depth (negative = deeper)
struct WorldPosition {
    x: f32,  // East-West
    y: f32,  // Depth (0 = surface, -10000 = deepest)
    z: f32,  // North-South
}

// Chunk coordinates for spatial indexing
struct ChunkCoord {
    x: i32,  // Chunk index X
    y: i32,  // Chunk index Y (depth)
    z: i32,  // Chunk index Z
}

const CHUNK_SIZE: f32 = 32.0;  // 32x32x32 meter chunks
```

### Scale Reference
- **Surface to Corporate Hub**: 100m (3 chunks)
- **Corporate to Active Mining**: 500m (15 chunks)
- **Mining to Frontier**: 2000m (62 chunks)
- **Frontier to Deep**: 5000m (156 chunks)
- **Deep to Abyss**: 10000m (312 chunks)
- **Total vertical chunks**: ~500 chunks possible

## Streaming Architecture

### Memory Management
```rust
struct ChunkManager {
    // Currently loaded chunks
    loaded: HashMap<ChunkCoord, LoadedChunk>,
    
    // Loading queue
    load_queue: VecDeque<ChunkCoord>,
    
    // Unload queue
    unload_queue: VecDeque<ChunkCoord>,
    
    // Player position for reference
    player_chunk: ChunkCoord,
}

struct LoadedChunk {
    coord: ChunkCoord,
    entities: Vec<Entity>,
    lod_level: LODLevel,
    last_accessed: Instant,
    memory_size: usize,
}
```

### Loading Strategy
```rust
// Concentric loading zones around player
const LOAD_DISTANCES: [(i32, LODLevel); 4] = [
    (1, LODLevel::Full),      // Adjacent chunks: everything
    (2, LODLevel::Reduced),   // Nearby: geometry + lights
    (3, LODLevel::Minimal),   // Distant: geometry only
    (4, LODLevel::Unloaded),  // Beyond: nothing
];

fn calculate_chunks_to_load(player_chunk: ChunkCoord) -> Vec<(ChunkCoord, LODLevel)> {
    let mut chunks = Vec::new();
    
    for (radius, lod) in LOAD_DISTANCES.iter() {
        for dx in -radius..=radius {
            for dy in -radius..=radius {
                for dz in -radius..=radius {
                    // Skip if in inner radius
                    let dist = dx.abs().max(dy.abs()).max(dz.abs());
                    if dist != *radius { continue; }
                    
                    let coord = ChunkCoord {
                        x: player_chunk.x + dx,
                        y: player_chunk.y + dy,
                        z: player_chunk.z + dz,
                    };
                    
                    chunks.push((coord, *lod));
                }
            }
        }
    }
    
    chunks
}
```

## Floating Origin

### Precision Management
```rust
// Prevent floating point errors at extreme distances
struct FloatingOrigin {
    offset: DVec3,  // 64-bit precision offset
    threshold: f32, // When to recenter (e.g., 1000m)
}

fn floating_origin_system(
    mut origin: ResMut<FloatingOrigin>,
    mut player: Query<&mut Transform, With<Player>>,
    mut everything: Query<&mut Transform, Without<Player>>,
) {
    let player_pos = player.single().translation;
    
    if player_pos.length() > origin.threshold {
        // Record offset
        origin.offset += player_pos.as_dvec3();
        
        // Shift world back to origin
        for mut transform in everything.iter_mut() {
            transform.translation -= player_pos;
        }
        
        // Reset player to origin
        player.single_mut().translation = Vec3::ZERO;
    }
}
```

## Chunk Types

### Classification
```rust
enum ChunkType {
    // Hand-authored critical path
    Narrative {
        id: String,
        script: Option<String>,
    },
    
    // Procedural with constraints
    Procedural {
        template: String,
        seed: u64,
    },
    
    // Mix of both
    Hybrid {
        base: String,
        modifications: Vec<Modification>,
    },
    
    // Empty space (optimization)
    Void,
}
```

### Chunk Templates
```yaml
templates:
  corporate_hub:
    type: narrative
    size: [64, 32, 64]  # Can span multiple chunks
    connections:
      - {pos: [32, 0, 0], dir: north, type: transit}
      - {pos: [0, -16, 32], dir: down, type: shaft}
    always_loaded: true
    
  mining_shaft:
    type: procedural
    size: [32, 32, 32]
    variations: [intact, damaged, flooded]
    prop_density: 0.4
    lighting: sparse
    
  connector_tunnel:
    type: hybrid
    size: [32, 32, 32]
    base: tunnel_straight
    can_modify: [props, lighting]
```

## Performance Optimization

### Occlusion Culling
```rust
// Portal-based occlusion for tunnels
struct Portal {
    position: Vec3,
    normal: Vec3,
    connected_chunks: [ChunkCoord; 2],
}

fn is_chunk_visible(
    chunk: ChunkCoord,
    player_pos: Vec3,
    portals: &[Portal],
) -> bool {
    // Check if any portal to this chunk is visible
    for portal in portals.iter() {
        if !portal.connected_chunks.contains(&chunk) {
            continue;
        }
        
        let to_portal = portal.position - player_pos;
        let facing = to_portal.normalize().dot(portal.normal);
        
        if facing > 0.0 {
            return true;  // Portal is visible
        }
    }
    
    false
}
```

### Batch Rendering
```rust
// Instance rendering for repeated elements
struct InstanceData {
    transform: Mat4,
    color_variation: Vec4,
}

fn batch_props(props: &[PropInstance]) -> HashMap<MeshHandle, Vec<InstanceData>> {
    let mut batches = HashMap::new();
    
    for prop in props {
        batches.entry(prop.mesh.clone())
            .or_insert_with(Vec::new)
            .push(InstanceData {
                transform: prop.transform.compute_matrix(),
                color_variation: prop.color,
            });
    }
    
    batches
}
```

## Save System Integration

### Chunk State Persistence
```sql
-- Tracks modifications to chunks
CREATE TABLE chunk_modifications (
    chunk_x INTEGER,
    chunk_y INTEGER,
    chunk_z INTEGER,
    modification_type TEXT,
    data TEXT,  -- JSON blob
    timestamp REAL,
    PRIMARY KEY (chunk_x, chunk_y, chunk_z, modification_type)
);

-- Example: Door opened
INSERT INTO chunk_modifications VALUES (
    0, -3, 0, 
    'door_state',
    '{"door_id": "security_door_01", "open": true}',
    12345.67
);
```

### Loading Modified Chunks
```rust
fn load_chunk_with_modifications(
    coord: ChunkCoord,
    db: &Connection,
) -> ChunkData {
    // Load base chunk
    let mut chunk = load_base_chunk(coord);
    
    // Apply modifications
    let mods: Vec<Modification> = db.prepare(
        "SELECT modification_type, data 
         FROM chunk_modifications 
         WHERE chunk_x = ? AND chunk_y = ? AND chunk_z = ?"
    )?.query_map([coord.x, coord.y, coord.z], |row| {
        Ok(Modification {
            mod_type: row.get(0)?,
            data: serde_json::from_str(&row.get::<_, String>(1)?).ok()?,
        })
    })?.collect();
    
    for modification in mods {
        chunk.apply_modification(modification);
    }
    
    chunk
}
```

## Network Considerations (Future)

### Chunk Ownership
```rust
// For potential multiplayer
struct ChunkOwnership {
    chunk: ChunkCoord,
    owner: Option<PlayerId>,
    version: u64,
    dirty: bool,
}

// Deterministic generation means only changes need syncing
struct ChunkDelta {
    chunk: ChunkCoord,
    base_version: u64,
    modifications: Vec<Modification>,
}
```

## Debug Visualization

### Chunk Boundaries
```rust
fn debug_draw_chunks(
    mut gizmos: Gizmos,
    chunks: Query<&LoadedChunk>,
    debug: Res<DebugSettings>,
) {
    if !debug.show_chunk_bounds { return; }
    
    for chunk in chunks.iter() {
        let world_pos = chunk.coord.to_world_pos();
        let color = match chunk.lod_level {
            LODLevel::Full => Color::GREEN,
            LODLevel::Reduced => Color::YELLOW,
            LODLevel::Minimal => Color::RED,
            _ => Color::GRAY,
        };
        
        gizmos.cuboid(
            Transform::from_translation(world_pos)
                .with_scale(Vec3::splat(CHUNK_SIZE)),
            color,
        );
    }
}
```

### Performance Metrics
```rust
struct ChunkingMetrics {
    chunks_loaded: usize,
    chunks_in_memory: usize,
    total_memory_mb: f32,
    load_time_ms: f32,
    generation_time_ms: f32,
}

fn display_metrics(metrics: Res<ChunkingMetrics>) {
    debug_ui.text(format!(
        "Chunks: {}/{} | Memory: {:.1}MB | Load: {:.1}ms",
        metrics.chunks_loaded,
        metrics.chunks_in_memory,
        metrics.total_memory_mb,
        metrics.load_time_ms,
    ));
}
```

## Seamless Transitions

### Chunk Loading Priority
```rust
enum LoadPriority {
    Critical = 0,     // Player's current chunk
    High = 1,         // Adjacent chunks
    Medium = 2,       // Visible chunks
    Low = 3,          // Predictive loading
    Background = 4,   // Preloading
}

fn prioritize_loading(
    player_velocity: Vec3,
    player_chunk: ChunkCoord,
    chunk_to_load: ChunkCoord,
) -> LoadPriority {
    let distance = chunk_distance(player_chunk, chunk_to_load);
    
    // Predictive loading based on movement
    let predicted_chunk = predict_next_chunk(player_chunk, player_velocity);
    
    match distance {
        0 => LoadPriority::Critical,
        1 => LoadPriority::High,
        2 if chunk_to_load == predicted_chunk => LoadPriority::High,
        2 => LoadPriority::Medium,
        3 if chunk_to_load == predicted_chunk => LoadPriority::Medium,
        _ => LoadPriority::Low,
    }
}
```

### Smooth LOD Transitions
```rust
fn smooth_lod_transition(
    chunk: &mut LoadedChunk,
    target_lod: LODLevel,
    delta_time: f32,
) {
    const TRANSITION_SPEED: f32 = 2.0;
    
    match (chunk.lod_level, target_lod) {
        (LODLevel::Minimal, LODLevel::Reduced) => {
            // Fade in props
            for prop in &mut chunk.props {
                prop.opacity = (prop.opacity + delta_time * TRANSITION_SPEED).min(1.0);
            }
        },
        (LODLevel::Reduced, LODLevel::Full) => {
            // Fade in details
            for detail in &mut chunk.details {
                detail.opacity = (detail.opacity + delta_time * TRANSITION_SPEED).min(1.0);
            }
        },
        _ => {},
    }
}
```

## Best Practices

### Do's
- Keep chunks power-of-2 sizes (32m, 64m)
- Preload chunks along predicted path
- Cache generated chunks aggressively
- Use LOD for distant chunks
- Stream assets asynchronously

### Don'ts
- Don't load all chunks at once
- Don't generate on main thread
- Don't save unmodified chunks
- Don't use unbounded view distances
- Don't ignore memory limits

### Memory Budget
```yaml
Target Memory Usage:
  Full chunks: 10MB each × 27 = 270MB
  Reduced chunks: 2MB each × 98 = 196MB  
  Minimal chunks: 0.5MB each × 218 = 109MB
  Total chunk memory: ~575MB
  
  Asset cache: 200MB
  Audio: 100MB
  UI/Other: 125MB
  
  Total target: ~1GB
```

This architecture supports the full 10km depth while maintaining performance through aggressive streaming and LOD management.
