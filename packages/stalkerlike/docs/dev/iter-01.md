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
- Spawn player at origin
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

### Progress Summary (Week 2)
**Completed**: 8/9 core systems ✅
- ✅ Camera controller with fly-around controls
- ✅ Primitive spawning with placement system
- ✅ Grid display with snapping and status bar
- ✅ Selection system (Bevy's built-in picking - MIGRATED!)
- ✅ Inspector panel (basic implementation, ready for expansion)
- ✅ Transform gizmos (translate, rotate, scale with RGB spheres)
- ✅ Editable inspector fields (with steppers and validation!)
- ❌ Play mode entry/exit (not started)

**Status**: Week 2 essentially complete! Only play mode remains, which is deferred pending better game mode integration.

### Core Functionality
- [x] Can launch editor with `--editor` flag
- [x] Can fly around scene smoothly with editor camera
- [x] Grid display shows spatial reference
- [x] Can toggle grid snapping with G key (0.5m position, 15° rotation)
- [x] Can spawn primitives (cube, sphere, plane, cylinder, capsule) into scene
- [x] Can select objects by clicking in viewport (single object)
- [ ] **Can press P to enter play mode and test level (deferred)**
- [ ] **Can press ESC in play mode to return to editor (deferred)**
- [x] Can move selected objects with translate gizmo (F to cycle modes)
- [x] Can rotate selected objects with rotate gizmo (snaps when grid enabled)
- [x] Can scale selected objects with scale gizmo
- [x] Can see object properties in inspector panel
- [x] Can edit transform values numerically in inspector
- [x] Can save scene to YAML file (Ctrl+S)
- [x] Can load saved scene from YAML (Ctrl+O)
- [ ] Can group objects (Ctrl+G) and ungroup (Ctrl+Shift+G)
- [ ] Can multi-select with Ctrl+click (after grouping implemented)
- [ ] Can box-select multiple objects (after grouping implemented)
- [ ] Can duplicate objects with Ctrl+D (offset +1m on X-axis)
- [x] Can delete objects with Del key

### Quality Checks
- [x] Gizmo interactions feel responsive and accurate (RGB sphere handles with distance-based scaling)
- [x] Selection is unambiguous (clear visual feedback with yellow outline)
- [x] **Grid snapping feels natural and predictable (visual + functional, works with gizmos)**
- [ ] **Play mode works by end of week 1 (tight iteration loops)** - DEFERRED
- [ ] No crashes when switching between editor and play mode (not yet testable)
- [x] Camera movement feels smooth and controllable
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

**Deferred to Week 3**:
- Editable inspector fields (numeric transform input)
- Play mode entry/exit (P key) - needs better game mode integration
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
- [ ] Spawn player entity at origin
- [ ] Hide all editor UI (panels, gizmos, grid)
- [ ] Enable game systems (physics, player controller, etc.)
- [ ] Add ESC key binding to exit play mode
- [ ] Restore editor state on exit (camera position, selection)
- [ ] Deserialize scene state to revert changes
- [ ] Add visual indicator in UI showing current mode

---

### Week 2: Transform Tools

#### 7. Translate gizmo with drag interaction (respects grid snapping) ✅ COMPLETE
- [x] Create gizmo rendering system (always on top)
- [x] Render X-axis arrow (red arrow with cylinder + cone)
- [x] Render Y-axis arrow (green arrow with cylinder + cone)
- [x] Render Z-axis arrow (blue arrow with cylinder + cone)
- [x] Add thin axis lines connecting center to handles
- [x] Implement constant screen-space sizing (scale with camera distance)
- [x] Implement ray-cast intersection with gizmo handles (using Bevy's picking)
- [x] Add hover highlighting for gizmo handles (emissive material brightening)
- [x] Implement click-and-drag logic for handles
- [x] Constrain movement to selected axis only
- [x] Apply grid snapping during drag (if enabled)
- [x] Update object transform in real-time during drag
- [x] Release on mouse-up to finalize transform
- [x] Add Local/Global orientation toggle (O key)
- [x] Implement speed modifiers (Shift = 4x, Ctrl = 0.25x)

**Implementation Notes:**
- Arrow-style gizmos like Blender: cylinder shaft (0.02 radius, 1.0 length) + cone tip (0.05 radius, 0.2 height)
- Thin axis lines (0.01 radius cylinders) connect center to handles for better spatial reference
- Color scheme: X=red, Y=green, Z=blue (standard 3D convention)
- Constant screen-space sizing: scale = camera_distance * 0.15
- Custom GizmoMaterial with disabled depth testing (CompareFunction::Always) - always renders on top
- Material uses individual f32 fields (color_r, color_g, color_b, color_a, emissive_r/g/b/a) for WGSL buffer alignment
- Built on Bevy's picking system with Pickable component
- Observer pattern: on_gizmo_drag fires continuously during drag
- Distance-based drag scaling (0.001 * camera distance) for consistent feel across distances
- Vertical mouse movement controls Y and Z axes (intuitive up/down)
- Horizontal mouse movement controls X axis
- Grid snapping respects GridConfig.snap_enabled (0.5m spacing)
- Hover highlighting: emissive increases to (2.0, 2.0, 2.0) white, restored to axis color on hover end
- Local/Global orientation (O key toggle):
  - Global: gizmos align with world axes (Quat::IDENTITY)
  - Local: gizmos rotate with object (uses object's rotation)
  - Transforms applied in correct space (world for Global, rotated by object for Local)
- Speed modifiers applied to drag_scale:
  - Normal: 1.0x (base_drag_scale = 0.001)
  - Shift: 4.0x faster
  - Ctrl: 0.25x slower (precision mode)
- Material organization: Moved to core/materials.rs for centralized material management

#### 8. Rotate gizmo with drag interaction (15° snap when enabled) ✅ COMPLETE
- [x] Switch gizmo to rotation mode with F key
- [x] Render X-axis rotation handle (red arrow around X)
- [x] Render Y-axis rotation handle (green arrow around Y)
- [x] Render Z-axis rotation handle (blue arrow around Z)
- [x] Implement arc handle intersection testing (reuses arrow picking)
- [x] Add hover highlighting for rotation handles (emissive material)
- [x] Implement circular drag logic (convert mouse delta to angle)
- [x] Constrain rotation to selected axis only
- [x] Apply 15° snapping during drag (if grid snap enabled)
- [x] Update object rotation in real-time during drag
- [x] Display angle value during rotation (shown in inspector - deferred to editable inspector)

**Implementation Notes:**
- Same arrow handles used for all modes (Translate/Rotate/Scale) - unified visual language
- Color scheme: X=red, Y=green, Z=blue (standard 3D convention)
- F key cycles forward through modes: Translate → Rotate → Scale
- Shift+F cycles backward: Scale → Rotate → Translate
- Rotation speed: 0.02 radians per pixel of drag distance
- Drag direction (positive/negative) determines rotation direction
- 15° snap when grid_config.snap_enabled = true (matches spec)
- Uses Euler angles (XYZ order) for rotation editing
- Mode displayed in status bar with light blue color
- Works in both Local and Global orientation modes
- Speed modifiers (Shift/Ctrl) apply to rotation speed

#### 9. Scale gizmo with drag interaction ✅ COMPLETE
- [x] Switch gizmo to scale mode with F key
- [x] Render X-axis scale handle (red arrow)
- [x] Render Y-axis scale handle (green arrow)
- [x] Render Z-axis scale handle (blue arrow)
- [ ] Add center handle for uniform scaling (white/gray) - deferred, single-axis sufficient for MVP
- [x] Implement handle intersection testing (reuses arrow picking)
- [x] Add hover highlighting for scale handles (emissive material)
- [x] Implement drag-to-scale logic (mouse delta → scale factor)
- [x] Constrain scaling to selected axis (or uniform for center - deferred)
- [x] Update object scale in real-time during drag
- [x] Prevent negative or zero scale values (clamped to 0.01 minimum)
- [x] Add Shift+F to cycle gizmo modes in reverse

**Implementation Notes:**
- Scale speed: 0.01 per pixel of vertical mouse drag
- Vertical drag: up = increase scale, down = decrease scale
- Minimum scale: 0.01 to prevent zero/negative values
- Same arrow handles as translate/rotate modes (consistency)
- Color scheme: X=red, Y=green, Z=blue (matches grid and other modes)
- F key cycles forward, Shift+F cycles backward
- No uniform scale handle in MVP - can add later if needed
- Works per-axis only (matches spec for Week 2)
- Speed modifiers (Shift/Ctrl) apply to scale rate

#### 10. Inspector with editable numeric fields ✅ COMPLETE
- [x] Convert transform fields from read-only to editable
- [x] Add text input for position X, Y, Z
- [x] Add text input for rotation X, Y, Z (Euler angles)
- [x] Add text input for scale X, Y, Z
- [x] Validate numeric input (reject non-numbers)
- [x] Apply changes on Enter key or focus loss
- [x] Add increment/decrement buttons (+/- steppers)
- [x] Support precision to 3 decimal places
- [x] Update viewport in real-time as values change

**Implementation Notes:**
- InspectorState resource maintains text buffers for all transform fields
- Buffers update when selection changes or transform modified externally (e.g., gizmo)
- Text fields validate input in real-time with hover tooltip for invalid numbers
- Enter key applies changes immediately
- Stepper buttons: -/+ with configurable step size (0.1 for pos/scale, 5.0° for rotation)
- Rotation displayed in degrees (user-friendly) but stored as radians internally
- Scale clamped to 0.01 minimum to prevent zero/negative values
- Three decimal place precision throughout (format!("{:.3}"))
- Real-time viewport updates via mutable Transform query
- Detection of external changes via transform.is_changed() to keep buffers in sync

---

### Week 3: Scene Management + Multi-Object

#### 12. Scene serialization to YAML (save) ✅ COMPLETE
- [x] Create `SceneData` serializable struct (serde)
- [x] Implement entity serialization (name, transform, components)
- [x] Serialize mesh references (primitive type enum)
- [x] Serialize material references (base color)
- [x] Implement Ctrl+S keybinding for save
- [x] Write YAML to file using `serde_yml`
- [x] Add error handling for file write failures
- [x] Show confirmation message on successful save (console log)
- [x] Add scene dirty flag indicator in status bar
- [x] Implement Ctrl+S keybinding for save
- [x] Show file picker dialog if no current file
- [x] Write YAML to file using `serde_yaml`
- [x] Add error handling for file write failures
- [x] Clear scene dirty flag after successful save
- [x] Show confirmation message on successful save

**Implementation Notes:**
- Created extensible component-based serialization in `editor/persistence/`
- ComponentData enum allows adding new component types in future
- TransformData stores position, rotation (quaternion), and scale
- PrimitiveTypeSerde enum maps to PrimitiveType for serialization
- CurrentFile resource with Option<PathBuf> - defaults to None (unset)
- Status bar shows filename with asterisk (*) when dirty
- Uses serde_yaml for human-readable YAML output
- **File Menu Integration**:
  - Top menu bar with File > New, Open, Save, Save As
  - Save button greyed out when no file is open (visual feedback)
  - Ctrl+S triggers Save As dialog when no file is set (user-friendly)
- **Native File Dialogs**: rfd crate provides OS-native file pickers
  - Open File: Filters for .yaml/.yml files
  - Save As: Save dialog with .yaml/.yml filter
  - Ctrl+O always opens file picker (consistent workflow)
- **Unsaved Changes Protection**:
  - ConfirmationDialog system prompts before New/Open if dirty
  - Three-button dialog: "Save", "Don't Save", "Cancel"
  - Prevents accidental data loss
- **Error Handling**:
  - ErrorDialog resource displays save/load failures
  - Centered modal with error details and OK button
  - Handles directory creation, file I/O, and YAML parsing errors

#### 13. Scene deserialization from YAML (load) ✅ COMPLETE
- [x] Implement Ctrl+O keybinding for open
- [x] Read YAML file using `serde_yml`
- [x] Validate scene format and version
- [x] Clear existing scene entities (EditorEntity filter)
- [x] Spawn entities from scene data
- [x] Restore transforms, meshes, materials
- [x] Add error handling for malformed YAML
- [x] Show confirmation message on successful load (console log)

**Implementation Notes:**
- load_scene() function in persistence/scene.rs
- Clears all EditorEntity-marked entities before loading
- Iterates through SceneData.entities and spawns with correct components
- Primitive type restoration uses PrimitiveType::create_mesh()
- Material base_color restored from YAML
- Native file picker integrated via rfd crate
- Proper error propagation with Result type
- Status bar updates to show loaded filename
- ErrorDialog shows user-friendly error messages on load failure
- Ctrl+O always triggers file picker (no default path assumptions)

#### 14. Group/ungroup operations (Ctrl+G / Ctrl+Shift+G)
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

#### 15. Multi-select (Ctrl+click) and box select (now that grouping exists)
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

#### 16. Duplicate/delete operations (with defined +1m X-axis offset)
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

#### 17. Hierarchy panel
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

#### 18. Scene dirty flag and auto-save (every 5 minutes)
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

#### 19. Polish gizmo visuals and interactions
- [ ] Smooth out gizmo handle highlighting
- [ ] Add subtle animation to gizmo on selection
- [ ] Improve handle size scaling based on camera distance
- [ ] Add anti-aliasing to gizmo lines
- [ ] Polish color scheme for accessibility
- [ ] Add haptic feedback cues (visual pulse on snap)
- [ ] Ensure gizmo renders on top of all geometry
- [ ] Add fade-in/fade-out transitions

#### 20. Bug fixes and edge cases
- [ ] Test scene load with missing assets
- [ ] Test save/load with complex hierarchies
- [ ] Test multi-select with grouped objects
- [ ] Test gizmo interaction at extreme scales
- [ ] Test camera collision with scene bounds
- [ ] Test rapid mode switching (editor ↔ play)
- [ ] Test grid snapping at chunk boundaries
- [ ] Fix any reported crashes or data loss

#### 21. Keyboard shortcut refinements
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
2. **Resource pattern for editor state**: GridConfig, PlacementState, SelectedEntity, CurrentFile are clean and queryable
3. **Component markers**: EditorEntity, Selected, PreviewEntity make entity filtering trivial
4. **EGUI for rapid prototyping**: Fast to iterate on UI without fighting with styling
5. **Continuous placement mode**: Users can place multiple objects efficiently
6. **Middle-mouse temporary lock**: Blender-like workflow without disrupting mouse-unlocked state
7. **Native file dialogs (rfd)**: OS-native look provides familiar UX without custom UI work
8. **Modal dialog pattern**: ConfirmationDialog and ErrorDialog provide clear user feedback
9. **Option<PathBuf> for file state**: Makes "no file" explicit vs implicit default path
10. **Extensible serialization**: ComponentData enum makes adding new component types straightforward

#### Areas for Improvement
1. **Hover feedback**: No visual hint before clicking
   - Bevy picking provides `Pointer<Over>` events out of the box
   - Could add subtle highlight on hover easily with observer
   - Low priority - outline on selection is sufficient for MVP
2. **No undo/redo yet**: Deferred to Iteration 2
   - Foundation must be solid before adding history system
   - Current workflow: test frequently, save often
   - Half-working undo is worse than none
3. **Single file format only**: Only YAML supported
   - Future: Binary format for production builds
   - Current YAML is human-readable and version-control friendly
4. **No scene versioning**: Format changes could break old files
   - Future: Add version field to SceneData
   - Migration system for backward compatibility
5. **Limited component support**: Only Transform, Mesh3d, MeshMaterial3d
   - Extensible design makes adding more straightforward
   - Physics components, lights, etc. coming in future iterations

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

### Transform Gizmos (Week 2)
1. **Arrow-style gizmos over spheres**: Blender-like visual language
   - Cylinder shafts (0.02 radius, 1.0 length) + cone tips (0.05 radius, 0.2 height)
   - More intuitive axis direction than simple spheres
   - Thin axis lines (0.01 radius) connect center to handles
2. **Constant screen-space sizing**: `scale = camera_distance * 0.15`
   - Gizmos maintain consistent apparent size regardless of distance
   - Prevents tiny gizmos at long distances or giant gizmos up close
   - 0.15 factor tuned for comfortable visual size
3. **Custom GizmoMaterial for always-on-top rendering**:
   - Uses `Material::specialize()` to override depth pipeline
   - `CompareFunction::Always` disables depth testing
   - `depth_write_enabled = false` prevents depth buffer writes
   - Gizmos always visible even when behind other geometry
4. **WGSL buffer alignment with individual f32 fields**:
   - Initially used vec4 for color and emissive
   - Hit shader validation error: "Buffer size 32 > min_binding_size 16"
   - Solution: Break into 8 individual f32 fields (color_r/g/b/a, emissive_r/g/b/a)
   - WGSL fragment shader constructs vec4s from individual floats
   - Helper function `GizmoMaterial::new(color, emissive)` unpacks LinearRgba
5. **Local/Global orientation toggle**:
   - O key switches between world-aligned (Global) and object-aligned (Local)
   - Global: gizmo rotation = Quat::IDENTITY
   - Local: gizmo rotation = selected object's rotation
   - Transforms applied in correct coordinate space
6. **Speed modifiers for precision control**:
   - Base drag scale reduced from 0.002 to 0.001 (slower default)
   - Ctrl: 0.25x speed (fine adjustments)
   - Shift: 4.0x speed (quick movements)
   - Applies to all transform modes (translate, rotate, scale)
7. **Unified handle system**: Same arrow geometry for all modes
   - Translate, Rotate, and Scale all use same visual handles
   - F key cycles forward, Shift+F cycles backward
   - Consistent color scheme: X=red, Y=green, Z=blue
8. **Observer-based interaction**: Clean event-driven architecture
   - `on_gizmo_hover`: Increases emissive to (2.0, 2.0, 2.0) white
   - `on_gizmo_hover_end`: Restores axis-specific emissive color
   - `on_gizmo_drag`: Updates transform continuously during drag
   - Each handle entity has its own observers (entity-specific callbacks)

### Material Organization (Week 2)
1. **Centralized materials in core/materials.rs**:
   - GridMaterial: Grid rendering with distance-based fade
   - GizmoMaterial: Transform gizmo with always-on-top rendering
   - OutlineMaterial: Selection outline using inverted normals
2. **Benefits of centralization**:
   - Single source of truth for all custom materials
   - Easier to find and modify materials
   - Consistent with domain-based architecture (core = shared fundamentals)
   - Clean imports: `use crate::editor::core::materials::{GridMaterial, GizmoMaterial, OutlineMaterial};`
3. **Shaders remain in assets/shaders/**:
   - `grid.wgsl`: Grid line rendering with distance fade
   - `gizmo_material.wgsl`: Gizmo fragment shader (color + emissive)
   - Separation of Rust structs (CPU) and WGSL code (GPU)
4. **Material registration in EditorPlugin**:
   - `MaterialPlugin::<GridMaterial>::default()`
   - `MaterialPlugin::<GizmoMaterial>::default()`
   - No need for OutlineMaterial plugin (uses StandardMaterial resource)

### Scene Persistence System (Week 3)
1. **Option<PathBuf> for CurrentFile**: Defaults to None (unset) instead of always having a path
   - Cleaner mental model: "untitled" truly means no file yet
   - Enables proper "Save As" flow when no file is open
   - has_path() helper method makes intent clear in code
2. **File menu as single source of truth**: Menu UI checks dirty state before triggering events
   - Simpler than event interception (avoids loops)
   - ConfirmationDialog shows on dirty state detection
   - Menu bar has access to all resources it needs
3. **Native file dialogs via rfd crate**: OS-native look and feel
   - Familiar UX for users (matches OS conventions)
   - Built-in file type filtering (.yaml/.yml)
   - No need to build custom file browser UI
4. **Separate dialog resources**: ConfirmationDialog and ErrorDialog as independent systems
   - ConfirmationDialog: Handles unsaved changes workflow
   - ErrorDialog: Displays save/load failures
   - Both render in EguiPrimaryContextPass schedule
   - Modal windows block interaction until dismissed
5. **CurrentFile helper methods provide clean API**:
   - `get_path()` returns Option<PathBuf> (explicit about "no file" state)
   - `has_path()` for boolean checks
   - `get_filename()` returns "untitled" when path is None
   - `mark_dirty()`/`mark_clean()` for change tracking
   - `is_dirty()` for status queries
6. **Keyboard shortcuts integrate with dialogs**:
   - Ctrl+S: Opens Save As dialog when no file is open
   - Ctrl+O: Always opens file picker, checks for unsaved changes first
   - No assumptions about "default" file paths
   - User always in control of file locations
7. **Error handling with user feedback**:
   - Console logging for developers (info!/error! macros)
   - ErrorDialog for users (friendly messages with context)
   - Both directory creation and file I/O errors handled
   - YAML parsing errors show file path in message

## References
- Blender's transform gizmo system (industry standard UX)
- Unity's scene editor (inspector panel layout)
- Unreal's level editor (play-in-editor workflow)
- Godot's editor (EGUI panel approach for Rust ecosystem)

## What's Next: Iteration 2

With the editor MVP proving we can hand-craft scenes with primitives, Iteration 2 will focus on:

1. **Inspector Panel Refactor** - Transition from monolithic inspector to component-driven architecture where each component type can register its own sub-panel with custom UI, enabling extensibility and proper validation (see `uncommitted/editor-inspector.md` for detailed specification)

2. **Custom 3D Model Import** - GLTF/GLB asset pipeline with proper material handling, LOD support, and mesh optimization for the low-poly aesthetic

3. **Prefab System** - Reusable compound objects with override tracking, nested prefabs, and variant management for efficient content creation

4. **Advanced Snapping** - Edge-to-edge, vertex, and surface alignment tools to enable precise modular construction beyond simple grid snapping

5. **Material Editor** - Proper PBR material authoring with texture assignment, presets, and real-time preview for artistic control over the colony's visual atmosphere

Essentially evolving from "place cubes" to "assemble complex environments from custom assets and prefabs" while building the foundational systems (extensible inspector, asset pipeline) that will support all future editor features.
