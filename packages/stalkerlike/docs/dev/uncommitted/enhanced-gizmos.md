# Enhanced Gizmos - Feature Specification

## Problem Statement

The current gizmo implementation has several critical issues that make it frustrating to use:

1. **Drag Interaction Broken**: Click-and-drag on gizmo handles is unreliable or non-functional
2. **Selection Priority Issues**: Clicking gizmos sometimes triggers object selection instead of gizmo interaction
3. **Visual Feedback Inconsistent**: Hover state changes are not reliable
4. **Performance**: Gizmo updates may lag behind camera or selection changes
5. **Multi-select Behavior**: Gizmos don't properly handle multiple selected objects

## Current Implementation Analysis

### What Works
- ✅ Gizmo spawning/despawning via observers (`OnAdd<Selected>`, `OnRemove<Selected>`)
- ✅ F key cycling between Translate/Rotate/Scale modes
- ✅ O key toggling Global/Local orientation
- ✅ Basic arrow visualization (RGB color scheme)
- ✅ Distance-based scaling for constant screen-space size
- ✅ Grid snapping integration (respects `GridConfig`)
- ✅ Speed modifiers (Shift = 4x, Ctrl = 0.25x)

### What's Broken
- ❌ `Pointer<Drag>` events not firing reliably on gizmo handles
- ❌ Gizmo handles lack proper picking volumes (using parent entity transform instead of child meshes)
- ❌ Hover highlighting (emissive changes) doesn't always restore properly
- ❌ Multi-select gizmo positioning at center but transforms not batched correctly
- ❌ Gizmo handles can conflict with selection raycasting

## Proposed Solution

### Core Objectives

#### 1. Reliable Click Targets
**Goal**: Users can easily click gizmo handles without frustration.

**Problem**: Thin arrow geometry is hard to target with precision. Current meshes are visually clear but functionally difficult to click.

**Outcome**: 
- Gizmo handles have generous click areas (significantly larger than visual appearance)
- Users can click near the handle and still engage it
- Click success rate approaches 100% under normal conditions
- No need for pixel-perfect precision

**Why This Matters**: Frustrating click interactions break flow state and make the editor feel unresponsive.

#### 2. Continuous Drag Tracking
**Goal**: Dragging feels smooth and responsive throughout the entire motion.

**Problem**: Drag operations may lose tracking mid-motion, especially if the cursor moves quickly or leaves the handle area.

**Outcome**:
- Drag persists from mouse-down to mouse-up regardless of cursor position
- Smooth, continuous feedback as objects move
- No "jumps" or "stutters" in movement
- Drag works even if cursor goes off-screen during motion
- Transform updates every frame with consistent delta calculations

**Why This Matters**: Broken drag tracking makes precise positioning impossible and creates a feeling of broken interaction.

**Technical Note**: Prefer `bevy_picking`'s built-in drag event system rather than manual mouse tracking.

#### 3. Unambiguous Interaction Priority
**Goal**: Gizmos always take precedence over scene objects when both are under the cursor.

**Problem**: When a gizmo handle overlaps with a scene object from the camera's view, clicking can select the object instead of engaging the gizmo.

**Outcome**:
- Clicking a gizmo handle **never** selects or deselects objects
- Clicking an object **never** triggers gizmo interaction
- When both are under cursor, gizmo wins every time
- Priority resolution is deterministic (same click always produces same result)
- Empty space clicks still work normally (deselection)

**Why This Matters**: Ambiguous interaction creates unpredictability and erodes user trust in the tool.

**Priority Hierarchy** (clearest wins):
1. Gizmo handles (highest)
2. UI elements (handled separately by UI framework)
3. Scene objects (editor entities)
4. Background/ground plane (lowest)

**Technical Note**: Prefer `bevy_picking`'s built-in priority system. If insufficient, implement priority filtering that examines all raycast hits and deterministically selects the highest priority target.

#### 4. Clear Visual Feedback
**Goal**: User always knows what they can interact with and what state the gizmo is in.

**Problem**: Without visual cues, users don't know if they're hovering a handle or if a drag is active.

**Outcome**:
- Hovering a gizmo handle produces immediate visual response (color shift, highlight)
- Active drag state is visually distinct from idle/hover
- Different gizmo types (translate/rotate/scale) are immediately recognizable
- Feedback is instant (< 16ms latency from cursor movement)
- State transitions are smooth, not jarring

**Why This Matters**: Visual feedback provides affordance—users need to know what's clickable and when they're in control.

**State Indicators**:
- **Idle**: Default appearance, clearly distinguishable from scene objects
- **Hover**: Brightness/saturation increase, indicates interactivity
- **Active Drag**: Maximum visual prominence, confirms user is in control
- **Disabled**: Dimmed/grayed when gizmo shouldn't respond (e.g., no selection)

**Technical Note**: Prefer `bevy_picking`'s hover events for state detection. Update material properties or shader parameters to drive visual changes.

#### 5. Cohesive Multi-Select Transforms
**Goal**: Moving/rotating/scaling multiple objects maintains their spatial relationships perfectly.

**Problem**: Applying transforms to multiple objects independently causes drift—objects move apart or overlap unexpectedly.

**Outcome**:
- Selected objects move as a single rigid group
- Relative positions between objects stay constant during drag
- Rotation pivots around the selection's center, not individual origins
- Scaling maintains group proportions
- Works identically for 2, 10, or 100 selected objects

**Why This Matters**: Breaking spatial relationships destroys user intent. If objects are arranged meaningfully, transforms must preserve that arrangement.

**Transform Behaviors**:
- **Translation**: All objects move by identical delta, maintaining exact spacing
- **Rotation**: Group rotates around shared centroid, preserving relative positions
- **Scaling**: Objects move toward/away from centroid proportionally

**Technical Note**: Calculate offsets from gizmo center on drag start, maintain those offsets throughout the operation.

**Grid Snapping**:
- Apply snapping to gizmo center position
- Offsets remain unchanged (maintains group cohesion)
- Alternative: snap each object individually (may break alignment)

## Implementation Plan

### Phase 1: Fix Drag Interaction (Critical) 🔴
**Goal**: Make gizmos reliably draggable

**Approach**: Properly integrate with drag event lifecycle to maintain state across frames.

- [ ] Verify gizmo handles participate in picking
- [ ] Observe press events to initialize drag state
- [ ] Observe continuous drag events to apply transforms
- [ ] Observe release events to finalize drag
- [ ] Store initial transforms on drag start
- [ ] Calculate transform deltas from drag start position (not frame-to-frame)
- [ ] Ensure drag continues even if cursor leaves handle
- [ ] Add debug logging for drag lifecycle (press → drag → release)
- [ ] Test with single object selection
- [ ] Test with multi-object selection
- [ ] Test dragging off-screen and releasing

**Key Insight**: Drag tracking across frames is already provided by the framework—just observe the events properly.

**Success Criteria**:
- Press fires reliably when clicking gizmo handles
- Drag fires continuously while dragging (regardless of cursor position)
- Release fires reliably when releasing mouse button
- Drag delta is smooth and predictable
- Grid snapping works during drag
- Debug logs show complete event sequence for every drag operation

### Phase 2: Implement Picking Priority (High Priority) 🟡
**Goal**: Gizmo clicks never trigger object selection

**Approach**: Use `bevy_picking`'s built-in priority features first, add custom filtering only if needed.

- [ ] Test if built-in priority system resolves conflicts (gizmo vs object clicks)
- [ ] Add generous click volumes to gizmo handles for easier targeting
- [ ] If needed: implement priority filtering that examines all hits
- [ ] If needed: modify selection to check priority before proceeding
- [ ] If needed: modify gizmo observers to check priority before proceeding
- [ ] Test clicking gizmo vs clicking object (gizmo should always win)
- [ ] Test clicking overlapping objects (correct priority selection)
- [ ] Test clicking empty space (normal selection behavior)
- [ ] Add debug logging to verify priority resolution

**Decision Point**: Start with framework features. Only add custom system if defaults insufficient.

**Success Criteria**:
- Gizmo handles always take priority over scene objects (no conflicts)
- Can click objects that don't overlap gizmos (normal selection works)
- Clicking empty space deselects (normal behavior preserved)
- System integrates cleanly with framework (no fighting it)
- Debug logs show which entity won and why (if custom system added)

### Phase 3: Improve Visual Feedback (Medium Priority) 🟢
**Goal**: Clear visual indication of gizmo state

- [ ] Store color states for each handle (base, hover, active, disabled)
- [ ] Implement proper state machine (Idle → Hovered → Dragging)
- [ ] Brighten active axis during drag (not just on hover)
- [ ] Dim inactive axes during single-axis drag (optional)
- [ ] Add smooth color transitions (lerp over ~0.1s)
- [ ] Restore original colors on drag end
- [ ] Test hover → drag → release → hover sequence

**Success Criteria**:
- Hovering a handle highlights it (emissive increase)
- Dragging a handle makes it brighter (active state)
- Releasing returns to hover state (if still hovering)
- Moving mouse away returns to idle state
- No color flickering or state confusion

### Phase 4: Refine Multi-Select (Low Priority) 🔵
**Goal**: Multi-select transforms feel natural

- [ ] Calculate gizmo center as centroid of selection
- [ ] Store per-object offsets from gizmo center on drag start
- [ ] Apply transforms maintaining relative positions
- [ ] Test rotation around multi-select center
- [ ] Test scaling from multi-select center (optional)
- [ ] Verify grid snapping works for all objects in selection

**Success Criteria**:
- Moving multi-select gizmo moves all objects as a rigid group
- Rotating around gizmo center rotates group naturally
- Grid snapping doesn't break relative positioning

### Phase 5: Performance & Polish (Nice-to-Have) ⚪
**Goal**: Buttery smooth gizmo interaction

- [ ] Profile gizmo update systems (find bottlenecks)
- [ ] Optimize material updates (batch queries)
- [ ] Add interpolation to gizmo position updates (smooth lag)
- [ ] Consider using instancing for arrow meshes (reduce draw calls)
- [ ] Add subtle animation on mode switch (arrows morph)
- [ ] Add visual "snap" feedback when grid snapping occurs
- [ ] Polish arrow geometry (rounded caps, smoother cones)

**Success Criteria**:
- Gizmos update at 60+ FPS with large scenes
- No visible lag between camera movement and gizmo position
- Mode switching feels polished (not jarring)

## Technical Deep Dive

### Conceptual Architecture

**Core Challenge**: When a user clicks, multiple entities (gizmo handle, scene object, background) might be under the cursor. The system must consistently choose the most relevant one.

**Solution Approach**: Prioritize all potential click targets, resolve to the highest priority entity before any interaction systems run. This acts as a "traffic controller" for mouse events.

**System Flow**:
1. User clicks
2. Picking system collects all entities under cursor
3. Priority resolver evaluates all candidates
4. Highest priority entity is identified as "the pick"
5. Interaction systems check priority before proceeding
6. Only relevant system responds (gizmo OR selection, never both)

**Why This Works**: 
- Single source of truth (one resource holds the winner)
- Early resolution (before any observers fire)
- Domain separation (each system checks priority guard clause)
- Extensible (new priorities slot in naturally)

**Technical Note**: Prefer `bevy_picking`'s built-in features (`PickingBehavior`, layer ordering) first. Implement custom priority resolution only if needed.

### Drag Event Lifecycle

**Expected Behavior**:
1. **Press** mouse on handle → Initialize drag (snapshot transforms)
2. **Move** mouse while held → Continuous updates (apply deltas)
3. **Release** mouse → Finalize drag (cleanup state)

**Critical Requirement**: Drag must continue even if cursor leaves the original entity or exits the window. Users shouldn't have to keep cursor perfectly over the handle.

**Technical Note**: Prefer `bevy_picking`'s drag events—they already provide frame deltas and persist across cursor movement.

### Multi-Object Transform Coordination

**Challenge**: Applying transforms independently causes objects to drift apart or collide unexpectedly.

**Conceptual Solution**:
1. Calculate gizmo position as center of selection
2. On drag start, measure each object's offset from center
3. During drag, maintain those exact offsets
4. For rotation, pivot around center while preserving spatial relationships

**Why Offsets Matter**: Without storing initial positions, recalculating from frame-to-frame deltas accumulates error. Offsets provide a stable reference.

### Visual Feedback State Machine

**States**: Idle → Hover → Active → (back to Idle or Hover)

**Triggers**:
- Mouse enters handle → Idle to Hover
- Mouse exits handle → Hover to Idle
- Mouse pressed on handle → Hover to Active
- Mouse released → Active to Hover (if still over) or Idle (if left)

**Visual Changes**: Brightness, emissive color, or saturation shifts communicate state.

**Why State Machines**: Prevents state confusion (e.g., stuck "hovered" after drag). Each state has explicit transitions.

## Testing Strategy

### Unit Tests
- Transform delta calculations (given drag distance, calculate correct world movement)
- Grid snapping math (verify positions snap to nearest grid points)
- Multi-select offset maintenance (verify relative positions preserved)

### Integration Tests
- Drag single object → verify transform changed
- Drag multi-select → verify all objects moved together
- Grid snap during drag → verify snapped positions
- Mode switch during drag → verify drag cancelled
- Orientation toggle during drag → verify drag continues correctly

### Manual Test Cases
1. **Basic Translate**: Click X-axis arrow, drag right → object moves right
2. **Grid Snap**: Enable grid (G key), drag handle → snaps to 0.5m increments
3. **Multi-Select Drag**: Select 3 objects, drag gizmo → all move together
4. **Hover Feedback**: Hover over handle → highlights, move away → unhighlights
5. **Drag Feedback**: Start dragging → handle brightens, release → returns to hover
6. **Mode Switch**: Press F during drag → drag cancels, mode switches
7. **Orientation Switch**: Press O → gizmo rotates to local axes (if 1 object selected)
8. **Speed Modifiers**: Hold Shift during drag → moves 4x faster, Ctrl → 0.25x slower
9. **Selection Priority**: Click gizmo handle → does NOT deselect/reselect object
10. **Edge Case**: Drag off-screen, release outside window → still ends drag correctly
11. **🆕 Overlapping Picks**: Click where gizmo and object overlap → gizmo takes priority
12. **🆕 Priority Logging**: Enable debug logs → see priority resolution for each click
13. **🆕 Multiple Gizmos**: Select multiple objects → single gizmo at centroid, all transform together
14. **🆕 Priority Fallback**: Click entity without explicit priority → gets inferred from components
15. **🆕 Distance Tie-Breaking**: Click overlapping objects (same priority) → closest one selected

## Success Metrics

**Before Enhancement** (Current State):
- ❌ Drag success rate: ~30% (unreliable)
- ❌ Selection conflicts: Frequent (gizmo clicks select objects)
- ❌ Visual feedback: Inconsistent
- ❌ Multi-select: Broken/weird behavior
- ❌ Priority resolution: Non-existent (first hit wins)

**After Enhancement** (Target):
- ✅ Drag success rate: >99% (nearly always works)
- ✅ Selection conflicts: **Zero** (deterministic priority system)
- ✅ Visual feedback: Consistent and clear
- ✅ Multi-select: Smooth group transformations
- ✅ Performance: 60+ FPS with gizmos active
- ✅ User confidence: Can rely on gizmos without worrying about jank
- ✅ Priority resolution: Deterministic (gizmos always beat objects)
- ✅ Debug visibility: Can inspect system state to understand behavior

## Future Enhancements (Out of Scope)

These are nice-to-haves for later iterations:

- **Plane Gizmos**: Click center square to drag on XY/YZ/XZ planes
- **Uniform Scale**: Click center sphere to scale all axes together
- **Snap Indicators**: Visual grid lines appear during snap
- **Custom Gizmo Styles**: User-configurable colors, sizes, shapes
- **Gizmo Widgets**: Rotation rings, scale boxes, custom tools
- **Touch Support**: Multi-touch gestures for transform (if relevant)
- **VR Support**: Gizmos in 3D space for VR editing (far future)

## References

- **Blender Gizmos**: Industry standard for 3D transform tools
- **Unity Transform Handles**: Similar interaction model
- **Bevy Picking Docs**: https://docs.rs/bevy/latest/bevy/picking/
- **Current Implementation**: `packages/stalkerlike/src/editor/objects/gizmo.rs`

## Acceptance Criteria

This feature is complete when:

1. ✅ Click-and-drag works reliably on all gizmo handles (>99% success rate)
2. ✅ Gizmo interactions never conflict with object selection
3. ✅ Visual feedback (hover/active states) is clear and consistent
4. ✅ Multi-select transforms work smoothly (objects move as group)
5. ✅ Grid snapping works during gizmo drags
6. ✅ Speed modifiers (Shift/Ctrl) work during drag
7. ✅ Manual testing passes all test cases above
8. ✅ No performance regressions (60+ FPS maintained)
9. ✅ Code is well-documented with inline comments
10. ✅ Future maintainers can understand and extend the gizmo system

---

**Priority**: 🔴 **Critical** - Gizmos are essential for editor usability  
**Estimated Effort**: 2-3 days (Phase 1-2), 1 day (Phase 3-4), ongoing (Phase 5)  
**Dependencies**: None (self-contained enhancement)  
**Risk**: Low (well-understood problem with clear solutions)
