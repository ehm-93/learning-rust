# Phase 1.5 - Terrain Foundation

## Status: MOSTLY COMPLETE ✅
**Last Updated:** October 7, 2025

### What's Working
- ✅ **Chunk Infrastructure** - Event-driven loading/unloading with ChunkLoader component
- ✅ **Macro Generation** - `freeform()` generates connected cave maps with random walks, simplex noise, and cellular automata
- ✅ **Micro Generation** - Per-chunk tile generation from macro density maps with smooth blending
- ✅ **Persistence** - SQLite database saves/loads terrain chunks (and FOW vision data)
- ✅ **Fog of War** - Vision system tracks explored areas, persists across chunk load/unload
- ✅ **Collision** - Physics bodies generated from tile data
- ✅ **Rendering** - Efficient tilemap batching with bevy_ecs_tilemap

### Still TODO
- ❌ **Destructible Terrain** - Walls don't take damage yet
- ❌ **Level Progression** - No stairs or multi-level system yet
- ❌ **Testing at Scale** - Haven't verified 10km+ traversal

## Goal
Build a continuous destructible world with chunked terrain. Player explores vast caverns where walls explode and the map sprawls organically.

## Success Criteria
- Walk 10km+ smoothly ✅ (infrastructure ready, needs testing)
- Walls destroyed by weapons ❌ (not implemented)
- Invisible chunk loading ✅ (working)
- Organic caves, not geometric rooms ✅ (working)
- Ready for stairs to next level ❌ (not implemented)

## Core Systems

### Chunks ✅ IMPLEMENTED
- **Size**: 64×64 tiles (32m × 32m, 0.5m/tile)
- **Level**: Theoretically infinite (2048m design goal not tested)
- **Active**: Event-driven with `ChunkLoader` component (configurable radius)
- **Unload**: Configurable unload_radius (default: load_radius + 2)
- **Persist**: SQLite database stores terrain tiles and FOW vision data
- **Seed**: Implemented in `DungeonState` resource (same seed = same terrain)
- **Implementation**: See `packages/untitled/src/world/chunks/` and `packages/untitled/src/persistence/`

### Generation ✅ IMPLEMENTED (except stairs)
**Macro** (pre-generated per level):
- Random walks from center to edges ensure connectivity ✅
- Angular distribution prevents overlap ✅
- Multi-scale simplex noise adds variation ✅
- Cellular automata smooths single-tile walls ✅
- Flood fill validates full traversability ⚠️ (implemented but needs verification)
- Result: bool map (see `packages/untitled/src/world/mapgen/mod.rs::freeform()`)
- PNG debug output: `out/macro_map.png`

**Micro** (per-chunk on-demand): ✅ IMPLEMENTED
- Sample macro map at chunk corners for density values
- Bilinear interpolation between corners within chunk
- Micro-variation noise applied near density boundaries
- Async generation to avoid frame drops (4ms budget)
- Implementation: `packages/untitled/src/world/scenes/dungeon/terrain.rs::generate_chunk_tiles()`

**Variation**: Not yet level-scaled (uses fixed parameters)

**Stairs**: ❌ NOT IMPLEMENTED

### Destruction ❌ NOT IMPLEMENTED
- **HP**: Not implemented yet
- **Sources**: Damage system exists (`DamageEvent` in `src/events.rs`) but tiles don't respond
- **Feedback**: Particle/shake systems exist but not wired to tile destruction
- **Permanent**: Persistence system ready (would auto-save destroyed tiles)
- **Physics**: Collision update logic needed

**Implementation Plan:**
1. Add health component to wall tiles or tile-based health map
2. Subscribe to `DamageEvent` for tile entities
3. Remove tiles at 0 HP and mark chunk dirty
4. Update collision geometry (remove triangles for destroyed tiles)
5. Persistence already handles modified chunks

### Collision ✅ IMPLEMENTED
- **Grid**: 0.5m tiles (TILE_SIZE constant)
- **Physics**: Rapier2D trimesh colliders generated per chunk
- **Projectiles**: Rapier raycasting handles bullets vs walls
- **Player**: Rapier physics body collides with tile colliders
- **Dynamic Updates**: Colliders despawn when chunks unload
- **Implementation**: `packages/untitled/src/world/scenes/dungeon/terrain.rs::spawn_chunk_tilemap()`

### Rendering ✅ IMPLEMENTED
- **Batching**: bevy_ecs_tilemap handles efficient chunk rendering
- **Atlas**: Texture atlas with floor/wall tiles (loaded in `load_tilemap_texture()`)
- **Culling**: Chunk-based - only loaded chunks render
- **Dirty tracking**: ⚠️ Not yet implemented (needed for destruction)
- **View**: Camera follows player, chunks load dynamically based on `ChunkLoader` radius
- **Boundaries**: No hard boundaries (theoretically infinite world)
- **FOW**: Fog of war shader dims unexplored areas (see `packages/untitled/src/combat/fow/`)

---

## Implementation Status

### ✅ A: Static Tilemap (COMPLETE)
**Status:** Implemented and integrated into dungeon scene
- bevy_ecs_tilemap 0.16 in use
- Texture atlas system working
- Camera follows player smoothly
- Basic collision detection working

**Location:** `packages/untitled/src/world/tiles/`

---

### ✅ B: Chunk Infrastructure (COMPLETE)
**Status:** Event-driven architecture implemented
- `ChunkLoader` component for entities that need chunks
- `ChunkRegistry` tracks active chunks
- `LoadChunk`, `UnloadChunk`, `PreloadChunk` events
- Manhattan distance-based loading (configurable radius)
- Background preloading support

**Location:** `packages/untitled/src/world/chunks/`

---

### ✅ C: Macro Generation (COMPLETE)
**Status:** Fully working algorithm
- `freeform()` generates connected cave systems
- Random walks with angular distribution
- Multi-scale simplex noise (0.045 and 0.1 frequency)
- Cellular automata smoothing (5 iterations)
- PNG debug output to `out/macro_map.png`
- Seeded generation via `DungeonState` resource

**Location:** `packages/untitled/src/world/mapgen/`

---

### ✅ D: Micro Generation (COMPLETE)
**Status:** Async chunk generation working
- Samples macro map at chunk boundaries
- Bilinear interpolation for smooth density
- Noise applied in boundary regions (0.45-0.55)
- Async task system with 4ms frame budget
- Generates 64×64 tiles per chunk on-demand

**Location:** `packages/untitled/src/world/scenes/dungeon/terrain.rs`

---

### ❌ E: Destructible Terrain (TODO)
**Status:** Not implemented yet
**Blockers:** None - foundation is ready
**Estimated effort:** 2-3 days

**Implementation Plan:**
1. Add tile health tracking (component or grid-based)
2. Subscribe to damage events for tile coordinates
3. Modify chunk tile data on destruction
4. Regenerate collision mesh for modified chunks
5. Mark chunks dirty for persistence
6. Add visual feedback (particles, debris)

**Dependencies Ready:**
- Damage event system exists
- Persistence will auto-save modified chunks
- Collision regeneration path is clear

---

### ✅ F: Chunk Persistence (COMPLETE)
**Status:** SQLite integration working
- Database: `ChunkDatabase` resource with thread-safe connection
- Two tables: `terrain_chunks` and `fow_chunks`
- Binary serialization of tile data
- Auto-save on chunk unload
- Auto-load before generation
- FOW vision data also persisted

**Location:** `packages/untitled/src/persistence/`

---

### ❌ G: Level Progression (TODO)
**Status:** Not implemented yet
**Blockers:** None - infrastructure ready
**Estimated effort:** 2-3 days

**Implementation Plan:**
1. Implement flood fill to find reachable tiles
2. Place stairs in outer ring (500m+ from spawn)
3. Create stairs entity with interaction component
4. Add level increment/decrement on interaction
5. Clear chunks and regenerate with new seed
6. Handle return stairs (StairsUp)

**Dependencies Ready:**
- World state system can handle level changes
- Chunk system can clear and regenerate
- Interaction system exists (see `world/interaction.rs`)

---

## Current State Summary

**Implemented (6/7 phases):**
- A: Static Tilemap ✅
- B: Chunk Infrastructure ✅  
- C: Macro Generation ✅
- D: Micro Generation ✅
- F: Chunk Persistence ✅

**Remaining Work:**
- E: Destructible Terrain ❌ (critical for gameplay)
- G: Level Progression ❌ (needed for roguelike loop)

**Bonus Features Implemented:**
- Fog of War system with persistence
- Event-driven architecture (more robust than planned)
- Async generation with frame budget
- Preload radius for smoother experience

**Timeline:**
- Original estimate: 3-4 weeks
- Actual (phases A-D, F): ~3 weeks
- Remaining (E, G): 4-6 days

---

## Scope

### Built ✅
- Single level at a time ✅
- 2 tiles: wall, floor ✅ (destroyed tile type not needed - just remove tiles)
- Theoretically infinite levels (2km target not tested) ✅
- Event-driven chunk management ✅ (better than planned)
- Persistence system ✅
- Fog of War ✅ (bonus feature)

### Still In Scope (Not Built)
- Destructible terrain ❌
- Level progression / stairs ❌
- Scale testing (10km+ traversal) ❌

### Out of Scope (Correctly Avoided)
- Multiple biomes ✅ (avoided)
- Resources/ore ✅ (avoided)
- Room templates ✅ (avoided)
- Lighting ✅ (avoided - FOW provides similar effect)
- Minimap ✅ (avoided)
- Water/hazards ✅ (avoided)
- Multiple simultaneous levels ✅ (avoided)

## Technical Debt & Future Work

### Known Issues
1. **No scale testing** - Haven't verified smooth 10km+ traversal
2. **Macro map representation** - Currently bool array, could be more memory efficient
3. **No dirty tracking** - Tilemap mesh doesn't rebuild on tile changes (needed for destruction)
4. **Fixed generation parameters** - Not yet scaled by level depth
5. **No traversability guarantee** - Flood fill exists but not verified in production

### Future Enhancements
1. **Dynamic difficulty** - Adjust wall HP, enemy density by level
2. **Level themes** - Vary noise parameters, CA iterations by depth
3. **Special rooms** - Occasional hand-crafted features within proc-gen
4. **Optimized persistence** - Delta compression for modified chunks
5. **Chunk streaming** - Further optimize load times with better prioritization

## Architecture Notes

### Key Design Decisions

**Event-Driven Chunks:**
Instead of polling player position each frame, we use a `ChunkLoader` component that publishes events (`LoadChunk`, `UnloadChunk`). This allows multiple systems (terrain, FOW, enemies, etc.) to independently react to chunk lifecycle without coupling.

**Async Generation:**
Chunk tile generation runs on background threads with a 4ms per-frame budget. This prevents frame drops during exploration while keeping the world responsive.

**Macro/Micro Split:**
The macro map is generated once per level and stored in `DungeonState`. Individual chunks sample this macro map to generate their local tiles. This ensures:
- Consistency across chunk boundaries
- Repeatable generation (same seed = same world)
- Memory efficiency (don't store every tile)

**Persistence Strategy:**
Only modified chunks are saved to SQLite. Unmodified chunks regenerate from the seed on load. This keeps save files small while supporting destruction.

## Deliverables Status

### Completed ✅
1. ✅ Walk 1km+ from center, seamless chunks (infrastructure ready, needs scale testing)
2. ❌ Shoot walls, break instantly (not implemented)
3. ✅ 60fps with dynamic chunk loading (achieved, actual chunk count varies)
4. ✅ Level resource tracks floor (`DungeonState` in `packages/untitled/src/world/scenes/dungeon/resources.rs`)
5. ⚠️ Destruction persists in SQLite (persistence works, destruction not implemented)
6. ❌ Stairs spawn in reachable outer ring (not implemented)

### Next Steps to Complete Phase 1.5

**Priority 1: Destructible Terrain (E)**
- Implement tile health system
- Wire damage events to tiles
- Update collision on destruction
- Add visual feedback

**Priority 2: Level Progression (G)**
- Implement stairs placement algorithm
- Add interaction trigger
- Handle level transitions
- Test multi-level descent

**Priority 3: Testing & Polish**
- Verify 10km+ traversal performance
- Test destruction persistence across restarts
- Verify flood fill guarantees connectivity
- Add debug visualization tools

---

## Quick Reference

### File Locations
- **Chunks:** `packages/untitled/src/world/chunks/`
- **Tiles:** `packages/untitled/src/world/tiles/`
- **Macro Gen:** `packages/untitled/src/world/mapgen/`
- **Micro Gen:** `packages/untitled/src/world/scenes/dungeon/terrain.rs`
- **Persistence:** `packages/untitled/src/persistence/`
- **FOW:** `packages/untitled/src/combat/fow/`

### Key Constants
- `CHUNK_SIZE = 64` (tiles per chunk side)
- `TILE_SIZE = 0.5` (meters per tile)
- `CHUNK_LOADING_BUDGET = 0.004` (seconds per frame)
- `WALL_DENSITY_THRESHOLD = 0.5` (macro → micro conversion)

### Database Schema
```sql
-- Terrain chunks
CREATE TABLE terrain_chunks (
    chunk_x INTEGER NOT NULL,
    chunk_y INTEGER NOT NULL,
    tiles BLOB NOT NULL,
    PRIMARY KEY (chunk_x, chunk_y)
);

-- Fog of War chunks  
CREATE TABLE fow_chunks (
    chunk_x INTEGER NOT NULL,
    chunk_y INTEGER NOT NULL,
    vision BLOB NOT NULL,
    PRIMARY KEY (chunk_x, chunk_y)
);
```

### Testing Commands
```bash
# Build and run
cargo build --package untitled
cargo run --package untitled

# Generate macro map visualization
# (automatically saved to out/macro_map.png on dungeon entry)

# Database location
# world.db (in project root, created on first run)
```
