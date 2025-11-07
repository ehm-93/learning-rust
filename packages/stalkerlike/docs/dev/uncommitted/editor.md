# Level Editor Design

## Overview
The level editor is a Bevy-based tool for creating and editing game levels. It follows an atom/molecule/organism pattern for composable level design.

## Core Architecture

### Component Hierarchy
- **Atoms**: Individual meshes (wall panels, pipes, light fixtures)
- **Molecules**: Prefabs/modules (tunnel sections with integrated components)
- **Organisms**: Complete rooms/encounters (mining chambers with props and triggers)
- **Templates**: Chunk definitions for streaming (corporate sectors, mining shafts)

### Editor States
```rust
enum EditorMode {
    Play,           // Test level from current position
    Edit,           // Standard editing mode
    Place(String),  // Placing specific object type
}
```

## Core Features

### 1. Scene Manipulation
- **Transform Tools**: Move/Rotate/Scale with visual gizmos
- **Grid Snapping**: 0.5m position grid, 15° rotation snap
- **Multi-Select**: Box select, Ctrl+click, Shift+click
- **Grouping**: Parent/child relationships for moving complex structures
- **Duplication**: Ctrl+D for quick copying with transforms

### 2. Asset Management
- **Asset Browser**: Categorized list of placeable objects
  - Modules (tunnel pieces, rooms)
  - Props (furniture, equipment)
  - Lights (point, spot, emergency)
  - Triggers (zones, spawns)
  - Decals (wear, signage, damage)
- **Search & Filter**: Quick asset location
- **Prefab System**: Save configured entity groups for reuse
- **Material Overrides**: Per-instance material/color changes

### 3. Level Logic
- **Trigger Volumes**: Box/sphere collision zones
- **Spawn Points**: Enemy and item placement markers
- **Navigation Hints**: AI pathfinding helpers
- **Script Hooks**: Named entities for Lua scripting
- **Faction Zones**: Territory ownership markers

### 4. Viewport Features
- **Camera Controls**: 
  - WASD movement
  - Mouse look
  - Shift for speed
  - F to focus on selection
- **Visibility Layers**: Toggle categories on/off
- **LOD Preview**: See distance-based quality
- **Lighting Modes**: Full/Work/Unlit
- **Play-in-Editor**: P key to test from camera position

### 5. Inspector Panel
- **Transform**: Precise position/rotation/scale input
- **Components**: Edit ECS components directly
- **Metadata**: Custom properties as key-value pairs
- **Validation**: Warnings for issues (missing textures, invalid spawns)

## Workflow

### Creating a Level
1. Start with modular tunnel pieces for basic layout
2. Place key encounters and setpieces
3. Add props for environmental storytelling
4. Set up lighting (sparse, atmospheric)
5. Add triggers and spawns
6. Configure faction zones
7. Test with Play-in-Editor
8. Export to YAML/SQL

### Keyboard Shortcuts
- **G**: Move tool
- **R**: Rotate tool  
- **S**: Scale tool
- **Ctrl+D**: Duplicate selection
- **Delete**: Remove selection
- **Ctrl+S**: Save level
- **Ctrl+Z/Y**: Undo/Redo
- **P**: Play mode toggle
- **H**: Hide/show selection
- **F**: Focus on selection
- **1-9**: Visibility layers

## Data Format

### YAML Scene Definition
```yaml
chunk:
  id: mining_shaft_7
  position: [0, -2500, 1000]
  bounds: [32, 32, 32]
  
entities:
  - type: tunnel_4way
    pos: [0, 0, 0]
    rot: [0, 0, 0]
    state: abandoned
    
  - type: mining_drill
    pos: [5.2, 0, 3.1]
    rot: [0, 45, 0]
    components:
      health: 0.3
      powered: false
      
lights:
  - type: emergency_strip
    pos: [16, 3, 0]
    color: [1.0, 0.2, 0.1]
    intensity: 2.0
    flicker: true
```

## Implementation Priority

### Phase 1: Core Editing (Week 1)
- Basic transform tools
- Grid snapping
- Asset placement
- Save/load YAML

### Phase 2: Advanced Tools (Week 2-3)
- Multi-select and grouping
- Prefab system
- Play-in-Editor
- Undo/redo

### Phase 3: Polish (Week 4+)
- Material editor
- Lighting preview
- Validation system
- Performance optimizations

## Debug Features
```rust
// Development helpers
fn debug_commands(keys: Res<Input<KeyCode>>) {
    if keys.just_pressed(KeyCode::F12) {
        // Dump current selection to console
        println!("Selection: {:?}", selected_entities);
    }
    if keys.pressed(KeyCode::LControl) && keys.just_pressed(KeyCode::P) {
        // Copy position to clipboard
        clipboard.set_text(format!("pos: {:?}", transform.translation));
    }
}
```

## Integration with Procedural Generation
The editor creates "hero" chunks that the procedural system connects with generated content:
- Hand-authored setpieces remain untouched
- Procedural system fills connecting tunnels
- Editor can mark "connection points" for the generator
- Generator respects faction zones set in editor
