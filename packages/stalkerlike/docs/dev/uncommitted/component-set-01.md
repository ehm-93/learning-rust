# Component Set 01 - MVP Gameplay Components

## Overview

The minimum set of reusable Bevy components exposed in the editor that enable basic interactive gameplay. These are the **functional atoms** that designers combine with visual models to create gameplay prefabs.

**Philosophy**: Each component does one thing well. Complex behaviors emerge from combining simple components rather than creating monolithic "smart" components.

---

## Design Principles

### 1. Composable
Components work independently or together. `Container` doesn't require `Interactable`, but they combine naturally.

### 2. Editor-Friendly
All properties visible and editable in inspector. No hidden state requiring code inspection.

### 3. Script-Accessible
Lua scripts can read/modify component state through clean API.

### 4. Self-Contained
Each component owns its data. No complex cross-component dependencies.

---

## Component Categories

### Core Interaction (4 components)
Foundation for player-entity interaction

### Inventory & Items (3 components)
Item management and storage

### Navigation & Triggers (3 components)
Movement control and spatial events

### Doors & Switches (3 components)
Common environmental interactions

### Information & Feedback (2 components)
Player communication and HUD

**Total**: 15 components for MVP

---

## 1. Core Interaction Components

### `Interactable`
**Purpose**: Makes entity respond to player interaction (E key press, click, etc.)

**Properties**:
- `prompt_text`: String - UI text shown when player looks at entity ("Press E to open", "Examine")
- `interaction_range`: Float (default: 2.0m) - Max distance for interaction
- `require_line_of_sight`: Bool (default: true) - Must be visible to interact?
- `cooldown`: Float (default: 0.0s) - Time between interactions
- `enabled`: Bool (default: true) - Can be toggled by scripts

**Lua Events**:
- `on_interact(player)` - Fired when player presses interact key while targeting

**Use Cases**:
- Doors, containers, terminals, levers, buttons
- NPCs for dialogue
- Pickups (combined with `Item`)
- Quest objectives

**Editor Notes**:
- Automatically shows gizmo indicating interaction range (sphere)
- Prompt text supports color codes: `[yellow]Press E[/yellow] to unlock`

---

### `Inspectable`
**Purpose**: Provides detailed examination text when player looks at entity

**Properties**:
- `title`: String - Name shown in UI ("Security Terminal", "Damaged Pipe")
- `description`: String - Lore/detail text (multi-line supported)
- `inspection_range`: Float (default: 3.0m) - How close to see details
- `require_interaction`: Bool (default: false) - Must press E to inspect, or automatic on look?

**Lua Events**:
- `on_inspect(player)` - Fired when inspection triggered

**Use Cases**:
- Environmental storytelling objects
- Clues and hints
- Lore documents
- Damaged equipment descriptions

**Editor Notes**:
- Description supports markdown formatting
- Can link to audio logs or images

---

### `Highlightable`
**Purpose**: Visual feedback when player aims at entity

**Properties**:
- `highlight_color`: Color (default: white, alpha 0.3) - Overlay tint
- `outline_thickness`: Float (default: 2.0) - Pixel width of outline
- `outline_color`: Color (default: yellow) - Outline color
- `pulse`: Bool (default: false) - Gentle pulse animation?
- `style`: Enum - "Outline", "Tint", "Both", "None"

**Lua Events**:
- `on_highlight_start(player)` - Player started aiming
- `on_highlight_end(player)` - Player looked away

**Use Cases**:
- Works with `Interactable` to show what's usable
- Quest objective markers
- Hidden items (combined with `Item`)

**Editor Notes**:
- Preview highlight in editor viewport
- Automatically applied to entities with `Interactable`

---

### `Damageable`
**Purpose**: Entity can take damage and be destroyed

**Properties**:
- `max_health`: Float (default: 100.0)
- `current_health`: Float (runtime, editor shows max)
- `armor`: Float (default: 0.0) - Damage reduction
- `invulnerable`: Bool (default: false) - Cannot be damaged
- `destroy_on_death`: Bool (default: true) - Remove entity when health reaches 0
- `death_effect_prefab`: String (optional) - Spawn this prefab on death (explosion, debris)

**Lua Events**:
- `on_damage(amount, source, damage_type)` - When damaged
- `on_death(killer)` - When health reaches 0
- `on_heal(amount)` - When health restored

**Use Cases**:
- Destructible environment (crates, doors, walls)
- Enemies and NPCs
- Player health (special handling)
- Damageable objectives

**Editor Notes**:
- Health bar gizmo in editor (optional)
- Preview death effect placement

---

## 2. Inventory & Item Components

### `Container`
**Purpose**: Holds items that player can take or deposit

**Properties**:
- `capacity`: Int (default: 20) - Max number of item slots
- `locked`: Bool (default: false) - Requires unlock to open
- `lock_type`: Enum - "None", "Keycard", "KeyItem", "Code", "Script"
- `required_key_id`: String (if lock_type needs key) - Item ID that unlocks
- `lock_code`: String (if lock_type is Code) - 4-digit code
- `auto_unlock_on_interaction`: Bool (default: true) - Check keys automatically
- `contents`: Array of ItemStack (runtime) - What's inside
- `loot_table_id`: String (optional) - Auto-populate from loot table on spawn

**Lua Events**:
- `on_open(player)` - Container opened successfully
- `on_close(player)` - Container closed
- `on_attempt_locked(player)` - Tried to open while locked
- `on_unlock(player)` - Lock successfully removed
- `on_item_added(item, player)` - Item deposited
- `on_item_removed(item, player)` - Item taken

**Use Cases**:
- Chests, lockers, crates
- Corpse looting
- Storage containers
- Quest item stashes

**Editor Notes**:
- Preview contents in editor (manual placement)
- Link to loot table definitions
- Test lock/unlock in editor

---

### `Item`
**Purpose**: Entity is a pickupable item (world object, not inventory icon)

**Properties**:
- `item_id`: String - Unique identifier ("keycard_blue", "medkit")
- `stack_size`: Int (default: 1) - How many in this pickup
- `auto_pickup`: Bool (default: false) - Grab on touch, or require interaction?
- `pickup_sound`: String (optional) - Audio to play on pickup
- `destroy_on_pickup`: Bool (default: true) - Remove from world when taken

**Lua Events**:
- `on_pickup(player)` - Item collected by player
- `on_drop(player)` - Item dropped back into world

**Use Cases**:
- Key items and quest objectives
- Ammo and consumables
- Equipment and weapons
- Collectibles

**Editor Notes**:
- Automatically adds `Highlightable` if not present
- Links to item database for properties (weight, value, icon)

---

### `Inventory`
**Purpose**: Entity has an inventory (usually player, but could be NPCs, vehicles)

**Properties**:
- `capacity`: Int (default: 40) - Total item slots
- `weight_limit`: Float (default: infinite) - Max carry weight (if using encumbrance)
- `contents`: Array of ItemStack (runtime) - What's carried
- `auto_sort`: Bool (default: false) - Keep items organized by type

**Lua Events**:
- `on_item_added(item)` - Item placed in inventory
- `on_item_removed(item)` - Item removed from inventory
- `on_item_used(item)` - Item consumed/activated
- `on_inventory_full()` - Attempted to add item but no space

**Use Cases**:
- Player inventory (main use case)
- NPC inventories for trading
- Container inventories (alternative to `Container` component)

**Editor Notes**:
- Preview inventory contents in editor (for testing)
- Can initialize with starting items

---

## 3. Navigation & Trigger Components

### `AreaTrigger`
**Purpose**: Detects when entities enter/exit a volume, fires events

**Properties**:
- `shape`: Enum - "Box", "Sphere", "Cylinder"
- `size`: Vec3 (for Box), Float (for Sphere/Cylinder) - Dimensions
- `trigger_layer`: Bitmask - What entities trigger? (Player, NPC, Enemy, Item, etc.)
- `one_shot`: Bool (default: false) - Fire once then disable?
- `enabled`: Bool (default: true) - Can be toggled by scripts
- `cooldown`: Float (default: 0.0s) - Min time between triggers

**Lua Events**:
- `on_enter(entity)` - Entity entered volume
- `on_exit(entity)` - Entity left volume
- `on_stay(entity)` - Entity still inside (fires per frame, use sparingly!)

**Use Cases**:
- Quest zones and checkpoints
- Alarm triggers
- Auto-doors (opens when player near)
- Environmental hazards (radiation zones)
- Dialogue triggers

**Editor Notes**:
- Gizmo shows trigger volume (wireframe)
- Color-coded by layer (player=green, enemy=red, etc.)
- Test trigger in editor (place player manually)

---

### `NavBlocker`
**Purpose**: Blocks AI pathfinding through this area

**Properties**:
- `shape`: Enum - "Box", "Sphere", "Cylinder"
- `size`: Vec3 or Float - Dimensions
- `block_layer`: Bitmask - What's blocked? (Humanoid, Drone, Large, etc.)
- `dynamic`: Bool (default: false) - Updates navmesh when moved (expensive!)

**Use Cases**:
- Locked doors (until opened)
- Collapsed corridors
- Hazardous zones NPCs avoid
- Temporarily block AI routes

**Editor Notes**:
- Shows blocked area in navmesh preview
- Red wireframe gizmo

---

### `Waypoint`
**Purpose**: Marker for AI patrol routes or scripted paths

**Properties**:
- `waypoint_id`: String - Unique identifier for referencing in scripts
- `next_waypoint`: String (optional) - ID of next point in sequence
- `wait_time`: Float (default: 0.0s) - How long to pause at this point
- `animation`: String (optional) - Play this animation while waiting ("idle", "look_around")
- `radius`: Float (default: 1.0m) - How close to consider "reached"

**Lua Events**:
- `on_reached(entity)` - AI reached this waypoint

**Use Cases**:
- NPC patrol routes
- Scripted movement sequences
- Cinematic camera paths

**Editor Notes**:
- Line gizmo connecting to next waypoint
- Preview entire path chain
- Test path with debug AI

---

## 4. Door & Switch Components

### `Door`
**Purpose**: Opens and closes, blocks movement when closed

**Properties**:
- `state`: Enum - "Closed", "Opening", "Open", "Closing" (runtime)
- `locked`: Bool (default: false)
- `lock_type`: Enum - "None", "Keycard", "KeyItem", "Switch", "Script"
- `required_key_id`: String (optional)
- `open_duration`: Float (default: 1.0s) - Animation time
- `auto_close`: Bool (default: false)
- `auto_close_delay`: Float (default: 5.0s) - Time before auto-closing
- `opening_direction`: Enum - "Slide", "SwingIn", "SwingOut", "Up", "Script"
- `open_sound`: String (optional)
- `close_sound`: String (optional)
- `locked_sound`: String (optional)

**Lua Events**:
- `on_open(opener)` - Door opened
- `on_close()` - Door closed
- `on_attempt_locked(entity)` - Tried to open while locked
- `on_unlock(unlocker)` - Lock removed

**Use Cases**:
- Standard doors between rooms
- Security checkpoints
- Airlocks with delay
- Puzzle doors

**Editor Notes**:
- Preview open/close animation in editor
- Gizmo shows swing direction
- Collision automatically updates with state

---

### `Switch`
**Purpose**: Toggle between states, controls other entities

**Properties**:
- `state`: Bool (default: false) - On/Off, Up/Down, etc.
- `toggle_mode`: Bool (default: true) - Stays toggled, or spring-returns?
- `controlled_entities`: Array of EntityID - What does this control?
- `invert_control`: Bool (default: false) - On = target off?
- `animation_on`: String (optional) - Play this when turning on
- `animation_off`: String (optional) - Play this when turning off
- `sound_on`: String (optional)
- `sound_off`: String (optional)

**Lua Events**:
- `on_toggle(new_state, toggler)` - State changed
- `on_turn_on(toggler)` - Switched to true
- `on_turn_off(toggler)` - Switched to false

**Use Cases**:
- Light switches
- Lever puzzles
- Alarm controls
- Door controls

**Editor Notes**:
- Select controlled entities in viewport
- Shows connections with line gizmos
- Test switch in editor (see controlled entities respond)

---

### `PowerSource`
**Purpose**: Provides power to connected entities (lights, doors, terminals)

**Properties**:
- `powered`: Bool (default: true) - Currently providing power?
- `capacity`: Float (default: infinite) - Max power output
- `current_draw`: Float (runtime) - How much power being used
- `powered_entities`: Array of EntityID - What's connected?
- `auto_manage`: Bool (default: true) - Automatically power/depower connected entities?

**Lua Events**:
- `on_power_on()` - Power restored
- `on_power_off()` - Power lost
- `on_overload()` - Demand exceeds capacity

**Use Cases**:
- Generator that powers zone
- Battery pack for equipment
- Emergency power systems
- Power puzzle (restore power to progress)

**Editor Notes**:
- Shows power network in editor (line gizmos)
- Warns if capacity exceeded
- Can trigger power failure for testing

---

## 5. Information & Feedback Components

### `Terminal`
**Purpose**: Interactive computer terminal with text display

**Properties**:
- `title`: String - Terminal header ("SECURITY SYSTEM")
- `content`: String - Main text (multi-line, markdown supported)
- `locked`: Bool (default: false)
- `password`: String (optional)
- `requires_power`: Bool (default: true) - Disabled if unpowered?
- `screen_color`: Color (default: green) - Emissive tint for CRT look

**Lua Events**:
- `on_access(player)` - Terminal accessed successfully
- `on_access_denied(player)` - Wrong password
- `on_read(player)` - Content displayed
- `on_command_entered(command, player)` - If terminal accepts input

**Use Cases**:
- Lore documents and logs
- Door unlock terminals
- Quest information
- Security cameras (future)

**Editor Notes**:
- Preview terminal UI in editor
- Live-edit content
- Test password entry

---

### `InfoDisplay`
**Purpose**: Shows text/icon to player (floating label, subtitle, objective marker)

**Properties**:
- `text`: String - Message to display
- `display_mode`: Enum - "WorldSpace", "Screen", "Subtitle"
- `visible_range`: Float (default: infinite) - Max distance to show
- `always_visible`: Bool (default: false) - Ignore occlusion?
- `color`: Color (default: white)
- `icon`: String (optional) - Show icon alongside text
- `priority`: Int (default: 0) - Higher priority shows on top

**Lua Events**:
- `on_enter_range(player)` - Player close enough to see
- `on_exit_range(player)` - Player too far

**Use Cases**:
- Objective markers ("Reach extraction point")
- World-space labels ("Generator Room")
- Distance indicators (quest markers)
- Subtitle dialogue

**Editor Notes**:
- Preview in viewport
- Adjustable font size/style

---

## Component Combinations (Common Prefabs)

### Locked Chest
- `Interactable` (prompt: "Press E to open")
- `Container` (capacity: 10, locked: true, lock_type: Keycard)
- `Highlightable` (outline, yellow)

### Puzzle Door
- `Door` (locked: true, lock_type: Script)
- `Interactable` (prompt: "[red]LOCKED[/red]")
- `Damageable` (optional - can be destroyed)

### Light Switch
- `Switch` (toggle_mode: true)
- `Interactable` (prompt: "Toggle Lights")
- Controlled entities: All lights in room

### Quest Item Pickup
- `Item` (item_id: "keycard_blue", auto_pickup: false)
- `Highlightable` (pulse: true)
- `Inspectable` (description: "Security clearance keycard")

### Alarm Trigger
- `AreaTrigger` (shape: Box, trigger_layer: Player, one_shot: true)
- Lua script: `on_enter` spawns enemies and plays alarm

### Security Terminal
- `Terminal` (locked: true, password: "1234")
- `Interactable` (prompt: "Access Terminal")
- `PowerSource` requirement (disabled if unpowered)

---

## Scripting Integration

All components expose their state to Lua:

```lua
-- Check door state
if door:get_door():is_locked() then
    door:get_door():unlock()
end

-- Modify container
container:get_container():add_item("medkit", 3)
container:get_container():set_locked(false)

-- Trigger area check
if trigger:get_area_trigger():is_inside(player) then
    -- Do something
end

-- Switch control
switch:get_switch():toggle()
controlled = switch:get_switch():get_controlled_entities()
```

---

## Implementation Priority

### Phase 1: Core Interactions (Week 1-2)
1. `Interactable`
2. `Highlightable`
3. `Container`
4. `Item`
5. `Door`

**Goal**: Can open doors and loot chests

### Phase 2: Spatial Events (Week 3)
6. `AreaTrigger`
7. `Inspectable`
8. `Switch`

**Goal**: Can create scripted sequences triggered by player movement

### Phase 3: Systems & Feedback (Week 4)
9. `Inventory`
10. `Terminal`
11. `PowerSource`
12. `InfoDisplay`

**Goal**: Full quest/dialogue capability

### Phase 4: AI & Polish (Week 5+)
13. `Waypoint`
14. `NavBlocker`
15. `Damageable`

**Goal**: Enemy AI and destructible environment

---

## Editor Integration

### Inspector UI
Each component gets collapsible panel in entity inspector:

```
┌─ Entity: chest_locked ──────────┐
│ Transform                   [▼]  │
│ Mesh                        [▼]  │
│ Interactable                [▼]  │
│   Prompt: "Press E to open"      │
│   Range: [2.0] m                 │
│   Line of Sight: ☑               │
│ Container                   [▼]  │
│   Capacity: [10] slots           │
│   Locked: ☑                      │
│   Lock Type: [Keycard ▼]         │
│   Required Key: keycard_blue     │
│ Highlightable               [▼]  │
│   Style: [Outline ▼]             │
│   Color: #FFFF00                 │
│                                  │
│ [+ Add Component ▼]              │
└──────────────────────────────────┘
```

### Component Browser
Designer selects from categorized list:

```
Add Component:
├─ Core Interaction
│  ├─ Interactable
│  ├─ Inspectable
│  ├─ Highlightable
│  └─ Damageable
├─ Inventory & Items
│  ├─ Container
│  ├─ Item
│  └─ Inventory
├─ Navigation & Triggers
│  ├─ AreaTrigger
│  ├─ NavBlocker
│  └─ Waypoint
├─ Doors & Switches
│  ├─ Door
│  ├─ Switch
│  └─ PowerSource
└─ Information
   ├─ Terminal
   └─ InfoDisplay
```

---

## Validation Rules

### Required Dependencies
- `Container` works best with `Interactable` (warn if missing)
- `Item` auto-adds `Highlightable` if not present
- `Door` requires `Transform` (collision depends on position)

### Conflicts
- Cannot have both `Container` and `Item` (is it storage or pickup?)
- Cannot have both `Switch` and `Door` (use `Switch` to control `Door` instead)

### Warnings
- `AreaTrigger` with `on_stay` event (performance warning)
- `PowerSource` with no powered entities (unused component)
- `Terminal` without `Interactable` (how does player access it?)

---

## Testing Strategy

### Unit Tests (Per Component)
- Serialize/deserialize correctly
- State transitions valid (Door: closed→opening→open)
- Script API returns expected values

### Integration Tests (Component Combos)
- `Interactable` + `Container` = can open chest
- `Switch` + `Door` = switch controls door
- `AreaTrigger` + Lua script = trigger fires script

### Editor Tests (Designer Workflow)
- Add component to entity
- Modify properties in inspector
- Save prefab with components
- Load prefab in different level

---

## Success Metrics

### For Designers
- ✅ Can create 80% of gameplay interactions without Lua
- ✅ Component names/properties self-explanatory
- ✅ <2 minutes to create common prefabs (chest, door, switch)

### For Programmers
- ✅ Adding new component requires <1 hour (including editor UI)
- ✅ Components don't cross-reference (loose coupling)
- ✅ Lua API auto-generated from component definitions

### For Players
- ✅ Interactions feel consistent (same components = same behavior)
- ✅ Visual feedback clear (highlightable + inspectable)
- ✅ No "guess what this does" moments

---

## Future Expansions

### Phase 5+
- `Usable` (consumable items like medkits, batteries)
- `Craftable` (combine items to create new ones)
- `Dialogue` (NPC conversation trees)
- `QuestGiver` (NPC assigns quests)
- `Trader` (buy/sell items)
- `Health` (living entities with status effects)
- `Equipment` (worn items like suits, helmets)
- `Weapon` (firearms, melee, throwables)

But for MVP: **15 components is enough to build rich interactive environments.**
