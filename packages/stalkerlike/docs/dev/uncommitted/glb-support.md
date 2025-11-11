# GLB/GLTF Model Import Support

## Overview
Enable the editor to load custom 3D models from the `assets/models/` directory tree, bridging the gap between Blender exports and editor placement. This transforms the editor from "primitive shapes only" to "full custom asset pipeline," unlocking the low-poly art style and modular level design.

## Context
**Current State** (Iteration 1):
- Editor spawns 5 hardcoded primitives (cube, sphere, plane, cylinder, capsule)
- No way to import custom Blender exports
- `assets/models/` directory exists but is unused by editor
- Art pipeline spec defines GLB export workflow but no import path

**Next Step** (Iteration 2):
- Scan `assets/models/` directory tree for `.glb`/`.gltf` files
- Display available models in asset browser UI
- Support placement system (preview ghost + click to spawn)
- Preserve vertex colors and materials from Blender exports
- Enable modular tunnel system and environmental storytelling props

**Long-term Vision**:
- Foundation for prefab system (visual atoms → molecules)
- Enables full art pipeline (Blender → Editor → Level)
- Supports procedural generation (shared vocabulary of assets)

**Existing Assets**: 
- `assets/models/pipes/pipe_2m_8m_hollow.glb` exists but currently unused
- This feature will make it (and future models) accessible in the editor

## User Stories

### Story 1: Artist Workflow
**As an artist**, I want to export a model from Blender and immediately see it in the editor, so I can iterate on asset design without programmer intervention.

**Acceptance Criteria:**
- Export `mining_drill.glb` from Blender → save to `assets/models/industrial/`
- Launch editor → see `mining_drill` in asset browser under "Industrial" category
- Click asset → preview ghost appears in viewport
- Click to place → drill spawns with correct scale, materials, and vertex colors
- No code changes required between export and placement

### Story 2: Level Designer Workflow
**As a level designer**, I want to browse a structured library of models organized by category, so I can quickly find the right asset for environmental storytelling.

**Acceptance Criteria:**
- Asset browser shows folder hierarchy matching `assets/models/` structure:
  - `/pipes/` (pipe_2m_8m_hollow, etc.)
  - `/industrial/` (drills, carts, valves)
  - `/abandoned/` (sleeping bags, corpses, debris)
  - `/c7_infected/` (crystals, corrupted geometry)
  - `/structural/` (tunnel modules, walls, floors)
- Can expand/collapse categories
- Can search by name (e.g., "pipe" finds `pipe_2m_8m_hollow`)
- Visual thumbnails for each model (stretch goal - can be text list initially)

### Story 3: Tunnel Module Placement
**As a level designer**, I want to place modular tunnel pieces to build complex environments, so I can create narrative chunks efficiently.

**Acceptance Criteria:**
- Place `tunnel_straight.glb` from structural category
- Rotate with gizmo to align with intended direction
- Snap to grid (0.5m) for precise connection points
- Place `tunnel_corner.glb` adjacent → connects seamlessly
- Build multi-room environment using 5 core tunnel modules
- Save scene → all tunnel pieces preserved with correct transforms

**Note:** Chunk sizes are arbitrary (not fixed to 32m×32m), allowing flexibility for different level designs

## Requirements

### Asset Discovery
**What:** Editor automatically finds all 3D model files in the asset directory structure on startup.

**Why:** Artists need zero-friction workflow from Blender export to editor availability. Manual asset registration creates bottlenecks and breaks the creative flow.

**Outcome:** New GLB file appears in editor after restart, organized by folder structure.

**Stretch Goal (Hot Reload):** File system watcher detects new/modified GLB files and updates catalog without restart. Artist exports from Blender, switches to editor window, new model appears in browser within 1 second. Reduces iteration time from 30 seconds to 5 seconds.

**Acceptable Fallback (Hotkey Reload):** F5 or Ctrl+R hotkey rescans assets directory without full editor restart. Faster than restart (5-10 seconds vs 30 seconds), simpler than file watching.

### Asset Organization
**What:** Models displayed in hierarchical categories matching folder structure.

**Why:** Large asset libraries become unusable without organization. Folder-based categorization is intuitive and requires no additional metadata files.

**Outcome:** Designer can quickly find "pipes/pipe_2m_8m_hollow" by expanding the Pipes category, just like file explorer.

### Model Placement
**What:** Click a model in the browser to enter placement mode with preview, then click in viewport to spawn.

**Why:** Must match existing primitive placement workflow. Designers already know this pattern - don't make them learn something new.

**Outcome:** Placing a custom GLB model feels identical to placing a primitive cube. Preview shows exact appearance, respects grid snapping.

### Material Fidelity
**What:** Vertex colors and PBR properties from Blender export are preserved exactly.

**Why:** Art pipeline relies on vertex colors for the low-poly aesthetic. Materials must transfer 1:1 from Blender to editor or visual style breaks.

**Outcome:** Model looks identical in editor as it did in Blender viewport. No surprises, no iteration loop broken.

### Scene Hierarchy Preservation
**What:** Multi-mesh GLB files spawn with their parent-child relationships intact.

**Why:** Complex models (e.g., drill with moving parts) are built as hierarchies in Blender. Flattening destroys structure and breaks future animation support.

**Outcome:** Transform gizmo operates on root, all child meshes move together. Hierarchy visible in inspector if needed.

### Persistence
**What:** Scenes save model references (paths), not embedded geometry.

**Why:** Version control and file sizes become unmanageable with embedded assets. Path references keep scenes lightweight and diffable.

**Outcome:** YAML file stores "models/pipes/pipe_2m_8m_hollow.glb" string. Loading scene resolves path to actual asset. Committed scene files stay under 200KB.

### Inspector Metadata
**What:** Inspector shows model source path, poly count, submesh count when model selected.

**Why:** Designers need visibility into asset complexity for performance budgeting. Read-only metadata prevents accidental edits to shared assets.

**Outcome:** Select placed pipe, inspector shows "Source: models/pipes/pipe_2m_8m_hollow.glb" and "Triangles: 348". Designer knows budget impact immediately.

## Non-Requirements (Explicitly Out of Scope)

**Material Editing:** No color pickers or PBR sliders for MVP. Models use materials as exported. Material overrides deferred to future iteration when prefab system exists.

**Thumbnails:** Text-only asset list acceptable. Image previews are polish, not blocker for workflow.

**Search/Filter:** Browse by category is sufficient. Search bar is nice-to-have, defer if time-constrained.

**Animation Preview:** Static placement only. Animation support deferred until gameplay needs it.

**Drag-and-Drop:** Click-to-select from browser is proven pattern. Drag from file explorer is polish.

## Acceptance Criteria (Must Pass Before Merge)

### Discovery & Organization
- [ ] Place `test.glb` in `assets/models/test_category/` → launches editor → sees "test" under "Test Category" in asset browser
- [ ] 50 GLB files across 5 folders → all appear organized correctly
- [ ] Duplicate filenames in different folders → both appear with category prefix

### Placement Workflow
- [ ] Click model in browser → preview ghost appears at cursor
- [ ] Move mouse → preview follows ground plane
- [ ] Click → model spawns with identical materials to Blender export
- [ ] ESC → preview cancels, no entities spawned
- [ ] Grid snap enabled → preview snaps to 0.5m grid
- [ ] Click placed model → transform gizmo appears on root entity

### Material & Hierarchy
- [ ] Export model with red vertex colors → place in editor → appears red
- [ ] Export model with emissive material → place in editor → glows correctly
- [ ] Export model with 3-level hierarchy (root → body → detail) → place in editor → hierarchy inspector shows all 3 levels
- [ ] Select root → gizmo operates on root → all children move together

### Persistence
- [ ] Place 10 models → save scene → close editor → reopen → load scene → all 10 models restored with correct positions/rotations
- [ ] YAML file contains model paths (e.g., "models/pipes/pipe.glb") not embedded geometry
- [ ] Delete GLB file → load scene → error logged, red placeholder cube spawned with "MISSING MODEL" label

### Performance
- [ ] 500 GLB files in assets → editor starts in <3 seconds
- [ ] Click model → preview appears in <100ms
- [ ] Place 100 models in scene → editor maintains 60fps during camera movement
- [ ] Scene with 100 models → YAML save completes in <500ms

### Inspector
- [ ] Select placed model → inspector shows "Source: models/..." path
- [ ] Inspector shows triangle count (read-only)
- [ ] Inspector shows submesh count (read-only)

### 3. Asset Browser UI Integration
**Extend existing asset browser panel to show model hierarchy**

**Current Asset Browser** (Iteration 1):
- Simple vertical list of 5 primitives
- Click to enter placement mode
- Located on left side panel

**Enhanced Asset Browser** (Iteration 2):
- **Two Sections:**
  1. **Primitives** (collapsible) - existing cube/sphere/etc
  2. **Models** (collapsible) - new GLB import section
- **Category Tree:**
  - Folder icon + name (e.g., 📁 Industrial)
  - Expand/collapse arrow
  - Nested models indented under category
- **Model Entries:**
  - Model name (filename without extension)
  - Click to enter placement mode (same as primitives)
  - Optional: Right-click context menu (inspect, reload, etc.)
- **Search Bar:**
  - Filter models by name
  - Live filtering as you type
  - Clear button

**UI Layout:**
```
┌─────────────────────────┐
│ Asset Browser           │
├─────────────────────────┤
│ [Search: _________ X]   │
├─────────────────────────┤
│ ▼ Primitives            │
│   • Cube                │
│   • Sphere              │
│   • Plane               │
│   • Cylinder            │
│   • Capsule             │
├─────────────────────────┤
│ ▼ Models                │
│   ▶ Industrial          │
│   ▼ Structural          │
│     • tunnel_straight   │
│     • tunnel_corner     │
│     • tunnel_T          │
│     • tunnel_cross      │
│   ▶ Abandoned           │
│   ▶ C7 Infected         │
└─────────────────────────┘
```

### 4. Placement System Extension
**Reuse existing placement system with GLB scene support**

**Current Placement** (Iteration 1):
- `PlacementState` resource tracks active mode
- Preview entity with semi-transparent material
- Ground-plane ray intersection for positioning
- Click to spawn, ESC to cancel
- Works with primitives only

**Enhanced Placement** (Iteration 2):
- **Scene Spawning:**
  - Load scene from `ModelAsset.scene` handle
  - Wait for scene to load (show loading indicator if needed)
  - Extract meshes and materials from scene hierarchy
  - Spawn as entity with `EditorEntity` marker
  - Apply same placement logic (preview, grid snap, etc.)
- **Preview Ghost:**
  - Clone scene entities for preview
  - Apply semi-transparent material override to all meshes
  - Update position/rotation during hover (same as primitives)
  - Clean up preview entities on placement or cancel
- **Multi-Mesh Support:**
  - GLB files may contain multiple meshes (hierarchy)
  - Spawn root entity with children for each mesh
  - Preserve parent-child relationships from Blender
  - Transform gizmo operates on root entity
- **Material Preservation:**
  - Keep vertex colors from Blender export
  - Preserve base color and PBR properties (metallic, roughness)
- Handle emissive materials (e.g., emergency lights)
- Support alpha transparency (e.g., glass, crystals)

**Model Instance Tracking:**
- Each placed model tracks its source asset for prefab system
- Optional material overrides per instance (customization without affecting source)
- Maintain reference to original scene for updates and prefab creation### 5. Scene Persistence Integration
**Extend YAML serialization to store model references**

**Current Serialization** (Iteration 1):
```yaml
entities:
  - name: "cube_1"
    components:
      - type: primitive
        primitive_type: Cube
      - type: transform
        position: [0, 1, 0]
```

**Enhanced Serialization** (Iteration 2):
```yaml
entities:
  - name: "mining_drill"
    components:
      - type: model
        path: "models/industrial/mining_drill.glb"
      - type: transform
        position: [5.2, 0.0, 3.1]
        rotation: [0, 0.7071, 0, 0.7071]  # Quaternion (90° Y-axis)
        scale: [1.0, 1.0, 1.0]
      - type: material_overrides  # Optional
        overrides:
          0: {base_color: [0.8, 0.3, 0.2, 1.0]}  # Rust color
```

**Deserialization Process:**
- Read model path from YAML
- Look up in model catalog by path
- Load scene if not already cached
- Spawn entity with model instance tracking component
- Apply material overrides if present

### 6. Inspector Panel Integration
**Display model metadata in inspector**

**Current Inspector** (Iteration 1):
- Shows transform (position, rotation, scale)
- Entity name
- Component list (minimal)

**Enhanced Inspector** (Iteration 2):
- **Model Section** (when `ModelInstance` present):
  - Source model path (read-only)
  - Display name (editable)
  - Poly count (read-only - extracted from mesh)
  - Submesh count (read-only)
  - Bounding box size (read-only)
- **Material Override Section:**
  - List each submesh (Submesh 0, Submesh 1, etc.)
  - Base color picker (overrides vertex colors)
  - Metallic/Roughness sliders (0.0-1.0)
  - Emissive color picker (optional)
  - "Reset to Default" button per submesh

## Technical Design

### Module Organization
```
src/editor/
├── objects/
│   ├── mod.rs              # ObjectsPlugin coordinator
│   ├── primitives.rs       # Existing primitive spawning
│   ├── models.rs           # NEW: GLB/GLTF loading and spawning
│   └── placement.rs        # Extend to support models
├── persistence/
│   ├── scene.rs            # Extend serialization for models
│   └── components.rs       # Add ModelInstance component
└── ui/
    ├── asset_browser.rs    # Extend to show model tree
    └── inspector.rs        # Add model metadata display
```

### Bevy Asset System Integration
**Leverage Bevy's built-in scene loading**

Bevy provides:
- Scene root component for spawning GLTF scenes
- Asset server for async loading
- GLTF asset type with mesh/material extraction
- Scene hierarchy preservation (parent-child)

**Scene Hierarchy:**
- GLB root becomes entity with scene root component
- Meshes become child entities with mesh and material components
- Transforms preserved from Blender export
- Animations stored but not played (static placement)

### Performance Considerations

**Asset Catalog:**
- Build on startup: ~1ms per 100 files (acceptable for <500 models)
- Store in persistent resource (no rebuild per frame)
- Future: Cache to JSON for faster subsequent loads

**Scene Loading:**
- Async loading prevents frame drops
- On-demand loading (only load when clicked) vs preload all
- Recommendation: On-demand for MVP, preload for final release

**Preview Ghosts:**
- Clone scene entities (cheap - shared mesh data)
- Override materials (new material instance per preview)
- Limit to 1 preview at a time (clean up on mode switch)

**Memory Budget:**
- 500 models × ~500KB each = ~250MB total
- Bevy asset system handles unloading unused assets
- Scene references prevent unload (acceptable - editor keeps all loaded)

## Integration Points

### With Existing Systems

**Placement System:**
- Extend placement state to support model placement mode
- Reuse preview and spawn logic with scene spawning
- Grid snapping works identically (operates on root transform)

**Selection System:**
- Models spawn with editor entity marker → already selectable
- Multi-mesh models: Select root, gizmo operates on root transform
- No changes needed (hierarchy already supported)

**Transform Gizmos:**
- Works on root entity transform
- Child meshes move with parent (transform hierarchy)
- No changes needed

**Inspector Panel:**
- Query for model instance component
- Display model metadata section
- Add material override UI

**Scene Persistence:**
- Add model component variant to serialization
- Serialize path and material overrides
- Deserialize by looking up in model catalog

## User Workflows (Success Scenarios)

### Artist: Rapid Iteration
1. Artist modifies drill model in Blender
2. Export → overwrites `models/industrial/drill.glb`
3. Alt+Tab to editor → press F5 to rescan assets (or restart editor, or wait 1s if hot reload)
4. Click drill in browser → new version appears in preview
5. Place → updated model in scene
**Time to see changes: 5-10 seconds (hotkey rescan) | 30 seconds (restart) | <5 seconds (hot reload)**

### Designer: Building Environment
1. Browse Models > Structural → place tunnel pieces with grid snap
2. Browse Models > Pipes → place `pipe_2m_8m_hollow` along ceiling
3. Duplicate pipe 10 times to create piping run
4. Browse Models > Industrial → add carts, spools, equipment
5. Save scene → all models referenced by path
**Time to dress 32m corridor: <10 minutes**

## Error Handling (Required Behaviors)

**Missing Directory:** Assets/models/ doesn't exist → empty Models section, log warning. Editor still functional.

**Invalid GLB:** File corrupted or non-GLB → skip file, log error with path, show ⚠️ icon in browser if partially readable.

**Missing Asset on Load:** YAML references deleted GLB → spawn red error placeholder cube with label, log error with expected path. Scene partially loads.

**Load Timeout:** Scene takes >5 seconds → show spinner, continue waiting. No crash.

**Vertex Color Missing:** GLB has no vertex colors → load anyway with default white material. Log warning (violates art pipeline but non-fatal).

## Success Metrics

**Workflow Time:** Artist export → editor placement → scene save must complete in <60 seconds.

**Catalog Size:** Must handle 500+ GLB files across 20 categories without performance degradation.

**Material Accuracy:** 100% of vertex colors and PBR properties preserved from Blender export. No visual drift.

**Save File Size:** Scene with 100 placed models → YAML under 50KB. Path references only, no embedded geometry.

**Error Recovery:** Missing asset → placeholder spawns, scene partially loads. Editor never crashes on bad assets.

## Future Enhancements (Explicitly Deferred)

**Hot Reload (Stretch Goal):** File system watcher automatically detects new/modified GLB files and updates catalog without restart. Reduces iteration time from 30s to <5s. Low complexity, high value - consider for MVP if time allows. **Acceptable fallback:** F5/Ctrl+R hotkey to manually rescan (10s instead of 30s restart).

**Prefab System:** Model + components saved as reusable prefabs. Blocks on functional component system.

**LOD Support:** Auto-detect and switch LOD levels based on distance. Blocks on performance profiling showing need.

**Animation Preview:** Play skeletal animations in inspector. Blocks on gameplay needing animated props.

**Material Overrides:** Per-instance color/PBR tweaking. Blocks on prefab system (overrides without it create chaos).

**Thumbnail Generation:** Render preview images for each model. Polish only, doesn't improve workflow speed.

**Asset Validation:** Poly budget checking, vertex color verification. Nice-to-have, artists already follow pipeline.

## Dependencies & Constraints

**Bevy Scene System:** Must use Bevy's built-in GLB loading. Custom parsers create maintenance burden.

**Art Pipeline Compliance:** Vertex colors required, no external textures. Models violating this log warnings but still load.

**YAML Serialization:** Must serialize path references only. Embedded geometry breaks version control and file diffs.

**Grid Snapping:** Must respect existing 0.5m grid. Models place on same grid as primitives.

**Transform Hierarchy:** Must preserve Blender parent-child structure. Flattening breaks future animation support.

---

## Summary

**What:** Load custom GLB models from assets folder into editor with same workflow as primitives.

**Why:** Primitives-only editor blocks all art content and level design. This unlocks the full art pipeline (Blender → Editor → Level) and enables the low-poly environmental storytelling that defines the game's visual identity.

**Success:** Artist exports GLB, restarts editor, clicks model in browser, places in scene, saves. Model appears identical to Blender viewport. Time to iteration: <60 seconds.

**Priority:** Critical path blocker. All future art and design work waits on this.
