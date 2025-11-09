# World Architecture & Chunking

## Overview
True-scale 10km deep mining colony with seamless streaming. Primarily hand-authored content with copy-paste variations for efficiency. Procedural generation deferred until hand-crafting becomes prohibitive.

## World Structure

### Coordinate System
**World Space**: Standard 3D coordinates where Y-axis represents depth
- X: East-West positioning
- Y: Depth (0 = surface, -10000 = deepest point)
- Z: North-South positioning

**Chunk Coordinates**: Integer indices for spatial organization
- Each chunk is 32×32×32 meters
- Chunk indices derived from world position ÷ 32
- Simplifies loading/unloading regions

### Scale Reference
- **Surface to Corporate Hub**: 100m (3 chunks)
- **Corporate to Active Mining**: 500m (15 chunks)
- **Mining to Frontier**: 2000m (62 chunks)
- **Frontier to Deep**: 5000m (156 chunks)
- **Deep to Abyss**: 10000m (312 chunks)
- **Total vertical chunks**: ~500 chunks possible

## Streaming Architecture

### Memory Management
**Chunk Manager Responsibilities:**
- Track currently loaded chunks in memory
- Maintain load/unload queues
- Monitor player position
- Enforce memory budgets
- Handle level-of-detail transitions

**Per-Chunk Data:**
- Coordinate and spatial info
- Entity references (for ECS)
- Current LOD level
- Last access time (for LRU unloading)
- Memory footprint estimate

### Loading Strategy
**Concentric zones around player:**
- **Distance 1 (Adjacent)**: Full detail - everything loads
- **Distance 2 (Nearby)**: Reduced detail - geometry + lights only
- **Distance 3 (Distant)**: Minimal - simplified geometry
- **Distance 4+**: Unloaded - not in memory

**Algorithm:**
- Calculate player's current chunk
- Generate list of chunks within each distance ring
- Load new chunks by priority
- Unload chunks beyond maximum distance
- Transition LOD for chunks changing zones

## Coordinate Management

### Precision Considerations
At 10km depth, 32-bit floating-point precision is still adequate (sub-centimeter accuracy). Floating origin systems are typically unnecessary until distances exceed 50-100km.

**Simple Approach:**
- Use standard world coordinates with Y=0 at surface, Y=-10000 at deepest point
- 32-bit floats maintain millimeter precision at this scale
- No special coordinate handling required

**If Future Expansion Needed:**
If the world grows beyond 50km in any dimension, consider:
- Floating origin system (shift world periodically)
- 64-bit coordinates for true position
- Chunked coordinate system (sector + local offset)

**Current Scale:**
- 10km depth fits comfortably in standard floating-point range
- Precision remains adequate for gameplay and physics
- Simpler implementation without origin shifting

## Chunk Types

### Classification
**Narrative** (from `levels/`):  
Hand-authored critical path and major locations

**Filler** (from `levels/`):  
Copy-paste variations of common sections
- Start with template (e.g., `tunnel_straight_01`)
- Duplicate and modify (lighting, props, damage)
- Save as new level (`tunnel_straight_02`, etc.)
- Much faster than procedural tuning

**Void**:  
Empty space optimization (nothing to load)

### Content Strategy
**Phase 1 (MVP)**: 
- Hand-author all critical path content
- Create 5-10 tunnel templates
- Copy-paste and modify for variety
- Focus on quality over quantity

**Phase 2 (If Needed)**:
- If hand-authoring becomes bottleneck
- Consider simple procedural for filler only
- Keep narrative content always hand-crafted

### Level Templates (Hand-Authored)
```yaml
# levels/corporate_hub/level.yaml
level:
  id: corporate_hub
  type: narrative
  size: [64, 32, 64]  # Can span multiple chunks
  connections:
    - pos: [32, 0, 0]
      dir: north
      socket: tunnel_large
      tags: [main_path]
    - pos: [0, -16, 32]
      dir: down
      socket: shaft
      tags: [maintenance]
  always_loaded: true
  
# Procedural system references prefabs/
procedural_config:
  tunnel_prefabs: [tunnel_straight, tunnel_corner, tunnel_T]
  variation: [intact, damaged, flooded]
  prop_density: 0.4
  lighting: sparse
```

## Performance Optimization

### Occlusion Culling
**Portal-based system for tunnels:**
- Define portals at tunnel connections
- Track which chunks are visible through portals
- Only render chunks with visible path from player
- Effective in underground environment (natural occlusion)

**Portal Structure:**
- Position and normal vector
- References to connected chunks
- Can be closed/opened by gameplay (doors, cave-ins)

### Batch Rendering
**Instancing for repeated elements:**
- Group identical meshes together
- Single draw call for many instances
- Per-instance transforms and color variations
- Crucial for props (lights, pipes, debris)

**Batching Strategy:**
- Organize by mesh handle
- Collect instance data (transform, color)
- Submit as instanced draw call
- Can handle hundreds of props efficiently

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
**Chunk Loading Priority:**
1. Check if chunk contains hand-authored level (query database by coordinates)
2. If yes: Load from `levels/` directory YAML
3. If no: Empty/void space
4. Apply any runtime modifications from dynamic database (doors opened, items taken, etc.)
5. Spawn entities into ECS

**No Procedural Generation Initially:**
- All content explicitly authored or copy-pasted
- Simpler, more predictable, faster to iterate
- Add procedural generation later only if genuinely needed
- Most games overestimate need for procedural content

**Modification Tracking:**
- Store player-caused changes in dynamic database
- Keyed by chunk coordinates
- JSON blobs for flexibility
- Applied on top of base chunk data
- Preserves player agency in world

## Network Considerations (Future)

### Chunk Ownership
**For potential multiplayer:**
- Track which player "owns" each chunk (server authority)
- Version number for detecting conflicts
- Dirty flag for modified chunks needing sync

**Deterministic Generation:**
- Same seed = same procedural content
- Only modifications need network sync
- Reduces bandwidth dramatically
- Chunk deltas instead of full chunks

## Debug Visualization

### Chunk Boundaries
**Visual debugging tools:**
- Toggle display of chunk boundaries (wireframe boxes)
- Color-code by LOD level (green=full, yellow=reduced, red=minimal)
- Show chunk coordinates as text labels
- Display memory usage per chunk

### Performance Metrics
**Real-time monitoring:**
- Chunks loaded vs. total in memory
- Total memory consumption (MB)
- Load/generation time (ms)
- Frame time impact
- Cache hit/miss rates

**Display Options:**
- Overlay HUD during development
- Export to log file for analysis
- Integration with profiling tools

## Seamless Transitions

### Chunk Loading Priority
**Priority Levels:**
- **Critical**: Player's current chunk (must be loaded)
- **High**: Adjacent chunks + predicted next chunk
- **Medium**: Visible chunks in range
- **Low**: Predictive loading based on movement
- **Background**: Preloading for known paths

**Predictive Loading:**
- Analyze player velocity vector
- Predict next chunk based on direction
- Pre-load along movement path
- Reduces pop-in during fast movement

### Smooth LOD Transitions
**Fade System:**
- Gradually transition between LOD levels
- Fade in props when upgrading LOD
- Fade in detail geometry
- Prevents jarring visual changes
- Configurable transition speed (2-3 seconds typical)

**State Machine:**
- Track current and target LOD per chunk
- Interpolate opacity/visibility
- Handle bidirectional transitions (upgrading and downgrading)

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
