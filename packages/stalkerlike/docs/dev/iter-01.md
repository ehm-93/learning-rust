# Iteration 1: Editor MVP - Scene Building Foundations

## Context
With Iteration 0 complete (game mode with save/load, player controller, basic 3D scene) we now need a minimal viable editor that can create and manipulate 3D scenes. This iteration focuses on the essential tools needed to hand-craft level geometry and props.

This editor will create **narrative chunks** - hand-authored set pieces that define the critical path and major encounters. These will later integrate with the procedural generation system (see `uncommitted/procedural_generation.md`) which fills in the connective tissue between your hand-crafted content.

## End State Goal
A functional level editor with basic transform tools, asset placement, and scene persistence. You should be able to build a simple multi-room environment (ideally within a 32x32x32m chunk), save it to YAML, and play through it in game mode. 

The editor produces human-readable YAML files that will eventually feed into the two-database persistence system (static DB for immutable content, dynamic DB for save states - see `uncommitted/persistence.md`).

## Core Features

### 1. Editor Camera Controller
**Goal**: Navigate the 3D scene freely to build levels

- **Free-Fly Camera**: WASD for movement, mouse look for rotation
- **Speed Control**: Shift to move faster, Ctrl to move slower
- **Vertical Movement**: Q (down) and E (up) for altitude control
- **Grid Display**: Visual grid on ground plane for spatial reference
- **Chunk Bounds Visualization**: Optional display of 32m chunk boundaries (toggle with B key)
- **Unlocked Mouse Mode**: Left alt to toggle between locked mouse (camera control) and free mouse mode (scene interaction)
- **Context menu**: Right click in free mouse mode to open context menu

**Technical Details**:
- Camera component with velocity-based smooth movement
- Configurable base speed (default 5.0 units/sec)
- Speed multiplier when Shift held (4x) or Ctrl held (0.25x)
- Mouse look with sensitivity setting
- Smooth camera interpolation for focus operations

### 2. Transform Gizmo System
**Goal**: Visually manipulate object position, rotation, and scale

- **Mode Switching**: F to cycle between translate, rotate, and scale, shift-f to reverse
- **Visual Gizmos**: Color-coded 3D arrows/arcs for each axis
  - Red: X-axis
  - Green: Y-axis  
  - Blue: Z-axis
- **Mouse Interaction**: Click and drag on gizmo handles
- **Grid Snapping**: Optional snap-to-grid (0.5m for position, 15° for rotation, toggle with G key)
- **Numeric Input**: Type exact values in inspector panel

**Technical Details**:
- Separate gizmo render system (always on top)
- bevy_mod_picking for handle selection
- Constraint transformations based on selected axis
- Visual feedback for active axis (highlight)
- Grid snapping: 0.5m translation, 15° rotation (toggle with G key)
- Snap indicator shows grid alignment visually

### 3. Object Selection System
**Goal**: Pick and manage scene objects for editing

- **Click Selection**: Left-click on object in viewport to select
- **Multi-Select**: Ctrl+click to add/remove from selection (deferred - requires grouping support)
- **Box Select**: Click and drag to select multiple objects (deferred with multi-select)
- **Deselect All**: Escape or click empty space
- **Visual Feedback**: Selected objects highlighted with outline shader
- **Hierarchy Display**: List selected objects in inspector panel

**Technical Details**:
- Ray-casting from mouse to world for picking
- Selection state component on entities
- Outline rendering using post-process or mesh duplication technique
- Single-selection only in MVP (multi-select comes with grouping in week 3)

### 4. Asset Library & Placement
**Goal**: Spawn primitive shapes and basic props into the scene

**Primitive Meshes** (MVP set):
- Cube (1x1x1m default) - basic building block
- Sphere (1m diameter) - props, lights
- Cylinder (1m diameter x 2m height) - pipes, pillars
- Plane (10x10m floor section) - floors, walls
- Capsule (0.5m diameter x 2m height) - character proxies

**Note**: These primitives use vertex colors (see `uncommitted/art_pipeline.md` for low-poly aesthetic). Future iterations will add custom GLTF model imports.

**Placement Flow**:
- Asset browser panel (side panel or modal)
- Click asset to enter Place mode
- Hover over scene shows ghost preview
- Click to spawn at hover location
- Escape to cancel placement

**Technical Details**:
- Asset catalog resource with mesh/material definitions
- Spawn system that creates entities with appropriate components
- Preview system with semi-transparent rendering
- Ground-plane snapping for initial placement

### 5. Scene Hierarchy & Inspector
**Goal**: View and edit scene structure and properties

**Hierarchy Panel**:
- Tree view of all scene entities
- Group/ungroup functionality
- Show/hide toggle per entity
- Lock/unlock toggle (prevent selection)
- Rename entities

**Inspector Panel**:
- Transform component (position, rotation, scale)
- Mesh/material display
- Physics properties (collider type, rigid body)
- Add/remove components button
- Custom metadata fields (key-value pairs)

**Technical Details**:
- EGUI panels for UI
- Entity query systems for populating views
- Component editing with reflection system
- Validation warnings for invalid configurations

### 6. Editor Scene Persistence
**Goal**: Save and load editor scenes independently from game saves

**Scene Format**:
- **YAML serialization** (human-readable, version-control friendly)
- Store entity hierarchy with all components
- Material and mesh references (not embedded assets)
- Camera position for resuming editing
- Editor metadata (last modified, author, chunk bounds)
- **Chunk-aware**: Store world position and size (default 32x32x32m to align with streaming system)

**File Structure**:
```yaml
chunk:
  id: "mining_shaft_7"
  position: [0, -500, 0]  # World coordinates (Y is depth)
  bounds: [32, 32, 32]    # Size in meters
  faction: "union"        # Optional: faction control

entities:
  - name: "floor"
    position: [0, 0, 0]
    rotation: [0, 0, 0]
    scale: [10, 0.1, 10]
    components:
      - type: mesh
        primitive: cube
      - type: collider
        shape: box
        
  - name: "mining_drill"
    position: [5.2, 1.0, 3.1]
    rotation: [0, 45, 0]
    # ... etc
```

**File Operations**:
- **New Scene (Ctrl+N)**: Clear scene, prompt to save
- **Open Scene (Ctrl+O)**: File picker dialog
- **Save Scene (Ctrl+S)**: Save to current file
- **Save As (Ctrl+Shift+S)**: Pick new file location
- **Auto-save**: Every 5 minutes to temp file

**Technical Details**:
- Scene serialization using `serde_yaml`
- Marker component for editor entities vs game entities
- File path resource to track current scene file
- Scene dirty flag to prompt on exit with unsaved changes
- YAML files stored in `assets/levels/` for version control
- Future: Build pipeline converts YAML → SQLite (static DB)

### 7. Basic Entity Management
**Goal**: Create, duplicate, and delete scene objects

- **Duplicate (Ctrl+D)**: Copy selected entities with offset (+1m on X-axis, or +1m on dominant horizontal axis if rotated)
- **Delete (Del)**: Remove selected entities from scene
- **Undo/Redo (Ctrl+Z/Y)**: Command pattern for operation history (deferred to Iteration 2 - foundation must be solid first)
- **Group (Ctrl+G)**: Create parent entity for selected objects
- **Ungroup (Ctrl+Shift+G)**: Flatten hierarchy level

**Technical Details**:
- Entity cloning system that copies all components
- Parent/child transform hierarchies
- Duplicate offset: +1m along X-axis (or dominant horizontal axis for rotated groups)
- Undo/redo deferred until core editing loop is proven (avoid half-working undo)

### 8. Editor ↔ Game Mode Bridge
**Goal**: Test levels directly from the editor

**Play Mode**:
- P key or button to enter play mode
- Spawn player at designated spawn point (or origin if none)
- All editor UI hidden, game systems active
- ESC to exit back to editor
- Scene state preserved when returning to editor

**Technical Details**:
- State machine: `Editor` ↔ `EditorPlayMode`
- Serialize scene state before entering play mode
- Game systems run in `EditorPlayMode` state
- Restore editor state on exit (camera position, selection, etc.)
- Temporary scene saved to allow revert

## System Architecture

### Module Organization (Implemented)
```
src/
├── main.rs              # Entry point with --editor flag handling
├── editor/
│   ├── mod.rs          # EditorPlugin - coordinates all editor systems
│   ├── components.rs   # EditorCamera, EditorEntity
│   ├── resources.rs    # EditorMouseMotion, (future: GridConfig, EditorState)
│   └── camera.rs       # Camera movement and mouse look systems
├── game/               # Game mode (already implemented)
└── shared/             # Shared components/systems between editor and game (future)
```

### World Scale Conventions
- **1 unit = 1 meter** in world space
- **Chunk size = 32x32x32 meters** (aligns with streaming system)
- **Y-axis = depth**: 0 is surface, negative values go deeper (e.g., -500 = 500m deep)
- **Coordinate system**: Right-handed (X: East/West, Y: Up/Down, Z: North/South)

### Component Structure
```rust
// Editor-specific components (implemented in src/editor/components.rs)
#[derive(Component)]
struct EditorCamera {
    sensitivity: f32,
    pitch: f32,        // Radians
    yaw: f32,          // Radians
    velocity: Vec3,    // Smooth movement
    base_speed: f32,   // 5.0 m/s default
    mouse_locked: bool,
}

#[derive(Component)]
struct EditorEntity;  // Marks entities as part of editor scene

// Future components
#[derive(Component)]
struct Selectable;

#[derive(Component)]
struct Selected;

#[derive(Component)]
struct ChunkMetadata {
    chunk_id: String,
    world_position: Vec3,  // Position in world coordinates
    bounds: Vec3,          // Size (default [32, 32, 32])
    faction: Option<String>,
}
```

### Integration Notes
- **Current scope**: Simple YAML save/load
- **Future (post-MVP)**: YAML files feed into `game_static.db` via build pipeline
- **Chunk streaming**: Editor scenes will become loadable chunks in the world streaming system
- **Procedural integration**: Hand-authored chunks act as "islands" connected by procedural generation

See `uncommitted/persistence.md` for the full two-database architecture and `uncommitted/world_architecture.md` for chunk streaming details.

## UI Layout

```
┌─────────────────────────────────────────────────────────────┐
│ File  Edit  View  Tools  Play                         [P]   │ <- Menu Bar
├──────────┬──────────────────────────────────────┬───────────┤
│          │                                      │ Inspector │
│ Hierarchy│                                      ├───────────┤
│          │                                      │ Transform │
│ ┠─ Floor │         Viewport                     │  X: 0.0   │
│ ┠─ Wall1 │      (3D Scene)                      │  Y: 1.0   │
│ ┠─ Wall2 │                                      │  Z: 5.0   │
│ ┠─ Cube  │                                      ├───────────┤
│ ┗─ Light │                                      │ Components│
│          │                                      │  • Mesh   │
│          │                                      │  • Collider│
│          │                                      ├───────────┤
├──────────┤                                      │ Metadata  │
│  Assets  │                                      │  Name:    │
│          │                                      │  Tags:    │
│ • Cube   │                                      │           │
│ • Sphere │                                      │           │
│ • Plane  │                                      │           │
│ • ...    │                                      │           │
└──────────┴──────────────────────────────────────┴───────────┘
│ Transform: Translate (G) | Grid Snap: ON | Scene: untitled* │ <- Status Bar
└─────────────────────────────────────────────────────────────┘
```

## Success Criteria

### Progress Summary (Week 1)
**Completed**: 4/6 core systems ✅
- ✅ Camera controller with fly-around controls
- ✅ Primitive spawning with placement system
- ✅ Grid display with snapping and status bar
- ✅ Selection system (Bevy's built-in picking - MIGRATED!)
- ✅ Inspector panel (basic implementation, ready for expansion)
- ❌ Play mode entry/exit (not started)

**Status**: On track for Week 1 core editing. Play mode is the only remaining critical feature.

### Core Functionality
- [x] Can launch editor with `--editor` flag
- [x] Can fly around scene smoothly with editor camera
- [x] Grid display shows spatial reference
- [x] Can toggle grid snapping with G key (0.5m position, 15° rotation)
- [ ] Can toggle chunk boundary visualization (B key)
- [x] Can spawn primitives (cube, sphere, plane, cylinder, capsule) into scene
- [x] Can select objects by clicking in viewport (single object)
- [ ] **Can press P to enter play mode and test level (week 1 priority)**
- [ ] **Can press ESC in play mode to return to editor (week 1 priority)**
- [ ] Can move selected objects with translate gizmo (F to cycle modes)
- [ ] Can rotate selected objects with rotate gizmo (snaps when grid enabled)
- [ ] Can scale selected objects with scale gizmo
- [ ] Can see object properties in inspector panel
- [ ] Can edit transform values numerically in inspector
- [ ] Can save scene to YAML file (Ctrl+S)
- [ ] Can load saved scene from YAML (Ctrl+O)
- [ ] Scene includes chunk metadata (position, bounds, faction)
- [ ] Can group objects (Ctrl+G) and ungroup (Ctrl+Shift+G)
- [ ] Can multi-select with Ctrl+click (after grouping implemented)
- [ ] Can box-select multiple objects (after grouping implemented)
- [ ] Can duplicate objects with Ctrl+D (offset +1m on X-axis)
- [ ] Can delete objects with Del key

### Quality Checks
- [ ] Gizmo interactions feel responsive and accurate (not yet implemented)
- [x] Selection is unambiguous (clear visual feedback with yellow outline)
- [x] **Grid snapping feels natural and predictable (visual + functional from week 1)**
- [ ] **Play mode works by end of week 1 (tight iteration loops)** - NOT STARTED
- [ ] Scene YAML files are human-readable and version-control friendly (not yet implemented)
- [ ] No crashes when switching between editor and play mode (not yet testable)
- [x] Camera movement feels smooth and controllable
- [ ] Chunk bounds visualization helps with spatial awareness (32m reference) - not yet implemented
- [ ] Duplicate offset (+1m X) is consistent and predictable (not yet implemented)
- [x] Multi-select only available after grouping is implemented (correctly deferred)

## Out of Scope (Future Iterations)

**Defer to Later**:
- **Undo/Redo system** (Iteration 2 - needs solid foundation first, half-working undo is worse than none)
- Importing custom 3D models
- Material editor
- Lighting tools
- Prefab system
- Advanced snapping (edge, vertex, surface)
- Terrain tools
- Particle systems
- Audio preview
- Multi-user collaboration
- Plugin system

Keep it simple. This iteration is about proving we can build and test scenes with basic geometry.

---

## Implementation Priority

### Immediate Next Steps (Completing Week 1)

**Priority 1: Play Mode Entry/Exit** 🔴 CRITICAL
- This is the highest-value feature remaining in Week 1
- Enables tight iteration loops for level testing
- Required: EditorState enum, state transitions, player spawning
- Estimated effort: 4-6 hours

**Priority 2: Improve Selection Accuracy** 🟡 IMPORTANT
- Current sphere approximation works but could be better
- Consider implementing proper AABB or mesh bounds intersection
- Estimated effort: 2-3 hours

**Priority 3: Complete Inspector Panel** 🟢 NICE-TO-HAVE
- Add rotation and scale display (read-only)
- Show mesh type and material color
- Estimated effort: 1-2 hours

**Deferred to Week 2**:
- Transform gizmos (translate, rotate, scale)
- Editable inspector fields
- Chunk boundary visualization

### Week 1: Core Editing + Early Testing

#### 1. Editor camera controller (fly-around) ✅ COMPLETE
- [x] Create `EditorCamera` component with velocity and rotation state
- [x] Implement WASD movement input handling
- [x] Add mouse look with configurable sensitivity (default 0.002)
- [x] Add Q (down) and E (up) vertical movement
- [x] Implement Shift speed multiplier (4x)
- [x] Implement Ctrl speed reduction (0.25x)
- [x] Add smooth velocity-based movement with 10.0 smoothing factor
- [x] Add Left Alt toggle for free mouse mode vs locked camera mode
- [x] Test camera movement (works smoothly with test scene)

**Implementation Notes:**
- EditorCamera is a component attached to the same entity as Camera3d (no duplication)
- Velocity interpolation provides natural acceleration/deceleration feel
- Pitch clamped to ±89° to prevent gimbal lock
- Mouse lock state integrated with window cursor grab mode
- Y-axis movement always uses world up (not camera-relative) for predictable altitude control

#### 2. Primitive spawning (cube, sphere, plane) ✅ COMPLETE
- [x] Create `AssetCatalog` resource with primitive definitions
- [x] Implement mesh generation for cube (1x1x1m)
- [x] Implement mesh generation for sphere (1m diameter)
- [x] Implement mesh generation for plane (10x10m)
- [x] Implement mesh generation for cylinder (1m × 2m)
- [x] Implement mesh generation for capsule (0.5m × 2m)
- [x] Add vertex color support to primitive materials
- [x] Create asset browser UI panel (EGUI) - simple list for MVP
- [x] Implement "place mode" state when asset clicked
- [x] Add ghost preview rendering (semi-transparent material)
- [x] Implement ground-plane ray intersection for preview position
- [x] Spawn entity with mesh, material, and transform on click
- [x] Add ESC to cancel placement mode

**Implementation Notes:**
- Placement system fully functional with PlacementState resource tracking active mode
- Ghost preview uses semi-transparent material (alpha 0.5) with AlphaMode::Blend
- Ground-plane intersection uses simple ray-plane math at Y=0
- UI shows "Placement Mode" indicator with instructions when active
- Multiple placements allowed - user must press ESC to exit mode
- Mouse must be unlocked (Alt toggle) for placement to work
- Preview entity cleaned up when mode cancelled or new placement started

#### 3. Grid display with snapping (visual reference before placing objects) ✅ COMPLETE
- [x] Create grid rendering system (lines on XZ plane)
- [x] Add configurable grid size (default 0.5m spacing)
- [x] Implement grid line shader (fade with distance)
- [x] Add G key toggle for snap mode (persistent state resource)
- [x] Implement position snapping (0.5m increments)
- [x] Implement rotation snapping (15° increments)
- [x] Add visual indicator when snap is enabled (status bar text)

**Implementation Notes:**
- Grid uses LineList primitive topology for efficient rendering
- Grid lines rendered at Y=0.01 to avoid z-fighting with ground plane
- GridConfig resource stores visibility, snap state, spacing (0.5m), and size (50m)
- snap_to_grid() and snap_rotation() helper functions for reuse across systems
- Grid integrated into placement system - preview snaps when grid_config.snap_enabled
- Custom GridMaterial with WGSL fragment shader for distance-based fading:
  - Fades from full opacity at 10m to fully transparent at 50m
  - Uses smoothstep for smooth transition
  - Requires View binding for camera position
  - Registered via MaterialPlugin in EditorPlugin
- Status bar UI shows grid snap state with color-coded indicator (green=ON, gray=OFF)
- Status bar implemented as EGUI TopBottomPanel at screen bottom

####  4. Click selection system (single object only) ✅ COMPLETE
- [x] Migrate to Bevy's built-in picking system (upstreamed in 0.15+)
- [x] Add `Selected` component for selection state
- [x] Implement click-to-select logic via Pointer<Click> observer
- [x] Add outline rendering for selected objects (Bevy's Outline component)
- [x] Implement ESC key to deselect
- [x] Ensure selection persists across frames
- [x] Only select EditorEntity objects (filter in observer)
- [ ] Add visual feedback on hover (subtle highlight - optional for MVP)

**Implementation Notes:**
- **MIGRATED**: Now using Bevy's built-in picking system (`bevy::picking`)
- Uses `MeshPickingPlugin` for accurate mesh raycasting with BVH acceleration
- Observer pattern with `Trigger<Pointer<Click>>` for clean event handling
- Entities marked with `Pickable::default()` are selectable
- Ground plane marked with `Pickable::IGNORE` to prevent selection
- Selected component marker + SelectedEntity resource tracks current selection
- Bevy's built-in Outline component provides yellow outline (color: rgb(1.0, 0.8, 0.0), 3px width)
- Selection filtered to EditorEntity objects only
- ESC deselects only when not in placement mode (placement takes priority)
- Inspector panel displays selected entity's transform (read-only)
- Selection state properly cleaned up when switching between entities
- Proper mesh intersection - no more sphere approximation!

#### 5. Basic inspector panel (read-only transforms) ✅ COMPLETE
- [x] Create inspector EGUI panel on right side
- [x] Display selected entity name (or "No selection")
- [x] Display transform position (X, Y, Z) read-only
- [x] Show "No selection" message when nothing selected
- [x] Update panel in real-time as selection changes

**Implementation Notes:**
- EGUI window positioned at [1200.0, 100.0] with 200px default width
- Shows entity Name component if present, otherwise displays entity debug ID
- Transform displays position only (rotation/scale deferred to editable version)
- Updates every frame by querying SelectedEntity resource
- Clean separation between read and write - no accidental modifications yet

#### 6. **Play mode entry/exit (P key) - critical for iteration loops**
Deferred, will revisit with a better defined game mode. Current solution is a placeholder that cannot be dropped into the editor.

- [ ] Create `EditorState` enum (Editor, EditorPlayMode)
- [ ] Add P key binding to enter play mode
- [ ] Serialize current scene state before entering play mode
- [ ] Spawn player entity at origin (or spawn point if exists)
- [ ] Hide all editor UI (panels, gizmos, grid)
- [ ] Enable game systems (physics, player controller, etc.)
- [ ] Add ESC key binding to exit play mode
- [ ] Restore editor state on exit (camera position, selection)
- [ ] Deserialize scene state to revert changes
- [ ] Add visual indicator in UI showing current mode

---

### Week 2: Transform Tools

#### 7. Translate gizmo with drag interaction (respects grid snapping)
- [ ] Create gizmo rendering system (always on top)
- [ ] Render X-axis arrow (red) at selected object position
- [ ] Render Y-axis arrow (green) at selected object position
- [ ] Render Z-axis arrow (blue) at selected object position
- [ ] Implement ray-cast intersection with gizmo handles
- [ ] Add hover highlighting for gizmo handles
- [ ] Implement click-and-drag logic for handles
- [ ] Constrain movement to selected axis only
- [ ] Apply grid snapping during drag (if enabled)
- [ ] Update object transform in real-time during drag
- [ ] Release on mouse-up to finalize transform
- [ ] Add visual feedback showing drag axis constraint

#### 8. Rotate gizmo with drag interaction (15° snap when enabled)
- [ ] Switch gizmo to rotation mode with F key
- [ ] Render X-axis rotation arc (red circle around X)
- [ ] Render Y-axis rotation arc (green circle around Y)
- [ ] Render Z-axis rotation arc (blue circle around Z)
- [ ] Implement arc handle intersection testing
- [ ] Add hover highlighting for rotation handles
- [ ] Implement circular drag logic (convert mouse delta to angle)
- [ ] Constrain rotation to selected axis only
- [ ] Apply 15° snapping during drag (if grid snap enabled)
- [ ] Update object rotation in real-time during drag
- [ ] Display angle value during rotation (transient UI)

#### 9. Scale gizmo with drag interaction
- [ ] Switch gizmo to scale mode with F key
- [ ] Render X-axis scale handle (red cube)
- [ ] Render Y-axis scale handle (green cube)
- [ ] Render Z-axis scale handle (blue cube)
- [ ] Add center handle for uniform scaling (white/gray)
- [ ] Implement handle intersection testing
- [ ] Add hover highlighting for scale handles
- [ ] Implement drag-to-scale logic (mouse delta → scale factor)
- [ ] Constrain scaling to selected axis (or uniform for center)
- [ ] Update object scale in real-time during drag
- [ ] Prevent negative or zero scale values
- [ ] Add Shift+F to cycle gizmo modes in reverse

#### 10. Inspector with editable numeric fields
- [ ] Convert transform fields from read-only to editable
- [ ] Add text input for position X, Y, Z
- [ ] Add text input for rotation X, Y, Z (Euler angles)
- [ ] Add text input for scale X, Y, Z
- [ ] Validate numeric input (reject non-numbers)
- [ ] Apply changes on Enter key or focus loss
- [ ] Add increment/decrement buttons (+/- steppers)
- [ ] Support precision to 3 decimal places
- [ ] Update viewport in real-time as values change

#### 11. Chunk boundary visualization (B key toggle)
- [ ] Create chunk bounds rendering system
- [ ] Render wireframe box for current chunk (32x32x32m)
- [ ] Use distinct color (e.g., cyan/magenta) for chunk bounds
- [ ] Add B key toggle for visibility
- [ ] Display chunk position label (world coordinates)
- [ ] Show chunk size in meters
- [ ] Optionally render adjacent chunk outlines (faded)
- [ ] Add status bar indicator when chunk viz is enabled

---

### Week 3: Scene Management + Multi-Object

#### 12. Scene serialization to YAML (save)
- [ ] Create `SceneData` serializable struct (serde)
- [ ] Add chunk metadata fields (id, position, bounds, faction)
- [ ] Implement entity serialization (name, transform, components)
- [ ] Serialize mesh references (not embedded geometry)
- [ ] Serialize material references
- [ ] Add editor metadata (camera position, last modified)
- [ ] Implement Ctrl+S keybinding for save
- [ ] Show file picker dialog if no current file
- [ ] Write YAML to file using `serde_yaml`
- [ ] Add error handling for file write failures
- [ ] Clear scene dirty flag after successful save
- [ ] Show confirmation message on successful save

#### 13. Scene deserialization from YAML (load)
- [ ] Implement Ctrl+O keybinding for open
- [ ] Show file picker dialog (filter for .yaml)
- [ ] Prompt to save if current scene is dirty
- [ ] Read YAML file using `serde_yaml`
- [ ] Validate scene format and version
- [ ] Clear existing scene entities
- [ ] Deserialize chunk metadata
- [ ] Spawn entities from scene data
- [ ] Restore transforms, meshes, materials
- [ ] Restore editor camera position if saved
- [ ] Add error handling for malformed YAML
- [ ] Show confirmation message on successful load

#### 14. Chunk metadata in scene files (position, bounds, faction)
- [ ] Create `ChunkMetadata` component
- [ ] Add chunk ID field (string, e.g., "mining_shaft_7")
- [ ] Add world position field (Vec3, Y = depth)
- [ ] Add bounds field (Vec3, default [32, 32, 32])
- [ ] Add optional faction field (string)
- [ ] Display chunk metadata in inspector (separate section)
- [ ] Make chunk metadata editable in inspector
- [ ] Include chunk metadata in YAML serialization
- [ ] Restore chunk metadata on scene load

#### 15. Group/ungroup operations (Ctrl+G / Ctrl+Shift+G)
- [ ] Add Ctrl+G keybinding for group
- [ ] Create parent entity when grouping selected objects
- [ ] Move selected entities to be children of parent
- [ ] Update transform hierarchy (local → world conversion)
- [ ] Name parent entity "Group" with auto-incrementing number
- [ ] Add Ctrl+Shift+G keybinding for ungroup
- [ ] Flatten hierarchy level (promote children to root)
- [ ] Convert local transforms to world transforms on ungroup
- [ ] Delete empty parent entity after ungroup
- [ ] Update hierarchy panel to reflect changes

#### 16. Multi-select (Ctrl+click) and box select (now that grouping exists)
- [ ] Add `SelectionSet` resource to track multiple selected entities
- [ ] Implement Ctrl+click to add/remove from selection
- [ ] Highlight all selected objects with outline
- [ ] Update gizmo to show at center of selection bounds
- [ ] Implement box select drag (click-drag in empty space)
- [ ] Render selection box rectangle during drag
- [ ] Select all objects intersecting box on release
- [ ] Transform gizmo applies to all selected objects
- [ ] Inspector shows multi-select summary (count, bounds)
- [ ] Duplicate/delete operations work on selection set

#### 17. Duplicate/delete operations (with defined +1m X-axis offset)
- [ ] Add Ctrl+D keybinding for duplicate
- [ ] Clone all components of selected entities
- [ ] Offset duplicates by +1m on X-axis (or dominant horizontal)
- [ ] Maintain parent-child relationships in duplicates
- [ ] Auto-select duplicated objects after creation
- [ ] Add Del keybinding for delete
- [ ] Remove selected entities from scene
- [ ] Clean up orphaned children on parent delete
- [ ] Show confirmation dialog for delete if preferred
- [ ] Clear selection after delete

#### 18. Hierarchy panel
- [ ] Create hierarchy EGUI panel on left side
- [ ] Display tree view of all scene entities
- [ ] Show parent-child relationships with indentation
- [ ] Add expand/collapse arrows for parent entities
- [ ] Display entity name (editable inline)
- [ ] Add show/hide toggle button per entity
- [ ] Add lock/unlock toggle button per entity
- [ ] Implement click-to-select in hierarchy
- [ ] Sync selection between hierarchy and viewport
- [ ] Add drag-and-drop to reparent entities (optional)

---

### Week 4: Polish & Stability

#### 19. Player spawn point designation
- [ ] Create `PlayerSpawnPoint` component marker
- [ ] Add "Set as Spawn Point" button in inspector
- [ ] Render spawn point icon in viewport (distinct gizmo)
- [ ] Ensure only one spawn point exists (remove others)
- [ ] Save spawn point to YAML scene file
- [ ] Use spawn point position when entering play mode
- [ ] Fall back to origin if no spawn point exists
- [ ] Show spawn point rotation as forward direction arrow

#### 20. Scene dirty flag and auto-save (every 5 minutes)
- [ ] Add `SceneDirty` resource (bool flag)
- [ ] Set dirty flag on any scene modification
- [ ] Show asterisk in title bar when dirty
- [ ] Implement Ctrl+N keybinding for new scene
- [ ] Prompt to save dirty scene on new/open/exit
- [ ] Implement auto-save timer (5 minutes)
- [ ] Save to temp file on auto-save (e.g., `.autosave`)
- [ ] Show notification on auto-save
- [ ] Recover from auto-save on crash/reload
- [ ] Clear dirty flag after save

#### 21. Polish gizmo visuals and interactions
- [ ] Smooth out gizmo handle highlighting
- [ ] Add subtle animation to gizmo on selection
- [ ] Improve handle size scaling based on camera distance
- [ ] Add anti-aliasing to gizmo lines
- [ ] Polish color scheme for accessibility
- [ ] Add haptic feedback cues (visual pulse on snap)
- [ ] Ensure gizmo renders on top of all geometry
- [ ] Add fade-in/fade-out transitions

#### 22. Bug fixes and edge cases
- [ ] Test scene load with missing assets
- [ ] Test save/load with complex hierarchies
- [ ] Test multi-select with grouped objects
- [ ] Test gizmo interaction at extreme scales
- [ ] Test camera collision with scene bounds
- [ ] Test play mode with no spawn point
- [ ] Test rapid mode switching (editor ↔ play)
- [ ] Test grid snapping at chunk boundaries
- [ ] Fix any reported crashes or data loss

#### 23. Keyboard shortcut refinements
- [ ] Add shortcut reference panel (accessible via F1 or ?)
- [ ] Document all keybindings in UI
- [ ] Ensure no conflicting keybindings
- [ ] Add customizable keybinding config (optional)
- [ ] Test shortcuts on different keyboard layouts
- [ ] Add visual feedback when shortcut is pressed
- [ ] Consider adding toolbar buttons for key actions

---

## Dependencies & Related Documentation

### Foundation Documents
- `foundational/mechanics.md` - Game loop and survival systems
- `foundational/setting.md` - World context (10km deep colony, factions)
- `foundational/timeline.md` - Historical context (Year 165, post-D+13)

### Technical Architecture (Uncommitted)
- `uncommitted/world_architecture.md` - Chunk streaming system (32m chunks, LOD, floating origin)
- `uncommitted/persistence.md` - Two-database system (static/dynamic split, YAML → SQLite)
- `uncommitted/procedural_generation.md` - Wave Function Collapse, depth zones, faction territory
- `uncommitted/art_pipeline.md` - Low-poly aesthetic, vertex colors, 100-500 poly budget
- `uncommitted/editor.md` - Full editor vision (this iteration is Phase 1)

## Technical Risks & Mitigations

**Risk**: Mouse lock conflicts between camera control and UI interaction
- **Status**: ✅ RESOLVED - Left Alt toggle provides clean separation
- **Solution**: `mouse_locked` state on EditorCamera + window cursor grab mode integration

**Risk**: Camera movement feels floaty or unresponsive
- **Status**: ✅ RESOLVED - Velocity interpolation with 10.0 smoothing factor feels natural
- **Solution**: `velocity.lerp()` with time.delta provides smooth acceleration without lag

**Risk**: Pitch rotation causes gimbal lock or camera flipping
- **Status**: ✅ RESOLVED - Pitch clamped to ±89°
- **Solution**: `pitch.clamp(-FRAC_PI_2 + 0.01, FRAC_PI_2 - 0.01)` prevents singularity

**Risk**: Y-axis movement feels confusing relative to camera orientation
- **Status**: ✅ RESOLVED - Always use world Y (up/down), not camera relative
- **Solution**: Separate world-space up vector for Q/E keys

**Risk**: Gizmo interaction feels clunky or imprecise
- **Mitigation**: Start with translate-only, iterate on feel before adding rotate/scale

**Risk**: Building for 3 weeks before testing in-game is painful
- **Mitigation**: ✅ FIXED - Play mode (P key) moved to week 1 for tight iteration loops

**Risk**: Grid snapping without visible grid is confusing
- **Mitigation**: ✅ FIXED - Grid display and snapping implemented together in week 1

**Risk**: Multi-select without grouping creates orphaned selection states
- **Mitigation**: ✅ FIXED - Multi-select deferred until week 3 when grouping is implemented

**Risk**: "Slight offset" on duplicate causes inconsistent behavior
- **Mitigation**: ✅ FIXED - Defined as +1m on X-axis (or dominant horizontal for rotated groups)

**Risk**: Undo/redo across 3 weeks of features becomes fragile
- **Mitigation**: ✅ FIXED - Undo/redo deferred to Iteration 2 entirely (foundation must be solid)

**Risk**: Scene serialization breaks with component changes
- **Mitigation**: Use robust serialization format, version scene files

**Risk**: Editor ↔ Game mode transition causes state corruption
- **Mitigation**: Clear separation of editor vs game entities, snapshot before transition

## Technical Decisions Log

### Lessons Learned & Best Practices

#### What's Working Well
1. **Velocity-based camera movement**: Feels natural and responsive without being twitchy
2. **Resource pattern for editor state**: GridConfig, PlacementState, SelectedEntity are clean and queryable
3. **Component markers**: EditorEntity, Selected, PreviewEntity make entity filtering trivial
4. **EGUI for rapid prototyping**: Fast to iterate on UI without fighting with styling
5. **Continuous placement mode**: Users can place multiple objects efficiently
6. **Middle-mouse temporary lock**: Blender-like workflow without disrupting mouse-unlocked state

#### Areas for Improvement
1. **Hover feedback**: No visual hint before clicking
   - Bevy picking provides `Pointer<Over>` events out of the box
   - Could add subtle highlight on hover easily with observer
   - Low priority - outline on selection is sufficient for MVP
2. **Inspector completeness**: Only shows position, missing rotation/scale
   - Deferred to editable inspector (Week 2)
   - Consider adding mesh type and material color for context
3. **Grid shader requires custom WGSL**: Had to learn material system
   - Worth it for distance fade effect
   - Could simplify with solid-color lines if performance becomes issue
4. **Outline component verification needed**: Used in code but not thoroughly tested
   - Appears to work with Bevy 0.16
   - May need fallback if issues arise

#### System Architecture Observations
1. **Startup vs Update separation works cleanly**:
   - setup_* functions run once
   - update_* functions are reactive
   - Clear mental model
2. **Event-driven vs polling**: Currently using keyboard polling
   - Works fine for editor controls
   - Might want event-driven for undo/redo history
3. **Module organization scales well**:
   - Each editor subsystem in its own file
   - mod.rs as coordinator is clean
   - Easy to find and modify specific features

### Camera Controller (Week 1)
1. **EditorCamera as augmentation component**: Attach to same entity as Camera3d rather than replacing it - avoids duplication and leverages Bevy's built-in camera features
2. **Separate mouse lock state from cursor grab**: Easier to reason about when state lives on component vs system-level resource
3. **World-space vertical movement**: Q/E always move on world Y-axis, not camera-relative, for predictable altitude control
4. **10.0 smoothing factor**: Tested multiple values; this provides good balance between responsiveness and smoothness
5. **Bevy 0.16 API**: Using `single_mut()` instead of deprecated `get_single_mut()`
6. **Middle mouse button temporary lock**: Holding MMB temporarily re-locks cursor for camera control even when unlocked - provides Blender-like workflow

### Primitive Spawning (Week 1)
1. **All 5 primitives implemented upfront**: Cube, sphere, plane, cylinder, capsule - minimal extra work to do all at once
2. **Vertex colors on all meshes**: Added to support low-poly art style, defaults to white (can be painted per-vertex later)
3. **Placement state as resource**: Simpler than per-entity component, tracks single active placement mode
4. **Continuous placement mode**: User can place multiple objects without re-clicking asset - more efficient workflow
5. **Semi-transparent preview**: Alpha 0.5 with AlphaMode::Blend gives clear visual feedback without obscuring scene

### Grid System (Week 1)
1. **Custom line rendering**: Avoided external dependencies, simple LineList topology with manual mesh generation
2. **Grid always visible by default**: Helps spatial awareness from first run
3. **Snap as toggle not hold**: G key toggles persistent snap state - less fatiguing than holding a key
4. **0.5m spacing**: Finer than typical 1m grids, better for detailed level design at colony scale
5. **Grid at Y=0.01**: Slight offset prevents z-fighting with ground plane entities
6. **Custom WGSL shader for distance fade**: Learned Bevy's material system and shader pipeline:
   - AsBindGroup derives automatic uniform binding for material properties
   - Fragment shader imports View for camera position
   - smoothstep provides smooth fade from 10m to 50m
   - MaterialPlugin registers custom material with render pipeline
7. **Status bar for state feedback**: EGUI TopBottomPanel provides persistent visual indicator at screen bottom

### Selection System (Week 1)
1. **Bevy's built-in picking system**: Migrated from manual raycasting ✅
   - **COMPLETED**: Successfully migrated to `bevy::picking` module
   - `MeshPickingPlugin` provides accurate mesh raycasting with BVH
   - Observer pattern with `Trigger<Pointer<Click>>` for event handling
   - Proper mesh intersection - no more sphere approximation
   - Performance benefits from spatial acceleration structures
2. **Observer pattern**: Clean event-driven architecture
   - `app.add_observer(handle_selection)` registers global observer
   - `Trigger<Pointer<Click>>` provides target entity and event data
   - More idiomatic than polling EventReader in system
3. **Pickable component**: Declarative selection control
   - `Pickable::default()` makes entities selectable
   - `Pickable::IGNORE` excludes entities (e.g., ground plane)
   - No need for separate Selectable marker component
4. **Resource + Component pattern**: SelectedEntity resource + Selected marker component
   - Resource makes it easy to query "what's selected" globally
   - Component enables efficient iteration over selected entities for rendering
5. **Bevy's built-in Outline**: No need for custom post-process or mesh duplication
   - Simple component insertion/removal
   - Consistent with Bevy's architecture
   - Works immediately without shader complexity
6. **Context-aware selection**: Only selects EditorEntity objects
   - Filter in observer checks EditorEntity component
   - Placement mode prevents selection (checked in observer)
7. **ESC key priority**: Placement cancellation > deselection
   - User's most likely intent when pressing ESC in placement mode

### UI Architecture (Week 1)
1. **EGUI for all UI panels**: Consistent framework for editor interface
   - Asset browser (left side) - simple button list
   - Inspector (right side) - property display
   - Status bar (bottom) - persistent state indicators
2. **Separation of concerns**: Three independent UI functions in ui.rs
   - asset_browser_ui: Spawning and placement
   - inspector_ui: Selection properties
   - status_bar_ui: Global editor state
3. **All UI runs in EguiPrimaryContextPass**: Proper system ordering
   - Ensures EGUI context is ready
   - Prevents flickering or state issues
4. **Placement mode feedback**: Clear visual indicators when in placement state
   - Yellow text in asset browser
   - Instructions displayed
   - Preview ghost in viewport
5. **Color-coded status indicators**: Intuitive state communication
   - Green = enabled/active (snap ON)
   - Gray = disabled (snap OFF)
   - Yellow = warning/special mode (placement active)

## References
- Blender's transform gizmo system (industry standard UX)
- Unity's scene editor (inspector panel layout)
- Unreal's level editor (play-in-editor workflow)
- Godot's editor (EGUI panel approach for Rust ecosystem)

## What's Next: Iteration 2

With the editor MVP proving we can hand-craft scenes with primitives, Iteration 2 will focus on importing custom 3D models (GLTF/GLB), building a prefab system for reusable compound objects, and implementing advanced snapping (edge-to-edge, vertex, surface alignment) to enable efficient modular construction - essentially evolving from "place cubes" to "assemble complex environments from custom assets and prefabs" while introducing a proper asset pipeline and material editor for artistic control over the colony's visual atmosphere.
