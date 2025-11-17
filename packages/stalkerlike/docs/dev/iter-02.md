# Iteration 02 - GLB Support and Enhanced Gizmos

## Overview

**Problem**: Two critical blockers prevent productive level design:
1. GLB models work but won't scale (no search, no hierarchy, minimal UI - usable for <10 models, breaks at 50+)
2. Gizmo drag operations fail ~70% of the time (unreliable interaction)

**Solution**: Build proper asset browser with search/hierarchy for GLB catalog + fix gizmo picking/drag lifecycle.

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

### GLB Support

#### Implementation Tasks

**Asset Browser UI Integration** (Critical)
- [ ] Refactor `asset_browser.rs` to show primitives + models in separate collapsible sections
- [ ] Build folder tree from `AssetBrowserState.glb_assets` using `relative_path`
- [ ] Group assets by directory (e.g., "pipes/", "industrial/", "structural/")
- [ ] Add expand/collapse arrows for categories
- [ ] Click asset → call existing `start_placement_asset()` with `PlacementAsset::GlbModel`
- [ ] Remove GLB display from hierarchy panel (keep scanning logic)
- [ ] Test: 50 models across 5 folders → all appear organized in asset browser

**Search & Filter** (High Priority)
- [ ] Add search bar above asset sections in asset browser
- [ ] Filter `asset_catalog.primitives` and `asset_browser_state.glb_assets` by name
- [ ] Case-insensitive matching, update on text change
- [ ] Clear button resets filter
- [ ] Check asset existence before spawning: `asset_server.get_load_state()`
- [ ] On load failure: spawn red cube + `MissingAsset` marker + log error
- [ ] Test: Delete GLB → load scene → red placeholder appears with "MISSING: {path}"

**Inspector Metadata & Material Verification** (Medium Priority)
- [ ] Detect `GlbModel` component in inspector system
- [ ] Display metadata section: source path (read-only), poly count, submesh count
- [ ] Extract poly count from mesh asset via `Mesh::count_vertices()`
- [ ] Export test model with red vertex colors → place → verify color preserved (existing system)
- [ ] Export test model with emissive material → place → verify glow preserved (existing system)
- [ ] Test alpha transparency (glass material) with existing placement
- [ ] Bind F5 key → `asset_browser_state.scan_glb_files()` → log results (optional)
- [ ] Document any material limitations found

---

### Enhanced Gizmos

#### Implementation Tasks

**Drag Interaction** (Critical)
- [ ] Add picking mesh colliders to gizmo handles (currently visual-only)
- [ ] Implement `Pointer<Down>` observer → store initial transform + drag start position
- [ ] Implement `Pointer<Drag>` observer → calculate delta from start, apply to transform
- [ ] Implement `Pointer<Up>` observer → finalize transform, clear drag state
- [ ] Ensure drag continues if cursor leaves handle (use frame-persistent state)
- [ ] Add debug logging: "Drag started on {axis}", "Drag delta: {distance}", "Drag ended"
- [ ] Test: Click X arrow → drag right → object moves smoothly, release → movement stops
- [ ] Test: Drag off-screen → release outside window → still finalizes correctly

**Picking Priority & Visual Feedback** (High Priority)
- [ ] Research `bevy::picking` priority features (PickingBehavior, picking layers)
- [ ] If built-in insufficient: implement priority filter examining all raycast hits
- [ ] Assign high priority to gizmo handles, lower to scene objects
- [ ] Selection system checks priority before deselecting/selecting
- [ ] Gizmo system checks priority before activating drag
- [ ] Test: Click gizmo overlapping object → gizmo activates, no selection change
- [ ] Enhance GizmoMaterial with hover/active states (currently uses single material)
- [ ] Update existing `on_gizmo_hover` observer to brighten handle
- [ ] Update existing `on_gizmo_hover_end` observer to restore color
- [ ] Add active state on drag start (maximize brightness)
- [ ] Add state restoration on drag end
- [ ] Test: Hover handle → highlights, move away → unhighlights, drag → brighter, release → restores

**Multi-Select & Performance** (Medium Priority)
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

## Success Criteria

**GLB Support Complete When**:
- Asset browser handles 100+ models with folder hierarchy and search
- Click asset → place workflow fully functional
- Materials (vertex colors, PBR) work correctly from Blender exports
- Missing assets handled gracefully (red placeholder, partial scene load)

**Gizmo Enhancement Complete When**:
- Drag success rate >99%
- Zero selection conflicts (gizmos always win)
- Visual feedback clear and consistent
- Multi-select transforms maintain spatial relationships
- 60+ FPS performance maintained

**Iteration Complete When**:
- All testing checklist items pass
- No critical bugs or workflow blockers

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

## Implementation Dependencies

**Critical Path**:
1. Asset Browser UI (enables model discovery)
2. Drag Lifecycle (foundation for all gizmo work)
3. Search/Filter + Priority System (parallel - no dependencies)
4. Error Handling + Visual Feedback (parallel - no dependencies)
5. Inspector Metadata + Multi-Select (can be done last)

**Parallelization Opportunities**:
- GLB search/filter and gizmo priority system can be developed simultaneously
- Error handling and visual feedback are independent
- Material verification can happen alongside other GLB work

**Key Insight**: Much of GLB support already exists (scanning, placement, persistence). Main work is UI integration and error handling.

## Technical Architecture

### GLB Asset Pipeline
- Startup: Scan `assets/models/` directory recursively
- Asset Browser: Display folder hierarchy with search/filter
- Placement: Load GLTF scene asynchronously, spawn preview ghost
- Persistence: Serialize path references in YAML (not geometry)
- Loading: Resolve paths, spawn from asset server, handle missing files

**Key Components**: `AssetBrowserState` (hierarchy.rs), `GlbModel` component (types.rs), `PlacementAsset::GlbModel` enum, `ComponentData::GlbModel` (scene.rs YAML)

### Gizmo Priority System
- Picking collects all raycast hits
- Priority resolver applies hierarchy: Gizmos > UI > Scene Objects > Background
- Systems check priority guard before responding
- Drag state persists across frames regardless of cursor position

**Key Components**: `GizmoHandle` marker, `GizmoRoot` marker, `GizmoAxis` enum, `GizmoState` resource (mode/orientation), `GizmoMaterial` asset, entity observers (on_gizmo_drag, on_gizmo_hover)

### Multi-Select Transform
- Calculate gizmo center as selection centroid
- Store per-object offsets from center on drag start
- Apply deltas to center, recompute object positions maintaining offsets
- Grid snapping applied to center (preserves relative positioning)

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

## References

- **Current Implementation**: 
  - `packages/stalkerlike/src/editor/ui/hierarchy.rs` - GLB scanning (AssetBrowserState)
  - `packages/stalkerlike/src/editor/ui/asset_browser.rs` - Asset browser UI (primitives only)
  - `packages/stalkerlike/src/editor/objects/placement.rs` - Placement system
  - `packages/stalkerlike/src/editor/objects/gizmo/` - Transform gizmos (types, systems, observers)
  - `packages/stalkerlike/src/editor/persistence/scene.rs` - YAML serialization
- **Specifications**:
  - `in-progress/glb-support.md` - Complete GLB feature spec
  - `in-progress/enhanced-gizmos.md` - Complete gizmo enhancement spec
- **Future Work**: 
  - Iteration 3: Inspector refactor, prefabs, advanced snapping, lighting


