# Iteration 02 - GLB Support and Enhanced Gizmos

## BLUF (Bottom Line Up Front)

**Problem**: Two critical blockers prevent productive level design:
1. GLB models work but won't scale (no search, no hierarchy, minimal UI - usable for <10 models, breaks at 50+)
2. Gizmo drag operations fail ~70% of the time (unreliable interaction)

TODO: undo/redo, add/remove components, click-drag transforms, (unique) entity counter, prefabs, scripting, drop-in testing, cut-paste

**Solution**: Build proper asset browser with search/hierarchy for GLB catalog + fix gizmo picking/drag lifecycle.

**Timeline**: 3-4 weeks

**Impact**: Scales art pipeline to hundreds of models and makes transform tools reliable.

---

## Desired Outcomes

### GLB Support
- **Scales to hundreds of models** - Asset browser handles large catalogs without UI degradation
- **Quick discovery** - Search filters 200 models to target in <1 second
- **Organized browsing** - Folder hierarchy mirrors disk structure (pipes/, industrial/, structural/)
- **Same workflow as primitives** - Click model → preview → place (already works, needs better UI)
- **Materials preserved exactly** - Vertex colors and PBR from Blender → Editor 1:1 (verify existing implementation)
- **Graceful error handling** - Missing assets show red placeholder, scene loads partially
- **Fast iteration** - F5 rescan updates catalog in <5 seconds

### Enhanced Gizmos
- **Drag always works** - >99% success rate, continuous tracking from press to release
- **No selection conflicts** - Gizmo handles take priority over objects (deterministic)
- **Clear visual feedback** - Hover/active states obvious, no flickering
- **Multi-select cohesion** - Objects move as rigid group, spatial relationships maintained
- **Smooth performance** - 60+ FPS with no gizmo lag

---

## Implementation Plan

### GLB Support (3 weeks)

#### High-Level Tasks
1. **Asset Browser Overhaul** - Replace flat list with hierarchical tree view (primitives + models sections)
2. **Search & Filter** - Live filtering across all assets by name
3. **Error Handling** - Missing/invalid asset recovery with placeholders
4. **Inspector Metadata** - Display source path, poly count, bounds for selected models
5. **Material Verification** - Test and document vertex color/PBR preservation (existing placement already works)

#### Detailed Checklist

**Week 1: UI Integration** 🔴
- [ ] Refactor `asset_browser.rs` to show primitives + models in separate collapsible sections
- [ ] Build folder tree from `AssetBrowserState.glb_assets` using `relative_path`
- [ ] Group assets by directory (e.g., "pipes/", "industrial/", "structural/")
- [ ] Add expand/collapse arrows for categories
- [ ] Click asset → call existing `start_placement_asset()` with `PlacementAsset::GlbModel`
- [ ] Remove GLB display from hierarchy panel (keep scanning logic)
- [ ] Test: 50 models across 5 folders → all appear organized in asset browser

**Week 2: Search & Error Handling** 🟡
- [ ] Add search bar above asset sections in asset browser
- [ ] Filter `asset_catalog.primitives` and `asset_browser_state.glb_assets` by name
- [ ] Case-insensitive matching, update on text change
- [ ] Clear button resets filter
- [ ] Check asset existence before spawning: `asset_server.get_load_state()`
- [ ] On load failure: spawn red cube + `MissingAsset` marker + log error
- [ ] Test: Delete GLB → load scene → red placeholder appears with "MISSING: {path}"

**Week 3: Inspector & Verification** 🟢
- [ ] Detect `GlbModel` component in inspector system
- [ ] Display metadata section: source path (read-only), poly count, submesh count
- [ ] Extract poly count from mesh asset via `Mesh::count_vertices()`
- [ ] Export test model with red vertex colors → place → verify color preserved (existing system)
- [ ] Export test model with emissive material → place → verify glow preserved (existing system)
- [ ] Test alpha transparency (glass material) with existing placement
- [ ] Bind F5 key → `asset_browser_state.scan_glb_files()` → log results (optional)
- [ ] Document any material limitations found

---

### Enhanced Gizmos (3 weeks)

#### High-Level Tasks
1. **Fix Drag Lifecycle** - Proper press → drag → release event handling
2. **Implement Priority System** - Gizmos beat objects in picking resolution
3. **Visual Feedback** - State machine for idle → hover → active transitions
4. **Multi-Select Coordination** - Maintain offsets from gizmo center during drag
5. **Performance Polish** - Optimize updates, smooth interpolation

#### Detailed Checklist

**Week 1: Drag Interaction** 🔴
- [ ] Verify gizmo handles have `PickableBundle` or equivalent
- [ ] Observe `Pointer<Down>` events → store initial transform + drag start position
- [ ] Observe `Pointer<Drag>` events → calculate delta from start, apply to transform
- [ ] Observe `Pointer<Up>` events → finalize transform, clear drag state
- [ ] Ensure drag continues if cursor leaves handle (use frame-persistent state)
- [ ] Add debug logging: "Drag started on {axis}", "Drag delta: {distance}", "Drag ended"
- [ ] Test: Click X arrow → drag right → object moves smoothly, release → movement stops
- [ ] Test: Drag off-screen → release outside window → still finalizes correctly

**Week 2: Picking Priority & Visual Feedback** 🟡
- [ ] Research `bevy_picking` priority features (layers, `PickingBehavior`)
- [ ] If built-in insufficient: implement priority filter examining all raycast hits
- [ ] Assign high priority to gizmo handles, lower to scene objects
- [ ] Selection system checks priority before deselecting/selecting
- [ ] Gizmo system checks priority before activating drag
- [ ] Test: Click gizmo overlapping object → gizmo activates, no selection change
- [ ] Store material states: base color, hover color, active color
- [ ] Implement state machine: Idle → Hovered → Dragging
- [ ] On hover enter: brighten handle (increase emissive)
- [ ] On drag start: maximize brightness
- [ ] On drag end: restore to hover (if still hovering) or idle
- [ ] Test: Hover handle → highlights, move away → unhighlights, drag → brighter, release → restores

**Week 3: Multi-Select & Polish** 🟢
- [ ] Calculate gizmo position as centroid of selected entities
- [ ] On drag start: measure each object's offset from gizmo center
- [ ] Store offsets in drag state resource
- [ ] During drag: apply delta to gizmo center, recompute each object position (center + offset)
- [ ] Apply grid snapping to gizmo center (offsets unchanged)
- [ ] Test: Select 3 cubes → drag → all move together, relative positions preserved
- [ ] Profile gizmo update systems with `tracy` or logging
- [ ] Batch material queries to reduce overhead
- [ ] Add position interpolation for smooth lag (optional)
- [ ] Test: 100 objects in scene, gizmos active → maintains 60 FPS
- [ ] Verify no visible lag between camera movement and gizmo position updates

---

## Testing Checklist

### GLB Integration
- [ ] 50 GLB files across 5 folders → all appear in asset browser organized by folder
- [ ] Search "pipe" → filters to pipe models only
- [ ] Click "tunnel_straight" → preview appears, click viewport → spawns correctly
- [ ] Save scene with 10 GLB models → reload → all restored with correct transforms
- [ ] Delete GLB file → load scene referencing it → red placeholder appears, logs error
- [ ] Export red vertex color model → place → appears red in editor
- [ ] Export emissive material → place → glows in editor

### Gizmo Reliability
- [ ] Click X arrow → drag right → smooth continuous movement (no stuttering)
- [ ] Click gizmo handle overlapping object → gizmo activates, object NOT selected
- [ ] Hover handle → highlights immediately, move away → unhighlights
- [ ] Drag handle → brightens, release → returns to appropriate state
- [ ] Select 3 objects → drag gizmo → all move as group, spacing maintained
- [ ] Hold Shift while dragging → moves 4x faster
- [ ] Hold Ctrl while dragging → moves 0.25x slower (precise)
- [ ] Grid snap enabled → gizmo snaps to 0.5m increments during drag

---

## Success Criteria

**GLB Support Complete When**:
- Asset browser handles 100+ models with folder hierarchy and search
- Click asset → place workflow smooth and discoverable
- Materials (vertex colors, PBR) verified working from Blender
- Missing assets handled gracefully (red placeholder, scene loads)

**Gizmo Enhancement Complete When**:
- Drag success rate >99% (works nearly every time)
- Zero selection conflicts (gizmos always win)
- Visual feedback clear and consistent
- Multi-select transforms maintain spatial relationships
- 60+ FPS performance maintained

**Iteration 2 Done When**:
- All testing checklist items pass
- Artists can export from Blender, press F5, and place immediately
- Designers can precisely position assets with reliable gizmos
- No critical bugs or workflow blockers

---

## References

- **Current Implementation**: 
  - `packages/stalkerlike/src/editor/ui/hierarchy.rs` - GLB scanning (lines 80-210)
  - `packages/stalkerlike/src/editor/objects/placement.rs` - Placement system
  - `packages/stalkerlike/src/editor/objects/gizmo.rs` - Transform gizmos
  - `packages/stalkerlike/src/editor/persistence/scene.rs` - YAML serialization
- **Specifications**:
  - `in-progress/glb-support.md` - Complete GLB feature spec
  - `in-progress/enhanced-gizmos.md` - Complete gizmo enhancement spec
- **Future Work**: 
  - Iteration 3: Inspector refactor, prefabs, advanced snapping, lighting

## References

- **Current Implementation**: 
  - `packages/stalkerlike/src/editor/ui/hierarchy.rs` - GLB scanning (lines 80-210)
  - `packages/stalkerlike/src/editor/objects/placement.rs` - Placement system
  - `packages/stalkerlike/src/editor/objects/gizmo.rs` - Transform gizmos
  - `packages/stalkerlike/src/editor/persistence/scene.rs` - YAML serialization
- **Specifications**:
  - `in-progress/glb-support.md` - Complete GLB feature spec
  - `in-progress/enhanced-gizmos.md` - Complete gizmo enhancement spec
- **Future Work**: 
  - Iteration 3: Inspector refactor, prefabs, advanced snapping, lighting
**Priority: CRITICAL - Unblocks entire art pipeline**

Enable loading custom 3D models from `assets/models/` directory, bridging the gap between Blender exports and editor placement. This transforms the editor from "primitive shapes only" to "full custom asset pipeline."

**Current State** (Partially Implemented):
- ✅ `GlbModel` component exists with path storage
- ✅ Asset directory scanning implemented (`scan_glb_files()` in hierarchy.rs)
- ✅ Automatic discovery on startup if directory found
- ✅ Recursive directory traversal for .glb/.gltf files
- ✅ GLB assets sorted and stored in `AssetBrowserState`
- ✅ Placement system supports `PlacementAsset::GlbModel` enum variant
- ✅ Preview ghost for GLB models using `SceneRoot`
- ✅ Spawning GLB entities with `SceneRoot`, `GlbModel`, `EditorEntity`
- ✅ YAML serialization/deserialization by path reference
- ✅ Scene loading resolves GLB paths and spawns via asset server
- ⚠️ UI only shows flat list in hierarchy panel, not asset browser

**What's Missing**:
- ❌ Asset browser UI doesn't display GLB assets (only primitives)
- ❌ No folder hierarchy display (assets scanned but not shown)
- ❌ No search/filter functionality
- ❌ F5 manual rescan implemented but no hotkey bound
- ❌ No error handling for missing/invalid GLB files on load
- ❌ No inspector metadata display (poly count, bounds, source path)
- ❌ Materials may not preserve correctly (needs verification)

**Target State**:
- All `.glb`/`.gltf` files displayed in asset browser with folder hierarchy
- Click-to-place workflow identical to primitives
- Materials and vertex colors preserved exactly
- Multi-mesh hierarchies maintained (already works)
- YAML references by path (already works)
- Error handling for missing assets (red placeholder)

#### User Workflows

**Artist: Rapid Iteration**
1. Modify drill model in Blender
2. Export → overwrites `models/industrial/drill.glb`
3. Press F5 in editor to rescan assets (or restart)
4. Click drill in browser → updated version appears
5. Place → new model in scene
**Time: 5-10 seconds (F5 rescan) | 30 seconds (restart)**

**Designer: Building Environment**
1. Browse Models > Structural → place tunnel pieces with grid snap
2. Browse Models > Pipes → place `pipe_2m_8m_hollow` along ceiling
3. Duplicate pipe 10 times to create piping run
4. Browse Models > Industrial → add carts, equipment
5. Save scene → all models referenced by path
**Time: <10 minutes to dress 32m corridor**

#### Implementation Phases

**Phase 1: Asset Browser UI Integration** (Week 1) 🔴 CRITICAL
- Move GLB asset display from hierarchy panel to asset browser panel
- Implement folder tree view with expand/collapse
- Display GLB assets organized by directory structure
- Reuse existing `AssetBrowserState` resource
- Click GLB asset → call `start_placement_asset()` with `PlacementAsset::GlbModel`

**Technical Details**:
- `AssetBrowserState` already has `glb_assets: Vec<GlbAsset>`
- Each `GlbAsset` has `name`, `path`, and `relative_path`
- Build folder tree from `relative_path` (e.g., "models/pipes/pipe.glb" → "pipes" category)
- Match UI structure shown in spec: Primitives section + Models section with categories

**Phase 2: Search and Filter** (Week 1-2) 🟡 MEDIUM
- Add search bar above asset sections
- Filter both primitives and GLB assets by name
- Live filtering as user types
- Clear button to reset filter
- Case-insensitive matching

**Phase 3: Error Handling** (Week 2) 🟢 HIGH
- Handle missing GLB files on scene load
- Spawn red placeholder cube with "MISSING MODEL" label
- Log error with expected path
- Scene loads partially (other objects still appear)
- Invalid GLB files during scan → skip with warning

**Technical Details**:
- Check if asset exists before loading: `asset_server.get_load_state(handle)`
- On load failure, spawn fallback entity with `MissingAsset` marker component
- Red material with emissive for visibility
- Inspector shows "Missing: {path}" when selected

**Phase 4: Inspector Metadata** (Week 2-3) 🔵 LOW
- Detect when selected entity has `GlbModel` component
- Display read-only metadata section:
  - Source path
  - Poly count (extract from mesh asset)
  - Submesh count
  - Bounding box dimensions
- Optional: Material override UI (can defer)

**Phase 5: Material Verification** (Week 3) 🟢 MEDIUM
- Test vertex color preservation from Blender exports
- Test PBR material properties (base color, metallic, roughness, emissive)
- Verify alpha transparency works
- Document any material limitations
- Add material validation logging

**Phase 6: F5 Hotkey Binding** (Week 3) ⚪ NICE-TO-HAVE
- Bind F5 key to rescan assets
- Show "Rescanning assets..." notification
- Update asset browser without full editor restart
- Log scan results (X files found in Y directories)

#### Acceptance Criteria

**Discovery & Organization**:
- [ ] Place `test.glb` in `assets/models/test_category/` → restart → appears under "Test Category"
- [ ] 50 GLB files across 5 folders → all organized correctly
- [ ] Duplicate filenames in different folders → both appear with category prefix

**Placement Workflow**:
- [ ] Click model → preview ghost appears at cursor
- [ ] Move mouse → preview follows ground plane with grid snap
- [ ] Click → model spawns identical to Blender export
- [ ] ESC → cancels placement
- [ ] Multi-mesh GLB → hierarchy preserved, gizmo on root

**Material & Hierarchy**:
- [ ] Export with red vertex colors → place → appears red
- [ ] Export with emissive material → place → glows correctly
- [ ] Export 3-level hierarchy → place → all 3 levels in inspector

**Persistence**:
- [ ] Place 10 models → save → reload → all restored correctly
- [ ] YAML contains paths (e.g., "models/pipes/pipe.glb"), not geometry
**Persistence**:
- [x] Place 10 models → save → reload → all restored correctly *(YAML serialization works)*
- [x] YAML contains paths (e.g., "models/pipes/pipe.glb"), not geometry *(ComponentData::GlbModel)*
- [ ] Delete GLB → load scene → red placeholder spawns with "MISSING MODEL" label

**Performance**:
- [x] 500 GLB files → editor starts in <3 seconds *(scan is fast)*
- [x] Click model → preview appears in <100ms *(async loading via Bevy)*
- [x] 100 models in scene → maintains 60fps during camera movement *(Bevy handles this)*

**Inspector**:
- [ ] Select placed model → inspector shows "Source: models/..." path
- [ ] Inspector shows triangle count (read-only)
- [ ] Inspector shows submesh count (read-only)

#### Error Handling

- **Missing Directory**: `assets/models/` doesn't exist → empty Models section, log warning *(already implemented)*
- **Invalid GLB**: Corrupted file → skip, log error, show ⚠️ icon in browser *(partial - needs UI)*
- **Missing Asset on Load**: YAML references deleted GLB → spawn red placeholder cube, log error *(needs implementation)*
- **Load Timeout**: Scene takes >5s → show spinner, continue waiting *(Bevy handles async loading)*
- **No Vertex Colors**: GLB missing vertex colors → load with white material, log warning *(needs verification)*

#### Non-Requirements (Out of Scope)

- ❌ Material editing (use Blender exports as-is)
- ❌ Thumbnail generation (text-only list acceptable)
- ~~❌ Search/filter~~ ✅ NOW IN SCOPE (Phase 2)
- ❌ Animation preview (static placement only)
- ❌ Drag-and-drop from file explorer
- ❌ LOD detection (future iteration)
- ~~❌ Hot reload file watcher~~ ⚪ F5 manual rescan (Phase 6, nice-to-have)

**See:** `in-progress/glb-support.md` for complete specification

---

### 2. Enhanced Transform Gizmos
**Priority: CRITICAL - Foundation for all positioning work**

Fix critical gizmo interaction issues that make precise positioning frustrating. Current implementation has unreliable drag tracking, selection priority conflicts, and inconsistent visual feedback.

**Current Problems**:
- ❌ Drag operations fail or lose tracking mid-motion (~30% success rate)
- ❌ Clicking gizmo handles sometimes selects objects instead
- ❌ Hover highlighting doesn't restore properly
- ❌ Multi-select transforms don't maintain spatial relationships
- ❌ No deterministic priority when gizmo and object overlap

**Target State**:
- ✅ >99% drag success rate (press → continuous drag → release)
- ✅ Gizmo handles always take priority over scene objects (zero conflicts)
- ✅ Clear visual feedback (idle → hover → active states)
- ✅ Multi-select moves objects as rigid group
- ✅ 60+ FPS performance with no gizmo lag

#### Core Objectives

**1. Reliable Click Targets**
- Gizmo handles have generous click areas (larger than visual appearance)
- Users can click near handle and still engage it
- No pixel-perfect precision required

**2. Continuous Drag Tracking**
- Drag persists from mouse-down to mouse-up regardless of cursor position
- Smooth, continuous feedback as objects move
- Works even if cursor goes off-screen
- Transform updates every frame with consistent delta calculations

**3. Unambiguous Interaction Priority**
- Clicking gizmo handle never selects/deselects objects
- Clicking object never triggers gizmo interaction
- When both under cursor, gizmo wins every time
- Priority hierarchy: Gizmos > UI > Scene Objects > Background

**4. Clear Visual Feedback**
- Hovering handle produces immediate visual response (color shift)
- Active drag state is visually distinct from idle/hover
- Feedback is instant (<16ms latency)
- State transitions are smooth, not jarring

**States**: Idle → Hover (brightness increase) → Active (maximum prominence) → Idle/Hover

**5. Cohesive Multi-Select Transforms**
- Selected objects move as single rigid group
- Relative positions stay constant during drag
- Rotation pivots around selection centroid
- Scaling maintains group proportions

**Technical Approach**: Calculate offsets from gizmo center on drag start, maintain throughout operation.

#### Implementation Phases

**Phase 1: Fix Drag Interaction** (Week 1) 🔴 CRITICAL
- Verify gizmo handles participate in `bevy_picking`
- Observe `Pointer<Down>` events to initialize drag state
- Observe `Pointer<Drag>` events to apply transforms continuously
- Observe `Pointer<Up>` events to finalize drag
- Store initial transforms on drag start
- Calculate deltas from drag start position (not frame-to-frame)
- Ensure drag continues even if cursor leaves handle
- Add debug logging for lifecycle (press → drag → release)

**Success Criteria**:
- [ ] Down fires reliably when clicking handles
- [ ] Drag fires continuously while dragging (regardless of cursor position)
- [ ] Up fires reliably when releasing mouse
- [ ] Drag delta is smooth and predictable
- [ ] Grid snapping works during drag
- [ ] Debug logs show complete event sequence

**Phase 2: Implement Picking Priority** (Week 1-2) 🟡 HIGH
- Test `bevy_picking`'s built-in priority system first
- Add generous collision volumes to gizmo handles
- If needed: Implement priority filtering examining all raycast hits
- Gizmo layer takes precedence over object layer
- Selection system checks priority before proceeding

**Success Criteria**:
- [ ] Gizmo handles always take priority over scene objects
- [ ] Can click objects that don't overlap gizmos (normal selection)
- [ ] Clicking empty space deselects (normal behavior)
- [ ] Debug logs show priority resolution for each click

**Phase 3: Improve Visual Feedback** (Week 2) 🟢 MEDIUM
- Store color states for each handle (base, hover, active, disabled)
- Implement state machine (Idle → Hovered → Dragging)
- Brighten active axis during drag
- Dim inactive axes during single-axis drag (optional)
- Add smooth color transitions (lerp over ~0.1s)
- Restore original colors on drag end

**Success Criteria**:
- [ ] Hovering highlights handle (emissive increase)
- [ ] Dragging makes handle brighter (active state)
- [ ] Releasing returns to hover (if still hovering) or idle
- [ ] No color flickering or state confusion

**Phase 4: Refine Multi-Select** (Week 2-3) 🔵 LOW
- Calculate gizmo center as centroid of selection
- Store per-object offsets from center on drag start
- Apply transforms maintaining relative positions
- Test rotation around multi-select center
- Verify grid snapping works for all objects

**Success Criteria**:
- [ ] Moving multi-select gizmo moves all objects as rigid group
- [ ] Rotating around center rotates group naturally
- [ ] Grid snapping doesn't break relative positioning

**Phase 5: Performance & Polish** (Week 3) ⚪ NICE-TO-HAVE
- Profile gizmo update systems (find bottlenecks)
- Optimize material updates (batch queries)
- Add interpolation to gizmo position updates (smooth lag)
- Consider instancing for arrow meshes (reduce draw calls)
- Add visual "snap" feedback when grid snapping occurs

**Success Criteria**:
- [ ] Gizmos update at 60+ FPS with large scenes
- [ ] No visible lag between camera movement and gizmo position
- [ ] Mode switching feels polished (not jarring)

#### Technical Architecture

**Drag Event Lifecycle**:
1. Press mouse on handle → Initialize drag (snapshot transforms)
2. Move mouse while held → Continuous updates (apply deltas)
3. Release mouse → Finalize drag (cleanup state)

**Key**: Drag must continue even if cursor leaves entity. Prefer `bevy_picking`'s drag events—they provide frame deltas and persist across cursor movement.

**Priority Resolution**:
- Single source of truth (resource holds the winner)
- Early resolution (before observers fire)
- Domain separation (each system checks priority guard)
- Extensible (new priorities slot in naturally)

**Multi-Object Transform**:
- Calculate gizmo as centroid of selection
- Measure each object's offset from center on drag start
- During drag, maintain exact offsets (prevents drift)
- For rotation, pivot around center while preserving relationships

#### Acceptance Criteria

**Drag Interaction**:
- [ ] Click X-axis arrow, drag right → object moves right (>99% success)
- [ ] Drag off-screen, release outside window → ends correctly
- [ ] Grid snap enabled → snaps to 0.5m increments during drag
- [ ] Hold Shift → moves 4x faster, Ctrl → 0.25x slower

**Selection Priority**:
- [ ] Click gizmo handle → does NOT deselect/reselect object
- [ ] Click where gizmo and object overlap → gizmo takes priority
- [ ] Click object away from gizmo → selects normally
- [ ] Click empty space → deselects normally

**Visual Feedback**:
- [ ] Hover handle → highlights immediately
- [ ] Start drag → handle brightens
- [ ] Release → returns to hover or idle
- [ ] No flickering during state transitions

**Multi-Select**:
- [ ] Select 3 objects, drag gizmo → all move together
- [ ] Relative positions maintained during drag
- [ ] Rotation around centroid works naturally

**Performance**:
- [ ] 60+ FPS with gizmos active in large scenes
- [ ] No lag between camera movement and gizmo updates
- [ ] Mode switch (F key) responds instantly

#### Non-Requirements (Out of Scope)

- ❌ Plane gizmos (click center for XY/YZ/XZ drag)
- ❌ Uniform scale (click center sphere to scale all axes)
- ❌ Snap indicators (visual grid lines)
- ❌ Custom gizmo styles (colors, sizes, shapes)
- ❌ Rotation rings (full 3D rotation widget)
- ❌ Touch/VR support

**See:** `in-progress/enhanced-gizmos.md` for complete specification
## Implementation Priority & Timeline

**Week 1: Asset Browser UI & Gizmo Drag Fix**
- GLB: Move asset display from hierarchy to asset browser panel
- GLB: Implement folder tree view with categories
- GLB: Add search/filter functionality
- Gizmo: Fix drag event lifecycle (press → drag → release)
- Gizmo: Begin priority system implementation

**Week 2: Error Handling & Gizmo Priority**
- GLB: Missing asset error handling (red placeholder)
- GLB: Invalid GLB file handling during scan
- Gizmo: Complete priority resolution system
- Gizmo: Visual feedback state machine

**Week 3: Material Verification & Multi-Select**
- GLB: Test and verify material preservation from Blender
- GLB: Inspector metadata display (source path, poly count)
- GLB: F5 rescan hotkey binding (optional)
- Gizmo: Multi-select transform coordination
- Gizmo: Performance optimization

**Week 4: Polish & Final Testing**
- GLB: Edge case testing (complex hierarchies, missing files)
- GLB: Documentation updates
- Gizmo: Polish and final testing
- Integration testing (GLB placement + gizmo transforms)

**Estimated Duration:** 3-4 weeks (1 month)
**Risk Buffers:** Material preservation edge cases (+2 days), picking priority complexity (+2 days)

**Key Insight**: Much of GLB support already exists (scanning, placement, persistence). Main work is UI integration and error handling.

## System Architecture

### GLB Asset Pipeline Flow

```
Startup → Scan assets/models/ → Build ModelCatalog
              ↓
         Asset Browser UI (folder tree)
              ↓
         Click Model → Enter Placement Mode
              ↓
         Load GLTF Scene (async) → Spawn Preview Ghost
              ↓
         Click Viewport → Spawn Entity + Hierarchy
              ↓
         Save Scene → Serialize Path Reference
              ↓
         Load Scene → Resolve Path → Spawn from Catalog
```

**Key Components**:
- `ModelCatalog` - Resource storing all discovered models
- `ModelInstance` - Component tracking source asset path
- `PlacementState::Model(path)` - Enum variant for model placement
- YAML format: `{type: model, path: "models/..."}`

### Gizmo Priority System

```
Mouse Click → Picking System Collects All Hits
              ↓
         Priority Resolver (Gizmos > Objects > Background)
              ↓
         Highest Priority Entity Identified
              ↓
         Interaction Systems Check Priority Guard
              ↓
         Only Relevant System Responds (Gizmo OR Selection)
```

**Key Components**:
- `PickingPriority` - Component or layer determining precedence
- Priority resource - Single source of truth for current pick
- Guard clauses in selection/gizmo systems
- Drag state tracking - Persists across frames

### Multi-Select Transform Pattern

```
Drag Start → Calculate Gizmo Center (Centroid)
              ↓
         Measure Each Object's Offset from Center
              ↓
         Store Offsets + Initial Transforms
              ↓
Drag Update → Apply Delta to Gizmo Center
              ↓
         Recompute Each Object Position (Center + Offset)
              ↓
         Apply Grid Snapping (if enabled)
              ↓
Drag End → Finalize Transforms, Clear State
```

**Why Offsets**: Prevents accumulating frame-to-frame error, maintains exact spatial relationships.

## Testing Strategy

### Unit Tests
- Model catalog building from directory structure
- Path resolution and category extraction
- Gizmo drag delta calculations
- Multi-select offset maintenance
- Grid snapping math verification

### Integration Tests
- Import GLB → spawn → inspect → save → load roundtrip
- Drag single object → verify transform changed
- Drag multi-select → verify all moved together
- Priority resolution → gizmo beats object when overlapping
- Material preservation → vertex colors match Blender

### Manual Validation
- Place 50 models from 5 categories → verify organization
- Export model from Blender → F5 rescan → place immediately
- Click gizmo handle with object behind → gizmo activates (not selection)
- Drag multi-select with grid snap → spatial relationships maintained
- Load scene with missing GLB → red placeholder appears

## Known Challenges

1. **GLTF Hierarchy Complexity**: Multi-mesh GLB files with deep nesting
   - Mitigation: Preserve parent-child structure, test with complex models early

2. **Picking Priority Edge Cases**: Multiple overlapping entities with same priority
   - Mitigation: Use distance tie-breaker (closest wins), document behavior

3. **Drag State Management**: Cursor leaves window during drag
   - Mitigation: Use `bevy_picking`'s built-in drag events (persist automatically)

4. **Material Conversion**: GLTF PBR to Bevy StandardMaterial mapping
   - Mitigation: Use Bevy's GLTF loader (already handles conversion), test with various materials

5. **Performance with Large Catalogs**: 500+ models causing slow startup
   - Mitigation: Async catalog building, on-demand scene loading, cache metadata

## Success Metrics

**Before Enhancement**:
- ❌ GLB models scanned but not visible in asset browser (hidden in hierarchy panel)
- ❌ No folder organization or search capability
- ❌ Gizmo drag success: ~30% (unreliable)
- ❌ Selection conflicts frequent (gizmo vs object)
- ❌ No error handling for missing assets

**After Enhancement**:
- ✅ Full GLB import pipeline visible in asset browser (Blender → Editor → Level)
- ✅ Folder hierarchy with search/filter
- ✅ Gizmo drag success: >99% (nearly always works)
- ✅ Selection conflicts: Zero (deterministic priority)
- ✅ Time to iterate: <60 seconds (export → F5 rescan → place)
- ✅ Catalog size: 500+ models without performance issues
- ✅ Material accuracy: 100% vertex color preservation
- ✅ Multi-select: Smooth group transforms maintaining relationships
- ✅ Error recovery: Missing assets spawn placeholders, partial scene load

## What's Next: Iteration 3

With custom model import and reliable gizmos complete, Iteration 3 will focus on:

1. **Inspector Panel Refactor** - Component-driven architecture with extensible sub-panels (foundation for all future components)

2. **Prefab System** - Reusable compound objects with override tracking and nested prefabs

3. **Advanced Snapping** - Edge-to-edge, vertex, and surface alignment for modular construction

4. **Scene Composition Tools** - Layers, grouping, hiding/locking for managing complex scenes

5. **Lighting System** - Directional, point, spot lights with shadow configuration

Essentially evolving from "placing individual assets" to "composing complex scenes" while building the extensibility foundation (inspector architecture) that will support all future editor features.

## References

- Blender's asset browser and GLTF export (industry standard workflow)
- Unity's transform handles with priority system (proven interaction model)
- Bevy's GLTF loader (`bevy_gltf`) and picking system (`bevy_mod_picking`)
- `in-progress/glb-support.md` - Complete GLB import specification
- `in-progress/enhanced-gizmos.md` - Complete gizmo enhancement specification
- `uncommitted/editor-inspector.md` - Future inspector architecture (Iteration 3+)


