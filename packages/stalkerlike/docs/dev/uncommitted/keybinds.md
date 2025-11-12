# Customizable Keybindings - Feature Specification

## Problem Statement

The current keybinding system is hardcoded throughout the editor codebase, creating several issues:

1. **No User Customization**: Users cannot remap shortcuts to match their preferred workflows or other tools (Blender, Unity, etc.)
2. **Keyboard Layout Issues**: QWERTY-centric shortcuts may not work well on AZERTY, QWERTZ, or other layouts
3. **Conflict Detection**: No mechanism to detect or prevent overlapping keybindings
4. **Scattered Logic**: Keybinding checks are duplicated across multiple systems (camera.rs, gizmo.rs, selection.rs, etc.)
5. **Documentation Drift**: Shortcuts panel must be manually updated when keybindings change in code

## Current Implementation Analysis

### What Works
- ✅ F1 opens keyboard shortcuts panel
- ✅ Comprehensive shortcut documentation in UI
- ✅ Shortcuts organized by functional category (Camera, File, Selection, Transform)
- ✅ Consistent modifier key handling (Ctrl, Shift, Alt)
- ✅ Mode-specific shortcuts (e.g., F cycles gizmo modes only when object selected)

### What's Hardcoded
Current keybindings spread across ~8 files:
- **Camera**: WASD movement, Q/E vertical, Alt mouse toggle, MMB temporary lock
- **File**: Ctrl+N/O/S/Shift+S for New/Open/Save/SaveAs
- **Selection**: Left Click, Ctrl+Click, Escape, Delete
- **Objects**: Ctrl+D duplicate, Ctrl+G group, Ctrl+Shift+G ungroup
- **Transform**: F/Shift+F gizmo cycle, O local/global, G snap toggle
- **Speed**: Shift/Ctrl speed modifiers (camera and gizmo drag)

### Why Customization Matters
- **Muscle Memory**: Users coming from Blender, Unity, Unreal have deeply ingrained shortcuts
- **Accessibility**: Physical limitations may require alternative key arrangements
- **Workflow Optimization**: Different tasks benefit from different shortcut layouts
- **International Support**: Non-QWERTY layouts need position-based or scancode options

## Proposed Solution

### Core Objectives

#### 1. Centralized Keybinding Configuration
**Goal**: Single source of truth for all editor keybindings.

**Problem**: Keybindings scattered across multiple systems make changes error-prone and documentation difficult.

**Outcome**:
- All keybindings defined in one configuration structure
- Systems query the configuration instead of checking keys directly
- Easy to see all shortcuts at a glance
- Changes propagate automatically to UI and behavior

**Why This Matters**: Maintainability and consistency. Adding new shortcuts or changing existing ones should be trivial.

#### 2. User-Customizable Mappings
**Goal**: Users can remap any shortcut to their preference.

**Problem**: Hardcoded shortcuts force users to adapt to the tool rather than tool adapting to user.

**Outcome**:
- Keybindings stored in user config file (`editor_keybinds.toml` or similar)
- GUI for remapping shortcuts (click binding → press new key → save)
- Preset options for common tools (Blender Mode, Unity Mode, Default)
- Reset to defaults option
- Import/export keybinding profiles

**Why This Matters**: Reduces friction for users switching between tools. Faster onboarding for experienced users.

#### 3. Conflict Detection & Resolution
**Goal**: System prevents or warns about overlapping keybindings.

**Problem**: User remaps Ctrl+D to "Delete" not realizing it conflicts with existing "Duplicate" binding.

**Outcome**:
- Automatic detection of conflicting bindings during configuration
- Warning UI when conflicts detected
- Suggest alternatives or require explicit conflict resolution
- Context-sensitive bindings allowed (e.g., F works differently in camera vs gizmo mode)

**Why This Matters**: Prevents user error and confusion. Clear conflict resolution builds trust.

#### 4. Automatic UI Documentation
**Goal**: Shortcuts panel always shows current keybindings, not stale hardcoded values.

**Problem**: If keybinding changes in code but shortcuts.rs isn't updated, users see incorrect info.

**Outcome**:
- Shortcuts panel queries keybinding configuration at runtime
- Menu items show current bindings dynamically (e.g., "Save (Ctrl+S)" updates if user changes it)
- No manual synchronization needed between code and documentation

**Why This Matters**: Single source of truth eliminates documentation drift.

## Design Details

### Keybinding Structure

**Action-Based System**:
- Each editor action has a unique identifier (enum or string)
- Actions map to one or more key combinations (primary + alternatives)
- Actions can have context requirements (e.g., "GizmoCycleForward" only when object selected)

**Example Actions**:
```
Camera:
  - CameraMoveForward (default: W)
  - CameraMoveBack (default: S)
  - CameraMoveLeft (default: A)
  - CameraMoveRight (default: D)
  - CameraMoveUp (default: E)
  - CameraMoveDown (default: Q)
  - CameraToggleMouseLock (default: Alt)
  - CameraSpeedBoost (default: Shift, modifier)
  - CameraSpeedSlow (default: Ctrl, modifier)

File:
  - FileNew (default: Ctrl+N)
  - FileOpen (default: Ctrl+O)
  - FileSave (default: Ctrl+S)
  - FileSaveAs (default: Ctrl+Shift+S)

Selection:
  - SelectObject (default: Left Click)
  - SelectMultiAdd (default: Ctrl+Left Click)
  - DeselectAll (default: Escape)
  - DeleteSelected (default: Delete)

Objects:
  - DuplicateSelected (default: Ctrl+D)
  - GroupSelected (default: Ctrl+G)
  - UngroupSelected (default: Ctrl+Shift+G)

Transform:
  - GizmoCycleForward (default: F)
  - GizmoCycleBackward (default: Shift+F)
  - GizmoToggleOrientation (default: O)
  - GridSnapToggle (default: G)
  - TransformSpeedBoost (default: Shift, modifier)
  - TransformSpeedSlow (default: Ctrl, modifier)

Help:
  - ShowShortcuts (default: F1)
```

### Key Representation

**Physical vs Logical**:
- **Logical Keys**: Based on character (e.g., "W" for forward regardless of keyboard layout)
- **Physical Keys**: Based on position (e.g., "key in QWERTY W position" works on AZERTY)
- **Recommendation**: Default to logical for letters/numbers, physical for arrow keys/function keys

**Modifier Handling**:
- Ctrl, Shift, Alt, Super/Cmd treated as modifiers
- "Any Ctrl" matches either left or right Ctrl key
- Support for order-independent combos (Ctrl+Shift+S = Shift+Ctrl+S)

**Special Cases**:
- Mouse buttons (Left, Right, Middle, Extra1, Extra2)
- Modifier-only bindings (e.g., "hold Shift" for speed boost)
- Context-dependent bindings (same key, different action based on mode)

### Configuration Storage

**File Format**: TOML for human readability and comments
```toml
# Stalkerlike Editor Keybindings
# Edit this file to customize shortcuts
# Restart editor or reload config for changes to take effect

[camera]
move_forward = "W"
move_back = "S"
move_left = "A"
move_right = "D"
move_up = "E"
move_down = "Q"
toggle_mouse_lock = "Alt"
speed_boost = "Shift"  # modifier
speed_slow = "Ctrl"    # modifier

[file]
new = "Ctrl+N"
open = "Ctrl+O"
save = "Ctrl+S"
save_as = "Ctrl+Shift+S"

[selection]
select = "LeftClick"
multi_select = "Ctrl+LeftClick"
deselect_all = "Escape"
delete = "Delete"

[objects]
duplicate = "Ctrl+D"
group = "Ctrl+G"
ungroup = "Ctrl+Shift+G"

[transform]
gizmo_cycle_forward = "F"
gizmo_cycle_backward = "Shift+F"
gizmo_toggle_orientation = "O"
grid_snap_toggle = "G"
speed_boost = "Shift"  # modifier
speed_slow = "Ctrl"    # modifier

[help]
show_shortcuts = "F1"
```

**Location**: 
- User config: `~/.config/stalkerlike/editor_keybinds.toml` (Linux/Mac)
- User config: `%APPDATA%/stalkerlike/editor_keybinds.toml` (Windows)
- Fallback: Built-in defaults if file missing or malformed

### Preset Profiles

**Blender Mode**:
```
- Camera: Same WASD + Q/E
- Duplicate: Shift+D (not Ctrl+D)
- Delete: X (not Delete key)
- Transform: G (move), R (rotate), S (scale) + axis lock (X/Y/Z)
- Select: Right Click (historical Blender default) or Left Click (modern)
```

**Unity Mode**:
```
- Camera: WASD + Q/E (similar)
- Scene navigation: Alt+Drag, Alt+RMB
- Focus: F (not gizmo cycle)
- Hand tool: Q
- Transform: W (move), E (rotate), R (scale)
```

**Default Mode**:
- Current Stalkerlike bindings
- QWERTY-optimized
- Ergonomic for long editing sessions

### Conflict Resolution

**Validation Rules**:
1. **No duplicate global bindings**: Two actions can't have same key combo unless context-dependent
2. **Modifier conflicts**: Warn if Ctrl+S and S are both bound (Ctrl+S won't fire if S triggers first)
3. **Action conflicts**: Warn if Delete bound to multiple actions (e.g., "Delete Object" and "Delete Component")

**Resolution UI**:
```
⚠️ Keybinding Conflict Detected

Action 1: "Duplicate Selected" → Ctrl+D
Action 2: "Delete Selected" → Ctrl+D

Resolve:
[ ] Change "Duplicate Selected" to: [_______]
[ ] Change "Delete Selected" to: [_______]
[ ] Keep both (context-dependent - expert mode)
[Cancel] [Apply]
```

### Rebinding UI

**Simple Workflow**:
1. Open Settings → Keybindings (or Help → Customize Shortcuts)
2. See table: Action | Current Binding | Category
3. Click binding to edit
4. Modal: "Press new key combination or Escape to cancel"
5. Display conflicts if any, require resolution
6. Click "Save" to persist changes
7. Optionally restart editor or hot-reload config

**Advanced Features** (optional):
- Search/filter actions by name
- Show conflicting bindings highlighted
- Export profile as `.toml` file
- Import profile from file
- Quick presets dropdown (Default, Blender, Unity)

## Implementation Strategy

### Phase 1: Centralization (Foundation)
**Goal**: Refactor existing hardcoded bindings to use central configuration.

**Work**:
1. Define `KeybindingAction` enum for all editor actions
2. Create `KeybindingConfig` resource with default mappings
3. Create `KeybindingService` to query bindings (e.g., `is_action_pressed()`)
4. Refactor camera.rs to use service instead of direct key checks
5. Refactor other systems one by one
6. Ensure no behavior changes (validation pass)

**Outcome**: All keybindings in one place, behavior unchanged, groundwork laid.

### Phase 2: Persistence (Configuration)
**Goal**: Load/save keybindings from user config file.

**Work**:
1. Implement TOML serialization for `KeybindingConfig`
2. Add config file loading on startup (with fallback to defaults)
3. Add config file saving on changes
4. Handle malformed config gracefully (warning + fallback)
5. Test cross-platform config paths

**Outcome**: Users can manually edit config file and see changes on restart.

### Phase 3: UI (Customization)
**Goal**: GUI for remapping shortcuts without editing files.

**Work**:
1. Create keybinding editor panel (EGUI window)
2. Display current bindings in table format
3. Implement "click to rebind" modal
4. Add conflict detection and resolution UI
5. Add preset dropdown (Default, Blender, Unity)
6. Add import/export functionality

**Outcome**: Users can customize keybindings entirely through UI.

### Phase 4: Polish (Nice-to-Have)
**Goal**: Refinements and advanced features.

**Work**:
1. Hot-reload config without restart
2. Per-context bindings (game mode vs editor mode)
3. Macro/sequence support (e.g., press G then X for "move on X axis")
4. Key chords (e.g., Ctrl+K Ctrl+S like VS Code)
5. Mouse gesture support (optional)
6. Gamepad binding support (optional)

**Outcome**: Production-quality keybinding system rivaling professional tools.

## Technical Considerations

### Bevy Input Handling
- Use `ButtonInput<KeyCode>` for keyboard
- Use `ButtonInput<MouseButton>` for mouse
- Check pressed/just_pressed/just_released as needed
- Handle modifier keys (Ctrl/Shift/Alt) explicitly

### Context Sensitivity
Some actions only valid in certain contexts:
- Gizmo cycle only when object selected
- Camera shortcuts only in editor mode (not play mode)
- File operations blocked during modal dialogs

**Solution**: Actions can specify required context. Service checks context before allowing action.

### Performance
- Keybinding queries happen every frame for active shortcuts
- Must be fast (avoid string comparisons, use enum matching)
- Cache commonly-used lookups
- Lazy evaluation where possible

### Accessibility
- Support for single-handed operation where feasible
- Avoid reliance on simultaneous multi-key presses (RSI concerns)
- Allow remapping to accessibility devices (foot pedals, voice commands via external tools)

## User Stories

### Story 1: Blender User Onboarding
**As a** Blender user learning Stalkerlike,
**I want to** use Blender-style shortcuts (Shift+D duplicate, X delete),
**So that** I don't have to relearn muscle memory.

**Acceptance**:
- User opens Settings → Keybindings
- Selects "Blender Mode" preset from dropdown
- All shortcuts update to Blender-style
- Shortcuts panel reflects new bindings
- Workflow feels familiar immediately

### Story 2: Keyboard Layout Compatibility
**As a** French AZERTY user,
**I want** WASD camera controls to work on my layout,
**So that** I don't have to remap physical keys.

**Acceptance**:
- User on AZERTY layout loads editor
- Detects ZQSD as appropriate for physical position
- OR user remaps to ZQSD manually
- Camera movement feels natural
- No awkward key positions

### Story 3: Custom Workflow Optimization
**As a** level designer,
**I want to** create a custom keybinding set for rapid prototyping,
**So that** I can work faster during iteration.

**Acceptance**:
- User remaps frequently-used actions to number keys (1-9)
- Saves custom profile as "rapid_proto.toml"
- Shares profile with team
- Team imports profile and uses same shortcuts
- Consistency across designers

## Out of Scope

**Explicitly NOT in scope for initial implementation**:
- Per-project keybinding overrides (global config only)
- Key recording/macro system
- Mouse gesture recognition
- Touchscreen/tablet gesture support
- Voice command integration
- Network-synced configs (cloud settings)
- Undo/redo for keybinding changes (file-based undo sufficient)

These may be reconsidered in future iterations based on user feedback.

## Success Metrics

**Qualitative**:
- Users can successfully remap shortcuts without reading documentation
- Blender/Unity users report "feels natural" when using presets
- Zero conflict-related support tickets after conflict detection implemented

**Quantitative**:
- Config file load time <10ms (imperceptible)
- Keybinding query time <1μs (frame budget negligible)
- 95%+ of users stick with defaults (good defaults) OR 50%+ customize (customization desired)

## Related Work

**Similar Systems**:
- Blender: Extensive keybinding customization, presets, export/import
- VS Code: Keybindings JSON, command palette, per-language overrides
- Unreal Editor: Input bindings, modifier chains, context-sensitive
- Unity: Some customization, but more limited than competitors

**Lessons**:
- Clear conflict detection is critical (Blender's system is excellent)
- Defaults matter more than customization options (VS Code nails this)
- UI should feel approachable, not overwhelming (Unity's simplicity is good)
- Export/import enables team standardization (Blender's keymaps)

## Next Steps

**Before Implementation**:
1. Review with users/testers for feedback on proposed system
2. Validate TOML format with sample configs
3. Prototype rebinding UI in Figma or similar
4. Define full action enum (exhaustive list of all editor actions)

**Implementation Order**:
1. Phase 1 (Centralization) - Iteration 2 or 3
2. Phase 2 (Persistence) - Same iteration as Phase 1
3. Phase 3 (UI) - Iteration 3 or 4
4. Phase 4 (Polish) - Iteration 5+ or based on feedback

**Dependencies**:
- Settings/Preferences system (where to put keybinding editor)
- Config file loading infrastructure (may need to build)
- EGUI table/list UI components (check if existing or need custom)

## Appendix: Full Action Enumeration

**Camera** (10 actions):
- CameraMoveForward, CameraMoveBack, CameraMoveLeft, CameraMoveRight
- CameraMoveUp, CameraMoveDown
- CameraToggleMouseLock, CameraTemporaryMouseLock
- CameraSpeedBoost, CameraSpeedSlow

**File** (4 actions):
- FileNew, FileOpen, FileSave, FileSaveAs

**Selection** (4 actions):
- SelectObject, SelectMultiAdd, DeselectAll, DeleteSelected

**Objects** (3 actions):
- DuplicateSelected, GroupSelected, UngroupSelected

**Transform** (6 actions):
- GizmoCycleForward, GizmoCycleBackward
- GizmoToggleOrientation
- GridSnapToggle
- TransformSpeedBoost, TransformSpeedSlow

**Help** (1 action):
- ShowShortcuts

**Total**: 28 actions (current editor MVP)

**Future** (post-MVP):
- Play mode shortcuts (Play, Pause, Step, Stop)
- Viewport shortcuts (Toggle grid, Toggle gizmo, Frame selected)
- History shortcuts (Undo, Redo)
- Component shortcuts (Add component, Remove component)
- Tool shortcuts (Box select, Paint mode, Vertex edit)

**Grand Total**: 40-50 actions for full-featured editor
