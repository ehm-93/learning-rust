# Phase 1.5 - Terrain Foundation

## Goal
Build a continuous destructible world with chunked terrain. Player explores vast caverns where walls explode and the map sprawls organically.

## Success
- Walk 10km+ smoothly
- Walls destroyed by weapons
- Invisible chunk loading
- Organic caves, not geometric rooms
- Ready for stairs to next level

## Core Systems

### Chunks
- **Size**: 64×64 tiles (32m × 32m, 0.5m/tile)
- **Level**: 2048m × 2048m (32×32 chunks = 1024 total)
- **Active**: Load 5×5 grid around player (160m radius)
- **Unload**: Remove chunks 7+ away
- **Persist**: Modified chunks → SQLite
- **Seed**: Same seed + level = same terrain

### Generation
**Macro** (pre-generated per level):
- Random walks from center to edges ensure connectivity
- Angular distribution prevents overlap
- Multi-scale simplex noise adds variation
- Cellular automata smooths single-tile walls
- Flood fill validates full traversability
- Result: 2×2 pixels/chunk density map (64×64 macro grid)

**Micro** (per-chunk on-demand):
- Sample 2×2 macro values for chunk quadrants
- Use densities as targets for tile placement
- Blend between quadrants to prevent seams
- Apply micro-variation noise

**Variation**: Adjust paths, noise, smoothing per level depth

**Stairs**: Flood fill from spawn, place in outer reachable ring (forces exploration)

### Destruction
- **HP**: 50-200 per wall tile (scales with level)
- **Sources**: Bullets, explosions, abilities
- **Feedback**: Particles, debris, shake
- **Permanent**: Walls stay destroyed
- **Physics**: Collision removed immediately

### Collision
- **Grid**: 0.5m tiles
- **Queries**: Spatial hashing for nearby tiles only
- **Projectiles**: Raycasting vs tile grid
- **Player**: Physics body vs solid tiles

### Rendering
- **Batching**: Entire chunk = single mesh
- **Atlas**: All tiles in one texture
- **Culling**: Only visible chunks
- **Dirty tracking**: Rebuild mesh on change
- **View**: Top-down, 5×5 chunk area (~160m radius)
- **Boundaries**: Hard walls at 2048m edges

---

## Implementation Phases

### A: Static Tilemap (2-3 days)
Render tiles, prove systems work.

1. Add `bevy_ecs_tilemap = "0.14"` to Cargo.toml
2. Add TilemapPlugin to App
3. Create 16×16px sprites: wall.png (gray), floor.png (dark)
4. Load as texture atlas
5. Define hardcoded 32×32 bool array (wall/floor)
6. Spawn tilemap from array
7. Add collision to walls
8. Camera smooth-follow (lerp to player)
9. Movement checks tile collision

**Test**: Walk around, can't pass walls, camera follows

---

### B: Chunk Infrastructure (3-4 days)
Spatial management without generation.

1. Chunk struct (position, tiles, dirty flag)
2. ChunkManager resource with HashMap<IVec2, Chunk>
3. System calculates 5×5 grid around player
4. Spawn tilemaps for needed chunks
5. Despawn chunks beyond distance 7
6. Move hardcoded pattern to per-chunk generation
7. Debug gizmos for chunk boundaries

**Test**: Walk 500m, chunk count stays stable

---

### C: Macro Generation (2-3 days)
Prove algorithm works independently.

1. Port freeform() function
2. MacroMap struct (64×64 bool array)
3. Generate macro once per level
4. Store in Level resource
5. Save PNG visualization (debug)
6. Test multiple seeds
7. Run flood fill, verify traversability

**Test**: 10 seeds, all show connected caves in PNG

---

### D: Micro Generation (4-5 days)
Generate chunk detail from macro.

1. `sample_macro(chunk_pos)` → 2×2 density array
2. `generate_chunk_tiles(densities)` → 64×64 tiles
   - Iterate tile positions
   - Find nearest quadrant
   - Weighted average of nearby densities
   - Random threshold vs density → wall/floor
3. Replace hardcoded with micro generation
4. Spawn player at center (16, 16)
5. Test full 2km × 2km traversal

**Test**: Walk to edges (1km from center), smooth variation, no seams

---

### E: Destructible Terrain (2-3 days)
Walls break.

1. Health component on walls (value by level)
2. Projectile collision detects tile hits
3. Damage system reduces health
4. Remove tiles at 0 HP
5. Update chunk collision immediately
6. Set chunk modified flag
7. Particles and shake on destruction

**Test**: Shoot walls, they break, collision updates

---

### F: Chunk Persistence (3-4 days)
Modified chunks survive sessions.

1. Add rusqlite dependency
2. Create database file
3. Schema: `chunks (level INT, x INT, y INT, tiles BLOB, timestamp INT)`
4. Serialize tiles to binary
5. Save on chunk modify
6. Load from DB before generating
7. Test persistence across restarts

**Test**: Destroy walls, restart multiple times, pattern persists

---

### G: Level Progression (2-3 days)
Stairs allow descent.

1. Flood fill from spawn
2. Filter tiles 500m+ from spawn (outer ring)
3. Random select for stairs
4. Stairs entity with interaction trigger
5. On interact: increment Level, clear chunks, regenerate
6. StairsUp at spawn (return to Level-1)
7. Test 5+ level descent

**Test**: Descend to level 5, distinct layouts

---

**Total**: 3-4 weeks

**Critical path**: A → B → D (skip C if needed)

**Parallel**: C and E independent

---

## Scope

### Build
- Single level at a time
- 3 tiles: wall, floor, destroyed
- 2km × 2km levels

### Don't Build
- Multiple biomes
- Resources/ore
- Room templates
- Lighting
- Minimap
- Water/hazards
- Multiple simultaneous levels

## Deliverables
1. Walk 1km from center, seamless chunks
2. Shoot walls, break instantly
3. 60fps with 25 chunks (5×5)
4. Level resource tracks floor
5. Destruction persists in SQLite
6. Stairs spawn in reachable outer ring

## Next Phase
Stairs complete, everything works. Just:
- Walk over stairs → increment Level, regenerate
- StairsUp at spawn → return to Level-1
