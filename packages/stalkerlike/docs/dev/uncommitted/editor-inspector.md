# Editor Inspector Panel - Extensible Architecture

## Overview

The inspector panel needs to be refactored from its current monolithic structure into a component-driven system where each ECS component type can register its own sub-panel with custom UI logic. This enables:
- Adding new inspectable components without modifying core inspector code
- Per-component validation and constraints
- Custom widgets for specialized data types
- Dynamic add/remove of components at runtime

## Core Architecture

### Component Registration System

Each component type registers an inspector panel definition that describes:
- **Component Type ID**: The Rust type being inspected (Transform, Collider, etc.)
- **Panel Renderer**: Function/closure that draws UI for this component
- **Add Handler**: Logic for adding this component to an entity
- **Remove Handler**: Logic for removing this component from an entity
- **Dependencies**: Required components (e.g., Collider requires Transform)
- **Conflicts**: Mutually exclusive components
- **Display Priority**: Order in inspector (core components first)

### Inspector Panel Structure

```
┌─ Inspector ──────────────────────────┐
│ Selected Object: "Cube_01"           │
│ ────────────────────────────────────│
│ [Core Components]                    │
│   ▼ Transform                        │
│   ▼ Mesh                             │
│   ▼ Material                         │
│ ────────────────────────────────────│
│ [Physics Components]                 │
│   ▼ RigidBody                        │
│   ▼ Collider                         │
│   ▶ Mass Properties (collapsed)     │
│ ────────────────────────────────────│
│ [Gameplay Components]                │
│   ▶ Health                           │
│   ▶ Inventory                        │
│ ────────────────────────────────────│
│ [+ Add Component ▼]                  │
└──────────────────────────────────────┘
```

## Component Sub-Panels

### 1. Transform Component
**Always Present - Cannot Remove**

Fields:
- **Position** (Vec3): X, Y, Z with steppers (step: 0.1m)
- **Rotation** (Euler): X, Y, Z in degrees (step: 5°)
  - Internal: Quaternion, displayed as Euler for user-friendliness
  - Gimbal lock warning when pitch approaches ±90°
- **Scale** (Vec3): X, Y, Z with steppers (step: 0.1)
  - Minimum: 0.01 (prevent zero/negative)
  - Lock aspect ratio button (chain icon)
  - Uniform scale slider when locked

Special Features:
- "Reset Transform" button (position to origin, rotation to identity, scale to 1)
- "Focus in Viewport" button (frames camera on object)
- Local/World space toggle for position editing
- Copy/Paste transform values (text format for sharing)

### 2. Mesh Component
**Core Component**

Display:
- **Mesh Type**: Cube, Sphere, Plane, Cylinder, Capsule, Custom
- **Vertex Count**: Read-only info
- **Triangle Count**: Read-only info
- **Bounding Box**: Read-only dimensions

Actions:
- "Replace Mesh" button → Opens asset picker
- "Duplicate Mesh" button → Makes unique copy if shared
- "Export Mesh" button → Save to .obj/.gltf

Special Features:
- Thumbnail preview of mesh (small 3D viewport or wireframe)
- LOD levels display (if LOD system implemented)
- Memory usage indicator

### 3. Material Component

Fields:
- **Base Color**: 
  - RGB sliders (0-255) with live color preview swatch
  - Hex color input field (#RRGGBB)
  - Eye dropper tool (pick from viewport)
  - Alpha channel (0-255) with transparency preview
- **Metallic**: 0.0 to 1.0 slider
- **Roughness**: 0.0 to 1.0 slider
- **Emissive Color**: RGB with intensity multiplier
- **Texture Slots**:
  - Albedo map
  - Normal map
  - Metallic/Roughness map
  - Emissive map
  - Each with "Assign Texture" button

Special Features:
- "Material Presets" dropdown (metal, plastic, wood, glass, etc.)
- "Clone Material" button (make unique copy)
- Real-time preview sphere in inspector
- Vertex color support toggle

### 4. RigidBody Component
**Physics Component**

Fields:
- **Body Type**: Dropdown
  - Dynamic: Full physics simulation
  - Fixed: Immovable (level geometry)
  - Kinematic (Position): Scripted movement, affects dynamics
  - Kinematic (Velocity): Velocity-driven, affects dynamics
- **Mass**: Auto-calculated from collider + density, or manual override
  - Display: kg
  - "Auto" checkbox vs manual input
- **Linear Damping**: 0.0 to 10.0 (air resistance)
- **Angular Damping**: 0.0 to 10.0 (rotation resistance)
- **Gravity Scale**: Multiplier (0.0 = no gravity, 2.0 = double gravity)

Constraints:
- **Lock Position**: X, Y, Z checkboxes
- **Lock Rotation**: X, Y, Z checkboxes

Special Features:
- "Physics Preview" toggle (show velocity vectors in viewport)
- "Reset Velocity" button (zero linear + angular)
- Warning if no collider present

### 5. Collider Component
**Physics Component - Requires Transform**

Fields:
- **Shape Type**: Dropdown with preview icons
  - Box: Half-extents (X, Y, Z)
  - Sphere: Radius
  - Capsule: Radius + height + axis (X/Y/Z)
  - Cylinder: Radius + height + axis
  - Mesh: Uses mesh geometry (performance warning)
  - Compound: Multiple sub-colliders (advanced)

- **Material Properties**:
  - Friction: 0.0 to 1.0 (0 = ice, 1 = rubber)
  - Restitution: 0.0 to 1.0 (0 = clay, 1 = super ball)
  - Density: kg/m³ (for mass calculation)

- **Collision Filtering**:
  - Collision Layer: Bitmask checkboxes (Player, Enemy, Environment, etc.)
  - Collision Mask: What layers this collides with

Special Features:
- "Fit to Mesh" button (auto-size collider to match visible mesh)
- "Show Collider" toggle (wireframe overlay in viewport)
- Visual offset/rotation gizmo for collider (separate from mesh)
- Collider complexity warning (mesh colliders expensive)

### 6. Light Component

Fields:
- **Light Type**: Dropdown
  - Directional: Sun-like, infinite distance
  - Point: Spherical emission (radius)
  - Spot: Cone emission (radius + angle)
  - Area: Rectangular emission (width + height) - future

- **Color**: RGB with color picker
- **Intensity**: Lumens or lux (physical units)
  - Preset buttons: Candle, Bulb, Sun
- **Range**: Max distance (point/spot only)
- **Shadows**: Enable checkbox
  - Shadow Resolution: Low/Medium/High/Ultra
  - Shadow Bias: Prevent acne artifacts

Special Features for Spot:
- Inner/Outer cone angle sliders
- Falloff curve visualization

Special Features:
- "Light Probe" visualization (show influence volume)
- Performance cost indicator (shadow quality × range)

### 7. Camera Component

Fields:
- **Projection**: Dropdown
  - Perspective: FOV (30° - 120°)
  - Orthographic: Size (world units)
- **Near Clip**: Minimum draw distance (0.01 - 1.0)
- **Far Clip**: Maximum draw distance (10 - 10000)
- **Viewport Rect**: X, Y, Width, Height (normalized 0-1)
  - For split-screen / picture-in-picture
- **Clear Color**: Background color when no skybox
- **Render Layers**: Which layers to render

Special Features:
- "Look Through Camera" button (viewport switches to this camera)
- "Align with View" button (match current editor camera)
- Frustum visualization toggle in viewport

### 8. Audio Source Component

Fields:
- **Audio Clip**: Asset picker for sound file
  - Format: .wav, .ogg, .mp3
  - Duration display
  - Waveform preview
- **Volume**: 0.0 to 1.0 slider with dB conversion display
- **Pitch**: 0.5 to 2.0 (half-speed to double-speed)
- **Spatial Blend**: 0.0 (2D) to 1.0 (3D spatial)
- **Loop**: Checkbox
- **Play on Awake**: Auto-start when scene loads

3D Settings (when Spatial Blend > 0):
- **Min Distance**: Full volume radius
- **Max Distance**: Inaudible distance
- **Rolloff Mode**: Linear, Logarithmic, Custom curve
- **Doppler Level**: Pitch shift from movement

Special Features:
- "Play/Stop" preview button
- Volume meters (current playback level)
- Occlusion toggle (muffled when behind walls)

### 9. Script/Behavior Component
**Gameplay Logic**

Fields:
- **Script Asset**: Reference to behavior file
- **Exposed Parameters**: Auto-generated from script reflection
  - Serialize public fields as inspector controls
  - Support: int, float, bool, string, Vec3, Color, Asset references
  - Custom drawers for special types

Special Features:
- "Edit Script" button (opens in external editor)
- "Reload Script" button (hot-reload without restart)
- Validation errors displayed inline
- Script execution toggle (enable/disable)

### 10. Tag Component
**Organizational**

Fields:
- **Tags**: Multi-select checklist
  - Player, Enemy, Interactable, Saveable, etc.
  - Custom tag creation
- **Layer**: Dropdown (rendering layer assignment)
- **Name**: String (entity identifier)

Special Features:
- "Find by Tag" button (selects all entities with tag)
- Tag color coding in hierarchy view

### 11. Health Component
**Gameplay - Example Custom Component**

Fields:
- **Current Health**: Integer input
- **Max Health**: Integer input (auto-clamps current)
- **Regeneration Rate**: HP per second
- **Invulnerability**: Boolean checkbox
- **Death Behavior**: Dropdown
  - Destroy Entity
  - Disable Components
  - Trigger Animation
  - Spawn Prefab (death FX)

Special Features:
- Health bar visualization (red/green)
- "Damage Test" button (subtract 10 HP)
- "Kill" button (set health to 0)

### 12. Inventory Component
**Gameplay - Example Custom Component**

Fields:
- **Capacity**: Max item count or weight
- **Items**: Collapsible list
  - Each item shows: Icon, Name, Quantity, Weight
  - Drag-reorder items
  - Right-click context menu (remove, duplicate)
- **Auto-Stack**: Checkbox (combine identical items)

Special Features:
- "Add Item" dropdown (from item database)
- Total weight/capacity bar
- "Clear All" button with confirmation

### 13. Particle System Component
**VFX**

Fields:
- **Particle Texture**: Sprite sheet asset
- **Emission Rate**: Particles per second
- **Lifetime**: Min/Max range (seconds)
- **Start Speed**: Initial velocity
- **Start Size**: Min/Max range
- **Start Color**: Color gradient over lifetime
- **Gravity Modifier**: Affects fall speed
- **Shape**: Sphere, Cone, Box, Mesh (emission volume)
- **Looping**: Checkbox
- **Play on Awake**: Auto-start

Advanced:
- **Color Over Lifetime**: Gradient editor
- **Size Over Lifetime**: Curve editor
- **Velocity Over Lifetime**: XYZ curves
- **Rotation**: Random spin rate

Special Features:
- "Play/Stop" preview in viewport
- Particle count display (current/max)
- Performance warning (high particle count)

### 14. Animation Component
**Future - Placeholder**

Fields:
- **Animation Clip**: Asset reference
- **Speed**: Playback multiplier
- **Blend Mode**: Replace, Additive, Layered
- **Looping**: Checkbox

Timeline Preview:
- Scrubber bar with keyframes
- Play/Pause/Stop controls
- Frame-by-frame stepping

### 15. AI Navigation Component
**Gameplay - Pathfinding**

Fields:
- **Agent Radius**: Collision avoidance size
- **Agent Height**: Clearance check
- **Max Speed**: Movement velocity cap
- **Acceleration**: Ramp-up time
- **Stopping Distance**: Target arrival threshold
- **Auto-Braking**: Slow down at destination
- **Walkable Layers**: Which nav mesh layers to use

Special Features:
- "Show Path" toggle (debug visualization)
- "Set Destination" button (click viewport)
- Pathfinding cost display

### 16. LOD Group Component
**Performance**

Fields:
- **LOD Levels**: List (LOD0, LOD1, LOD2, etc.)
  - Each level:
    - Screen Coverage %: When to switch (e.g., 50%, 25%, 10%)
    - Mesh Reference: Lower poly version
    - Material Override: Simpler shader
- **Fade Mode**: None, Cross-Fade, Speed Tree
- **Animate Cross-Fading**: Smooth transitions

Special Features:
- "Preview LOD" slider (force specific level)
- Triangle count comparison chart
- Distance labels (at what distance each LOD shows)

### 17. Occluder Component
**Performance - Culling**

Fields:
- **Is Occluder**: Checkbox (blocks rendering behind)
- **Is Occludee**: Checkbox (can be culled)
- **Smallest Occluder**: Checkbox (don't cull this)

Special Features:
- Occlusion volume visualization
- Statistics: Objects culled by this occluder

### 18. Prefab Instance Component
**Content Management**

Display:
- **Prefab Source**: Asset path (read-only)
- **Overridden Properties**: List of modified values
  - Show diff vs. prefab base values
  - Revert individual property buttons

Special Features:
- "Update Prefab" button (save changes back to asset)
- "Revert All" button (discard overrides)
- "Unpack Prefab" button (break link, become unique)
- Visual indicator (blue outline) in hierarchy

## Add Component Workflow

### Component Browser Modal

When clicking "+ Add Component":
1. **Search Box**: Type-ahead filter by name
2. **Category Tabs**: Core, Physics, Audio, Gameplay, etc.
3. **Component List**: 
   - Name + icon
   - Short description tooltip
   - Grayed out if conflicts with existing components
   - Dependency warnings ("Requires Transform")
4. **"Add" Button**: Insert component with default values

### Quick Add Shortcuts
- Common components have hotkeys (R = RigidBody, C = Collider, etc.)
- Right-click entity in hierarchy → "Add Component" submenu
- Templates: "Physics Object" (RigidBody + Collider), "Light Source" (Light), etc.

## Multi-Selection Support

When multiple entities selected:
- **Shared Components**: Show components present on ALL entities
- **Mixed Values**: Display "—" (dash) for differing values
- **Bulk Edit**: Changing value applies to all selected
- **Add Component**: Adds to all selected entities
- **Remove Component**: Removes from all selected entities
- **"Copy Components" / "Paste Components"**: Transfer setup between objects

## Validation & Warnings

### Dependency Warnings
- Red warning icon: "Collider requires RigidBody for physics simulation"
- Suggest "Add RigidBody" button

### Conflict Warnings
- Yellow warning icon: "Multiple cameras on same entity conflict"
- Auto-disable older component or prompt user

### Performance Warnings
- Orange indicator: "Mesh collider expensive, consider primitive shape"
- Particle count exceeds budget
- Shadow resolution too high for mobile

### Value Constraints
- Transform scale: Prevent zero/negative with visual feedback
- Light range: Warn if exceeding reasonable bounds (> 1000 units)
- Audio distance: Max must be > min

## Presets & Templates

### Component Presets
- **Material Presets**: Metal, Plastic, Glass, Skin, etc.
  - One-click apply realistic PBR values
- **Light Presets**: Candle, Bulb, Sun, Fire
  - Realistic lumens + color temperature
- **Physics Material Presets**: Ice, Rubber, Wood, Concrete
  - Friction + restitution tuned

### Entity Templates
- "3D Object" → Mesh + Material + Collider
- "Light Source" → Transform + Light
- "Physics Prop" → Transform + Mesh + RigidBody + Collider
- "Audio Emitter" → Transform + Audio Source
- User can save custom templates

## Inspector Customization

### Layout Options
- **Compact Mode**: Smaller padding, icon-only buttons
- **Detailed Mode**: Full descriptions, tooltips
- **Sections Collapsed by Default**: User preference per component type

### Color Themes
- Component headers color-coded by category
- Physics = blue, Audio = green, Gameplay = purple

### Pin Components
- Pin frequently-used components to top of inspector
- Pinned state persists across sessions

## Keyboard Shortcuts

- **Tab**: Cycle through text fields
- **Enter**: Apply value and move to next field
- **Ctrl+C/V**: Copy/paste component values
- **Delete**: Remove selected component
- **Ctrl+D**: Duplicate component (if allowed)
- **F**: Focus on value field for quick edit

## Undo/Redo Support

Every inspector change creates undo history entry:
- Value changes
- Component additions/removals
- Batch operations (multi-select edits)

Undo stack shows:
- "Changed Transform.Position.X to 5.0"
- "Added RigidBody to Cube_01"
- "Removed 3 components from [5 objects]"

## Inspector Extensions / Plugins

### Custom Component Registration API

Third-party code can register inspector panels:
- Register type
- Provide UI rendering callback
- Specify dependencies/conflicts
- Define serialization format

Example use cases:
- Modding API: Users add custom gameplay components
- Engine plugins: Navigation mesh, terrain, water system
- Debug components: Performance profilers, collision visualizers

### Custom Property Drawers

Override default UI for specific field types:
- Vector3 → XYZ fields with color-coded labels
- Color → Color picker wheel
- Curve → Interactive graph editor
- Asset reference → Thumbnail + drag-drop zone

## Performance Considerations

### Lazy Rendering
- Only render inspector for selected entity
- Collapse expensive sections by default (particle system curves)
- Throttle updates to 30fps (no need for 144Hz UI)

### Value Change Debouncing
- Don't apply every keystroke while typing
- Batch changes on focus loss or Enter key
- Reduces undo stack bloat

### Large Scene Support
- Inspector complexity independent of total entity count
- Only queries selected entity components, not all entities

## Future Enhancements

### Visual Scripting Integration
- "Convert to Visual Script" button on components
- Node graph editor for component logic

### Component Dependency Graph
- Visualize which components reference each other
- Show data flow between components

### Inspector History
- Recent selections dropdown
- "Back" button to previous selection

### Inspector Docking
- Undock into separate window
- Multi-monitor support

### Collaborative Editing Indicators
- Show which user is editing which component (multiplayer editing)
- Lock components to prevent conflicts

## Technical Architecture Notes

### Reflection System
- Components must register field metadata (name, type, range, tooltip)
- Support for custom attributes (e.g., `[Range(0, 100)]`, `[Tooltip("Speed in m/s")]`)
- Auto-generate UI from reflection data

### Serialization
- Inspector edits write directly to component fields
- Dirty flag tracking for save/revert
- JSON or binary format for scene files

### Query Optimization
- Inspector holds cached component references
- Only re-query on selection change
- Use Bevy's change detection to update UI when components modified externally

### UI Framework Considerations
- EGUI (current): Immediate mode, simple but less customizable
- Potential switch to retained-mode UI for complex layouts
- Support for custom widgets (color picker, curve editor, etc.)
