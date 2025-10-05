# Event-Driven Chunk Management System Specification

## Overview

A decoupled chunk management system where the core chunking plugin is responsible only for tracking chunk loaders and publishing load/unload events. Content systems (terrain, mobs, items) independently subscribe to these events and manage their own resources.

## Core Principles

1. **Single Responsibility**: ChunkingPlugin tracks spatial regions and loader positions, nothing else
2. **Event-Driven**: All chunk lifecycle changes communicated via Bevy events
3. **Independent Content**: Each system decides what "loading a chunk" means for its domain
4. **Reference Counting**: Chunks remain loaded while ANY loader requires them
5. **Global Chunk Size**: All systems use the same fixed chunk size (CHUNK_SIZE constant). Different granularities are not supported

---

## Core Components

### ChunkLoader Component

A component attached to entities that require chunks to be loaded around them.

**Fields:**
- `radius: i32` - Load chunks within this Manhattan distance (forms a square region)
- `unload_radius: i32` - Unload chunks beyond this distance (must be >= radius)
- `preload_radius: i32` - Optional preload distance for background loading (must be >= radius, <= unload_radius)

**Usage:**
- Attach to player entities, AI directors, network replication zones, etc.
- Multiple loaders can coexist; chunks remain loaded if ANY loader needs them
- Preload radius allows loading chunks in advance without blocking critical loads

**Edge Cases:**
- Loader with radius=0 loads only the chunk it's standing in
- Loader destroyed: triggers unload check for all its chunks next frame
- Loader teleports: may trigger both loads and unloads in same frame
- Preload chunks load with lower priority than critical chunks

### ChunkCoord Type

Simply `IVec2` representing chunk grid coordinates.

**Conversion Functions:**
- `world_pos_to_chunk_coord(Vec2) -> ChunkCoord` - Floor division by chunk world size
- `chunk_coord_to_world_pos(ChunkCoord) -> Vec2` - Returns center of chunk in world space

### LoadChunk Event

Published when a chunk needs to be loaded.

**Fields:**
- `pos: ChunkCoord` - Which chunk to load
- `world_pos: Vec2` - Center of chunk in world space (convenience field)
- `loaded_for: Vec<Entity>` - List of loader entities requiring this chunk

**Guarantees:**
- Never published twice for same chunk without intervening UnloadChunk
- Published before first frame where chunk is "required"
- All LoadChunk events for a frame published before PreloadChunk and UnloadChunk events

**Edge Cases:**
- Same chunk requested by multiple loaders: only ONE LoadChunk event
- Chunk already loaded when new loader arrives: NO new LoadChunk event
- Loader moves but chunk still in range: NO event

### PreloadChunk Event

Published when a chunk should be loaded in the background (non-critical).

**Fields:**
- `pos: ChunkCoord` - Which chunk to preload
- `world_pos: Vec2` - Center of chunk in world space (convenience field)
- `loaded_for: Vec<Entity>` - List of loader entities requesting preload

**Guarantees:**
- Published after all LoadChunk events in same frame
- Published before UnloadChunk events
- Never published if chunk already has LoadChunk (critical takes precedence)

**Edge Cases:**
- Preload chunk becomes critical before loading: LoadChunk event published, no PreloadChunk
- Chunk already loading/loaded: NO new PreloadChunk event

### UnloadChunk Event

Published when NO loaders require a chunk anymore.

**Fields:**
- `pos: ChunkCoord` - Which chunk to unload
- `world_pos: Vec2` - Center of chunk in world space (convenience field)

**Guarantees:**
- Only published when refcount reaches zero
- Published AFTER all LoadChunk events in same frame
- Systems receive in deterministic order (use system ordering)

**Edge Cases:**
- Loader destroyed: triggers unload check next frame (not same frame)
- Loader radius shrinks: triggers unloads immediately
- Chunk unloaded then immediately re-requested: will see UnloadChunk then LoadChunk in successive frames

---

## ChunkingPlugin Implementation

### ChunkRegistry Resource

Tracks which chunks are currently required by which loaders.

**Internal State:**
- `active_chunks: HashMap<ChunkCoord, HashSet<Entity>>` - Maps chunk to set of loader entities requiring it
- `previous_loader_positions: HashMap<Entity, (ChunkCoord, i32, i32)>` - Cache of last known (position, radius, unload_radius) per loader

**Purpose:**
- Reference counting via set size
- Fast diff calculation on loader movement
- Cleanup when loaders despawn

### Core System: track_chunk_loaders

Runs every frame to detect changes and publish events.

**Algorithm:**

1. **For each ChunkLoader entity:**
   - Calculate current chunk position
   - Calculate critical chunks (square region from radius)
   - Calculate preload chunks (square region from preload_radius, if specified)
   - Compare against previous_loader_positions cache
   - If changed: record delta

2. **For despawned loaders:**
   - Detect via previous_loader_positions having entries for non-existent entities
   - Remove from all active_chunks sets
   - Clear from cache

3. **Build event lists:**
   - Collect chunks that gained loaders in critical range: candidates for LoadChunk
   - Collect chunks that gained loaders in preload range only: candidates for PreloadChunk
   - Collect chunks that lost all loaders: candidates for UnloadChunk
   - Filter LoadChunk: only if refcount went 0→1
   - Filter PreloadChunk: only if not already critical, refcount went 0→1
   - Filter UnloadChunk: only if refcount went 1→0

4. **Publish events:**
   - Send all LoadChunk events first
   - Then send all PreloadChunk events
   - Then send all UnloadChunk events
   - Update cache with new positions

**Edge Cases Handled:**

- **Loader spawned mid-frame**: Detected as new entry, not in cache. Generates LoadChunk events for entire radius.
- **Loader teleports**: Treated as despawn + respawn. May generate many loads and unloads.
- **Loader component removed**: Detected via despawned entity logic (component removal = entity filter change).
- **Radius changed**: Detected as position change. Expansion generates loads, shrinkage generates unloads.
- **Two loaders move to overlap**: Second loader's chunks already have refcount ≥1, no duplicate LoadChunk.
- **Last loader leaves area**: Refcount drops to 0, UnloadChunk published.

### System Ordering

Must run in Update schedule, no special ordering required relative to content systems (they consume events at their own pace).

### Performance Considerations

**Fast Path:**
- Loaders that don't move: zero HashMap operations
- Cached previous positions avoid redundant chunk coordinate calculations
- HashSet operations are O(1) average

**Slow Path:**
- Loader teleporting 1000 chunks away: O(radius²) for old chunks + O(radius²) for new chunks
- Mitigation: unload_radius should be slightly larger than radius to create hysteresis, reducing thrashing

---

## Edge Cases & Failure Modes

### Multiple Loaders on Same Entity

**Scenario**: Entity has two ChunkLoader components (e.g., visual radius + audio radius).

**Behavior**: Bevy queries return multiple components per entity. Both are tracked independently with same entity ID. Works correctly - chunks load if either loader needs them.

**Caveat**: May confuse debugging. Recommend against this pattern; use single loader with max radius.

### Loader Despawned, Chunk Mid-Load

**Scenario**: LoadChunk event published, terrain system starts async task, loader despawns before task completes.

**Behavior**: UnloadChunk event published next frame. Terrain system receives both LoadChunk and UnloadChunk. Must handle gracefully:
- Cancel async task if still pending
- Or: complete load but immediately despawn result

**Recommendation**: Track pending loads per chunk, cancel on unload.

### Event Overflow

**Scenario**: 100 loaders all teleport simultaneously, generating 10,000+ events.

**Behavior**: Bevy event queues don't have size limits. All events delivered. May cause frame spike.

**Mitigation**: 
- Spread loader updates across multiple frames (staggered update groups)
- Or: batch chunk operations (load multiple chunks per event)

### Chunk Load Failure

**Scenario**: LoadChunk event published, but disk I/O fails or chunk generation errors.

**Behavior**: Core system doesn't care. Content system should:
- Log error
- Either retry or leave chunk unloaded
- No automatic UnloadChunk sent (refcount still >0)

**Recommendation**: Content systems should track load failures separately. Optionally publish ChunkLoadFailed event for debugging UI.

### Unload During Active Use

**Scenario**: Player standing in chunk A, item dropped in chunk B (outside radius), player moves away. Item still updating physics but chunk unloads.

**Behavior**: 
- UnloadChunk event published for chunk B
- Terrain system despawns tiles
- Item system must decide: despawn item or keep it (orphaned entity)

**Recommendation**: Items/mobs should subscribe to UnloadChunk and either:
- Serialize state and despawn (save to "chunk storage")
- Switch to simplified simulation (no collision)
- Force-keep chunk loaded via invisible loader component

### Event Ordering Between Frames

**Scenario**: Frame N publishes LoadChunk(5,5). Frame N+1 publishes UnloadChunk(5,5).

**Behavior**: Content systems see both in order. Must handle:
- Start load task frame N
- Cancel/complete task frame N+1

**Critical**: Do NOT assume chunk stays loaded for minimum duration. Hysteresis (unload_radius > radius) helps but doesn't guarantee.

### Zero Radius Loader

**Scenario**: ChunkLoader with radius=0.

**Behavior**: Loads only the chunk containing the loader entity's position. Valid use case (e.g., stationary camera showing single room).

### Negative Radius

**Scenario**: ChunkLoader with radius=-5.

**Behavior**: Undefined. Validation should clamp radius to max(0, radius) during component insertion or system run.

---

## Integration: Tilemap Terrain

This section describes how the existing tilemap system integrates with the event-driven chunk model.

### TerrainPlugin

Separate plugin from ChunkingPlugin. Manages tile spawning/despawning.

**Resources:**
- `TerrainChunks: HashMap<ChunkCoord, TerrainChunkState>` - Per-chunk state machine
- `texture_handle: Handle<Image>` - Shared tilemap texture
- `macro_map: Vec<Vec<bool>>` - World-level density map for procedural generation

**TerrainChunkState Enum:**
- `Loading { task: Task<ChunkData> }` - Async generation in progress
- `Loaded { entity: Entity, tiles: [[TileType; N]; N] }` - Spawned in world

### System: handle_chunk_load_events

Listens for LoadChunk and PreloadChunk events and starts async generation.

**Algorithm:**

1. Read all LoadChunk events, then all PreloadChunk events
2. **Prioritize chunks**: 
   - LoadChunk events process first
   - Within each tier, calculate distance from each chunk to its nearest loader (using `loaded_for` entities)
   - Process closest chunks first
3. For each event (in priority order):
   - Check if chunk already in TerrainChunks map
   - If not: spawn async task to generate tiles
   - Store task in TerrainChunks as Loading state
4. Async task generates:
   - 64x64 tile grid via procedural generation (existing logic)
   - Wall collision regions (existing logic)
   - Returns ChunkData struct

**Edge Cases:**
- LoadChunk for already-loaded chunk: Ignore (log warning for debugging)
- LoadChunk while Loading: Ignore (task already running)
- Multiple events in same frame: Process LoadChunk before PreloadChunk, closest to loaders first within each tier
- PreloadChunk becomes LoadChunk before loading: Already queued, no action needed

### System: poll_terrain_loading_tasks

Checks async tasks and spawns entities when complete.

**Algorithm:**

1. Iterate TerrainChunks entries in Loading state
2. For each, check if task.is_finished()
3. If finished:
   - Extract ChunkData from task
   - Spawn parent entity with ChunkParent component
   - Spawn tilemap children using bevy_ecs_tilemap
   - Spawn collision rectangles from wall_regions
   - Update TerrainChunks to Loaded state with entity references

**Edge Cases:**
- Task finishes same frame as UnloadChunk: See next system
- Task errors: Log and remove from map, leave chunk ungenerated

### System: handle_chunk_unload_events

Listens for UnloadChunk events and despawns terrain.

**Algorithm:**

1. Read all UnloadChunk events
2. For each event:
   - Look up chunk in TerrainChunks map
   - If Loading: cancel/drop task, remove from map
   - If Loaded: despawn entity tree, remove from map
   - If not found: Ignore (already unloaded or never loaded)

**Edge Cases:**
- UnloadChunk while Loading: Drop task immediately, no entities spawned
- UnloadChunk for never-loaded chunk: Silent no-op
- UnloadChunk same frame task finishes: Race condition - use system ordering (unload runs after poll)

### System Ordering

```
track_chunk_loaders (ChunkingPlugin)
  ↓ publishes events: LoadChunk, PreloadChunk, UnloadChunk
handle_chunk_load_events (TerrainPlugin)
poll_terrain_loading_tasks (TerrainPlugin)
handle_chunk_unload_events (TerrainPlugin)
  ↓ clears events
```

Ensures:
- Events published before consumption
- Loads checked before unloads in same frame
- Unload can catch tasks that finished this frame

### Terrain-Specific Edge Cases

**Chunk Modified During Unload**

User digs tile, marks chunk dirty, then walks away. UnloadChunk published.

**Behavior**: 
- handle_chunk_unload_events should check dirty flag
- If dirty: serialize chunk state to disk/storage before despawning
- Requires TerrainChunkState to track dirty: bool

**Chunk Re-Requested After Unload**

Player walks away (unload), then back (load). Chunk was modified (dirty).

**Behavior**:
- handle_chunk_load_events checks if saved state exists
- If yes: deserialize instead of generating
- If no: generate fresh

**Async Task Outlives Plugin**

App shutting down, async tasks still running.

**Behavior**: 
- Bevy's AsyncComputeTaskPool automatically cancels tasks on shutdown
- No manual cleanup needed
- If tasks write to external resources: must implement Drop to cancel I/O

---

## Visual Debug Tools

- Gizmo overlay showing loaded chunks (green squares)
- Gizmo showing loader radii (circles/squares)
- UI panel: active chunks count, events this frame, refcounts per chunk
- Color-code chunks: blue=loading, green=loaded, red=unloading

---

## Migration Notes

### From Current System

1. Extract ChunkManager terrain logic into TerrainChunkPlugin
2. Replace ChunkManager.request_chunk_load calls with LoadChunk event readers
3. Replace ChunkManager.unload_chunk calls with UnloadChunk event readers
4. Remove ChunkManager.calculate_required_chunks logic (now in track_chunk_loaders)
5. Attach ChunkLoader component to player entity
6. Delete old ChunkManager resource

### Backwards Compatibility

Not required. Clean break from existing ChunkManager implementation.

---

## Glossary

- **ChunkCoord**: Integer grid coordinates (x, y) identifying a chunk
- **Loader**: Entity with ChunkLoader component
- **Refcount**: Number of loaders requiring a specific chunk
- **Hysteresis**: Gap between load radius and unload radius to prevent thrashing
- **Manhattan distance**: Sum of absolute differences (|dx| + |dy|), forms square regions
- **Chunk thrashing**: Rapid load/unload cycles due to movement on chunk boundary
