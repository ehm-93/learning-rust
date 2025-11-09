# Level Editor Design

## Overview
The level editor is a Bevy-based tool for creating and editing game levels. It follows an atom/molecule/organism pattern for composable level design.

## Core Architecture

### Asset Hierarchy (Atoms → Molecules → Organisms)

**Visual Atoms** (`models/`)
- Individual `.glb` files exported from Blender
- Examples: `chair.glb`, `chest.glb`, `tunnel_straight.glb`
- Just meshes, no game logic

**Functional Atoms** (defined in Rust)
- ECS components available in editor
- Examples: `Interactable`, `Container`, `LootTable`, `Door`, `Health`
- Shown as dropdown in editor inspector
- Configurable per-instance (e.g., `Container { slots: 12, locked: true }`)

**Molecules** (`prefabs/`)
- Visual atom + functional atoms combined
- Created by designers using "Save as Prefab" in editor
- Reusable across levels
- Directory structure:
  ```
  prefabs/chest_common/
  ├── prefab.yaml       # chest.glb + Interactable + Container config
  └── on_open.lua       # Optional custom behavior
  ```

**Organisms** (`levels/`)
- Complete playable scenes (32m × 32m × 32m)
- Placed prefab instances + level-specific logic
- Directory structure:
  ```
  levels/mining_shaft_7/
  ├── level.yaml        # Prefab placements, spawn points, triggers
  ├── quest_puzzle.lua  # Level-specific scripts
  └── notes.md          # Design documentation
  ```

### Editor Workflow

1. **Place** visual atoms from `models/` browser
2. **Configure** by adding functional components
3. **Group** and arrange into useful combinations
4. **Save as Prefab** → creates entry in `prefabs/`
5. **Build Level** by placing prefabs
6. **Save Level** → creates directory in `levels/`

### Editor States
**Play Mode**: Test level from current camera position
**Edit Mode**: Standard editing with transform tools  
**Place Mode**: Actively placing a specific object type

## Core Features

### 1. Scene Manipulation
- **Transform Tools**: Move/Rotate/Scale with visual gizmos
- **Grid Snapping**: 0.5m position grid, 15° rotation snap
- **Multi-Select**: Box select, Ctrl+click, Shift+click
- **Grouping**: Parent/child relationships for moving complex structures
- **Duplication**: Ctrl+D for quick copying with transforms

### 2. Asset Management
- **Model Browser**: Visual atoms from `models/`
  - Modules (tunnel pieces, rooms)
  - Props (furniture, equipment)
  - Characters (for reference/placement)
  - Searchable and filterable
  
- **Component Library**: Functional atoms (Rust-defined)
  - Interactable
  - Container (inventory storage)
  - Door (with open/close states)
  - LootTable (dynamic item generation)
  - Health/Damageable
  - Light (point, spot, emergency)
  - Trigger (collision zones)
  - AISpawn (enemy placement)
  - Shown as dropdown when editing entity
  
- **Prefab Browser**: Pre-built molecules from `prefabs/`
  - Categorized by type (containers, furniture, modules)
  - Preview thumbnails
  - Drag-and-drop placement
  - "Save Selection as Prefab" button
  
- **Instance Overrides**: 
  - Override component values per-instance
  - Change materials/colors without affecting prefab
  - Mark instances as "prefab variant" for tracking

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

### Creating a Prefab
1. Place visual atoms from `models/` browser (e.g., `chest.glb`)
2. Select and add components in Inspector (e.g., `Interactable`, `Container`)
3. Configure component properties (slots: 12, requires_key: false)
4. Optionally add child objects (locks, particles, sounds)
5. Click "Save as Prefab" → name it `chest_common`
6. Creates `prefabs/chest_common/prefab.yaml`
7. Now available in Prefab Browser for reuse

### Creating a Level
1. Start with base layout using tunnel modules (from `prefabs/` or `models/`)
2. Place prefabs for gameplay elements (`chest_common`, `terminal_locked`, etc.)
3. Add lighting (emergency strips, work lights)
4. Set up triggers for spawns, events, narrative moments
5. Configure faction zones and patrol paths
6. Test with Play-in-Editor (P key)
7. Save as level → Creates `levels/mining_shaft_7/` directory
8. Add level-specific Lua scripts for puzzles/events
9. Document design intent in `notes.md`

### Example: Building a Mining Office
```yaml
# 1. Place base structure
- tunnel_4way.glb (from models/)

# 2. Add furniture prefabs
- desk_corporate × 3
- chair_office × 8
- filing_cabinet × 4
- terminal_interactive × 2

# 3. Add narrative elements
- datapad_spawn (component: DatapadSource("manager_log_03"))
- corpse_miner (with Container component for loot)

# 4. Configure lighting
- emergency_strip (flickering)
- desk_lamp × 3 (no power)

# 5. Set up gameplay
- enemy_spawn (component: AISpawn { type: "infected_worker", count: 2 })
- trigger_zone (for ambient sound/music change)

# 6. Save as level
levels/corporate_office_b12/
├── level.yaml (all the above)
├── power_puzzle.lua (restore power sequence)
└── notes.md ("This is where outbreak started")
```

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

### Prefab Definition
```yaml
# prefabs/chest_common/prefab.yaml
prefab:
  name: chest_common
  description: Standard storage chest
  
root:
  model: chest.glb
  components:
    - type: Interactable
      prompt: "Open Chest"
      
    - type: Container
      slots: 12
      locked: false
      
    - type: Health
      max: 100
      current: 100
      
children:
  - name: lock_visual
    model: padlock.glb
    visible_when: "container.locked == true"
    
scripts:
  on_open: on_open.lua
```

### Level Definition
```yaml
# levels/mining_shaft_7/level.yaml
level:
  id: mining_shaft_7
  name: "Mining Shaft 7 - Abandoned"
  bounds: [32, 32, 32]
  position: [0, -2500, 1000]
  
prefab_instances:
  - prefab: tunnel_4way_damaged
    position: [0, 0, 0]
    rotation: [0, 0, 0]
    overrides:
      damage_level: 0.7
      
  - prefab: chest_common
    position: [5.2, 0, 3.1]
    rotation: [0, 45, 0]
    overrides:
      container.locked: true
      container.loot_table: "mining_equipment"
      
  - prefab: mining_drill
    position: [-8, 0, 2]
    rotation: [0, 180, 0]
    components:  # Adding components not in prefab
      - type: Interactable
        prompt: "Examine Drill"
      
spawn_points:
  - type: enemy
    entity: infected_miner
    position: [10, 0, -5]
    patrol_path: [p1, p2, p3]
    
  - type: player
    position: [0, 0, 16]
    rotation: [0, 0, 0]
    
triggers:
  - type: zone
    name: alert_area
    shape: box
    bounds: [[5, 0, 5], [15, 3, 15]]
    on_enter: "quest_puzzle.on_alert()"
    
lights:
  - type: emergency_strip
    position: [16, 3, 0]
    color: [1.0, 0.2, 0.1]
    intensity: 2.0
    flicker: true
    
faction: abandoned
power_level: 0.0
narrative_state: pre_discovery
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
**Development Helpers:**
- F12: Dump current selection details to console
- Ctrl+P: Copy selected position to clipboard
- Ctrl+Shift+D: Toggle debug visualization (bounds, sockets, navmesh)
- Alt+Click: Quick-inspect entity details without selecting

## Integration with Procedural Generation
The editor creates hand-authored levels that the procedural system connects:
- Hero levels in `levels/` remain exactly as designed
- Procedural system generates connecting tunnels and filler spaces
- Editor can mark "connection sockets" for generator to use
- Generator respects faction zones and narrative state set in editor
- Prefabs from `prefabs/` can be used by both editor AND procedural system
  - Shared vocabulary between hand-authored and generated content
