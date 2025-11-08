# Iteration 1: Editor MVP - Scene Building Foundations

## Context
With Iteration 0 complete (game mode with save/load, player controller, basic 3D scene) we now need a minimal viable editor that can create and manipulate 3D scenes. This iteration focuses on the essential tools needed to hand-craft level geometry and props.

This editor will create **narrative chunks** - hand-authored setpieces that define the critical path and major encounters. These will later integrate with the procedural generation system (see `uncommitted/procedural_generation.md`) which fills in the connective tissue between your hand-crafted content.

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

### World Scale Conventions
- **1 unit = 1 meter** in world space
- **Chunk size = 32x32x32 meters** (aligns with streaming system)
- **Y-axis = depth**: 0 is surface, negative values go deeper (e.g., -500 = 500m deep)
- **Coordinate system**: Right-handed (X: East/West, Y: Up/Down, Z: North/South)

### Component Structure
```rust
// Editor-specific components
#[derive(Component)]
struct EditorEntity; // Marks entities as part of editor scene

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

### Core Functionality
- [x] Can launch editor with `--editor` flag
- [ ] Can fly around scene smoothly with editor camera
- [ ] Grid display shows spatial reference
- [ ] Can toggle grid snapping with G key (0.5m position, 15° rotation)
- [ ] Can toggle chunk boundary visualization (B key)
- [ ] Can spawn primitives (cube, sphere, plane) into scene
- [ ] Can select objects by clicking in viewport (single object)
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
- [ ] Gizmo interactions feel responsive and accurate
- [ ] Selection is unambiguous (clear visual feedback)
- [ ] **Grid snapping feels natural and predictable (visual + functional from week 1)**
- [ ] **Play mode works by end of week 1 (tight iteration loops)**
- [ ] Scene YAML files are human-readable and version-control friendly
- [ ] No crashes when switching between editor and play mode
- [ ] Camera movement feels smooth and controllable
- [ ] Chunk bounds visualization helps with spatial awareness (32m reference)
- [ ] Duplicate offset (+1m X) is consistent and predictable
- [ ] Multi-select only available after grouping is implemented (no half-features)

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

## Implementation Priority

### Week 1: Core Editing + Early Testing
1. Editor camera controller (fly-around)
2. Grid display with snapping (both visual grid and snap logic - G key toggle)
3. Primitive spawning (cube, sphere, plane)
4. Click selection system (single object only)
5. Basic inspector panel (read-only transforms)
6. **Play mode entry/exit (P key) - critical for iteration loops**

### Week 2: Transform Tools
7. Translate gizmo with drag interaction (respects grid snapping)
8. Rotate gizmo with drag interaction (15° snap when enabled)
9. Scale gizmo with drag interaction
10. Inspector with editable numeric fields
11. Chunk boundary visualization (B key toggle)

### Week 3: Scene Management + Multi-Object
12. Scene serialization to YAML (save)
13. Scene deserialization from YAML (load)
14. Chunk metadata in scene files (position, bounds, faction)
15. Group/ungroup operations (Ctrl+G / Ctrl+Shift+G)
16. Multi-select (Ctrl+click) and box select (now that grouping exists)
17. Duplicate/delete operations (with defined +1m X-axis offset)
18. Hierarchy panel

### Week 4: Polish & Stability
19. Player spawn point designation
20. Scene dirty flag and auto-save (every 5 minutes)
21. Polish gizmo visuals and interactions
22. Bug fixes and edge cases
23. Keyboard shortcut refinements

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

## References
- Blender's transform gizmo system (industry standard UX)
- Unity's scene editor (inspector panel layout)
- Unreal's level editor (play-in-editor workflow)
- Godot's editor (EGUI panel approach for Rust ecosystem)

## What's Next: Iteration 2

With the editor MVP proving we can hand-craft scenes with primitives, Iteration 2 will focus on importing custom 3D models (GLTF/GLB), building a prefab system for reusable compound objects, and implementing advanced snapping (edge-to-edge, vertex, surface alignment) to enable efficient modular construction - essentially evolving from "place cubes" to "assemble complex environments from custom assets and prefabs" while introducing a proper asset pipeline and material editor for artistic control over the colony's visual atmosphere.
