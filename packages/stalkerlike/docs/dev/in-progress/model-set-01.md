# Model Set 01 - Corporate MVP Asset Pack

## Overview
Minimal viable asset set for two Corporate-style demo rooms (0-4km depth). Emphasizes the oppressive bureaucratic atmosphere of a dying mining colony while keeping scope tight.

## Theme: Corporate Levels 0-4km
**Visual Language**: Clean but worn, maintained infrastructure, authoritarian aesthetics, surveillance-heavy, "Earth will return" propaganda.

**Atmosphere**: Fluorescent hum, clipboards, terminals displaying outdated quotas, signs threatening consequences, the uncanny feeling of a skeleton crew maintaining operations built for 10x the population.

---

## Demo Room Layout

### Room 1: Checkpoint Control (8m × 6m × 3m)
*Where workers pass through security checkpoints*
- **Purpose**: Entry/exit controlled zone, ID verification, contraband scanning
- **Mood**: Paranoid, bureaucratic, oppressive
- **Key Features**: Security desk, ID scanners, propaganda posters, locked door

### Connecting Hallway (16m × 3m × 3m)
*Too-long corridor emphasizing isolation and scale*
- **Purpose**: Liminal transition space, create unease through emptiness
- **Mood**: Uncanny, lonely, fluorescent hum, footstep echoes
- **Key Features**: Repetitive architecture, minimal variation, unsettling sameness

### Room 2: Worker Break Room (6m × 8m × 3m)
*Oversized cafeteria for 5 workers*
- **Purpose**: Rest area showing massive scale disparity (built for 80, used by 5)
- **Mood**: Lonely, eerie emptiness, echoes of better times
- **Key Features**: Empty tables, single occupied locker, vending machine, safety notices

---

## Asset List - 31 Models Total

### A. Architecture (6 models)

#### 1. `tunnel_straight_corporate.glb` (800 polys)
- 4m × 3m × 4m section
- Metal panel walls, riveted construction
- Recessed fluorescent light strips
- Conduit runs along ceiling
- Color: `metal [0.3, 0.3, 0.35]` with rust accents
- Vertex-painted wear patterns

#### 2. `tunnel_corner_corporate.glb` (850 polys)
- 90-degree turn, same dimensions
- Corner-mounted junction box
- Warning stripes on floor

#### 3. `door_checkpoint.glb` (400 polys)
- 2.4m tall heavy security door
- Red light strip on top (emission material)
- Access panel beside frame
- Simple pivot animation point

#### 4. `door_standard.glb` (250 polys)
- 2.2m tall standard door
- Simple handle, cleaner design
- Slide animation point

#### 5. `wall_terminal_mount.glb` (150 polys)
- 1m × 1m recessed panel
- For mounting terminals/switches
- Can instance multiple times

#### 6. `floor_grating.glb` (200 polys)
- 1m × 1m modular grating section
- See-through mesh
- For drainage/ventilation areas

### B. Interactive Objects (7 models)

#### 7. `terminal_security.glb` (300 polys)
- Checkpoint control console
- Screen (emissive rectangle)
- Keyboard, buttons
- Mount to `wall_terminal_mount`

#### 8. `id_scanner.glb` (180 polys)
- Wall-mounted card reader
- Green/red indicator light
- Simple rectangle with detail

#### 9. `locker_single.glb` (200 polys)
- 2m tall × 0.5m wide worker locker
- Door with simple handle
- Vents at top/bottom
- Pivot animation for door

#### 10. `container_crate.glb` (150 polys)
- 1m × 1m × 1m storage crate
- Corporate logo stenciled
- Can be opened (lid pivots)

#### 11. `lever_wall.glb` (80 polys)
- Simple wall-mounted lever
- Up/down animation
- Emergency/utility use
- **Reused for lighting toggle** (different label/context)

#### 12. `button_panel.glb` (120 polys)
- 3-button console panel
- Red/yellow/green buttons
- Simple activation visual

#### 13. `vending_machine.glb` (500 polys)
- 2m tall snack dispenser
- "OUT OF ORDER" sign
- Product display (vertex colors)
- Corporate logo

#### 14. `intercom_wall.glb` (100 polys)
- Wall-mounted speaker/mic
- For atmospheric announcements
- Grille + single button

### C. Furniture (5 models)

#### 15. `table_cafeteria.glb` (180 polys)
- 2m × 0.8m rectangular table
- Metal frame, worn surface
- Simple boxy construction

#### 16. `chair_cafeteria.glb` (150 polys)
- Basic metal chair
- Slightly uncomfortable looking
- Stackable design aesthetic

#### 17. `desk_security.glb` (250 polys)
- Checkpoint desk
- Built-in monitor mount
- Drawers (non-functional)

#### 18. `bench_metal.glb` (120 polys)
- 2m long waiting bench
- Cold institutional design
- Bolted to floor

#### 19. `shelf_wall.glb` (140 polys)
- Wall-mounted storage
- 3 shelves
- Can hold small props

### D. Props & Dressing (6 models)

#### 20. `poster_propaganda.glb` (20 polys)
- Flat textured plane
- "Earth Expects Your Best"
- Corporate seal
- 1m × 1.5m size
- Could have 3-4 variants

#### 21. `clipboard.glb` (60 polys)
- Small handheld prop
- Paper with text lines
- Can sit on desks/tables

#### 22. `mug_coffee.glb` (80 polys)
- Simple cylinder + handle
- Corporate logo
- Table scatter

#### 23. `hardhat.glb` (100 polys)
- Worker safety helmet
- Can sit on lockers/desks
- Yellow or white variants

#### 24. `cable_spool.glb` (100 polys)
- Industrial wire spool
- Corner clutter
- Shows maintenance

#### 25. `trash_bin.glb` (80 polys)
- Simple waste receptacle
- Slightly overfull
- Metal construction

### E. Liminal Space Fillers (6 models)

#### 26. `pipe_ceiling_run.glb` (120 polys)
- 4m length ceiling pipe
- Various diameters (color-coded)
- Conduit brackets
- Can chain together endlessly
- Adds visual repetition

#### 28. `light_fixture_fluorescent.glb` (140 polys)
- 2m long ceiling-mount strip
- Recessed housing
- Emissive panel rectangle
- Slight flicker (shader/code)
- Creates oppressive lighting

#### 29. `light_emergency_red.glb` (60 polys)
- Small wall/ceiling mount
- Red lens cover
- Lower poly than fluorescent
- Dim red glow (emissive)
- Always present but normally off

#### 30. `junction_box_wall.glb` (90 polys)
- Electrical panel on wall
- Warning labels (vertex color)
- Slightly ajar door
- Background detail that repeats

#### 31. `vent_grate_wall.glb` (100 polys)
- 1m × 0.5m air vent
- Slotted grille
- Suggests vast HVAC system
- Repeatable every 8m

#### 32. `floor_stripe_yellow.glb` (20 polys)
- 4m × 0.2m warning stripe
- Faded yellow paint
- Runs along hallway edges
- Guides eye down corridor
- Simple geometry, high impact

---

## Lighting Setup (Dual-Mode System)

### Normal Mode (Fluorescent)
Use Bevy's built-in light components:

- **Fluorescent Strips**: Rectangular area lights (cool white [1.0, 0.98, 0.95])
  - Intensity: 200-300 lux
  - Every 4m in hallways
  - Creates oppressive clinical atmosphere
  
- **Terminal Glow**: Emissive materials on screens (blue-white)

### Emergency Mode (Red)
Activated by `switch_lighting_emergency`:

- **Emergency Lights**: Small red point lights [0.8, 0.2, 0.1]
  - Intensity: 50-100 lux (much dimmer)
  - Same positions as fluorescent fixtures
  - Creates shadows and unease
  
- **Terminal Glow**: Remains on (helps player navigate)

### Switching Logic
```lua
-- Lua script for lighting toggle
function on_lever_toggle(lever)
    local emergency_mode = lever:get_state() == "down"
    
    for _, light in ipairs(scene:get_lights()) do
        if light.type == "Fluorescent" then
            light.enabled = not emergency_mode
        elseif light.type == "Emergency" then
            light.enabled = emergency_mode
        end
    end
end
```

### Atmosphere Shift
- **Normal**: Oppressive but safe, can see everything, clinical
- **Emergency**: Eerie, shadows move, limited visibility, dread
- **Gameplay**: Same space feels completely different
- **Cost**: Just toggling light visibility, no new geometry

---

## MVP Color Palette (From art_pipeline.md)

```yaml
Primary:
  metal: [0.3, 0.3, 0.35]        # Base walls/floors
  rust: [0.5, 0.3, 0.2]           # Accents/wear
  work_light: [1.0, 0.9, 0.7]     # Lighting (emissive)

Accents:
  emergency: [0.8, 0.2, 0.1]      # Warning lights/locked doors
  corporate_blue: [0.2, 0.3, 0.5] # Logos, accents
  yellow_safety: [0.9, 0.8, 0.2]  # Warning stripes

Details:
  worn_paint: [0.25, 0.25, 0.27]  # Darker metal variation
  clean_metal: [0.4, 0.4, 0.45]   # Lighter metal variation
```

---

## Production Priority

### Phase 1 - Walkable Space (1-2 days)
1. `tunnel_straight_corporate.glb`
2. `tunnel_corner_corporate.glb`
3. `door_standard.glb`
4. Basic lighting test

### Phase 2 - Liminal Hallway (1 day)
5. `light_fixture_fluorescent.glb`
6. `light_emergency_red.glb`
7. `pipe_ceiling_run.glb`
8. `floor_stripe_yellow.glb`
9. `vent_grate_wall.glb`
10. `junction_box_wall.glb`
11. Assemble 16m hallway (test repetition/atmosphere)
12. Test lighting toggle using `lever_wall` (normal ↔ emergency)

### Phase 3 - Core Interactions (2-3 days)
11. `terminal_security.glb`
12. `door_checkpoint.glb`
13. `id_scanner.glb`
14. `locker_single.glb`
15. `container_crate.glb`

### Phase 4 - Scene Dressing (2-3 days)
15-25. Furniture set (tables, chairs, desk, bench, shelf)
26-31. Props & liminal fillers (posters, clutter, pipes, lights)
### Phase 5 - Polish (1 day)
- Vertex color refinement
- Lighting pass (emphasize hallway oppression)
- Arrange into checkpoint → hallway → break room flow

**Total Estimated Time**: 7-10 days for one person

---

## Technical Specs

### Polygon Budget
- Total: ~7,100 triangles for all 31 models
- Per-room: ~3,000-4,000 tris visible at once
- Hallway: ~1,400 tris (highly instanced, very cheap)
- Leaves headroom for characters/effects
- **Savings**: Reusing lever for lighting toggle saves 100 polys

### Texturing Strategy
- **Primary**: Vertex colors (fast, low memory)
- **Optional**: Single 512×512 atlas for:
  - Poster text/images
  - Corporate logos
  - Safety signs
  - Terminal screens

### File Sizes (Estimated)
- Each .glb: 20-100KB
- Total asset pack: ~1.5MB
- Very git-friendly

---

## Scene Assembly Example

### Checkpoint Control Room Layout

```
[Door-Standard]  [Tunnel-Straight]  [Door-Checkpoint]
      ↑                                    ↑
   Entrance                            ID Scanner →
                                      Red Light ↑
    
[Desk-Security] ← Terminal
     ↑
  [Chair]

[Bench] ← Waiting area

[Poster-Propaganda] × 2 (on walls)

[Intercom] (wall-mounted, near door)
```

### Break Room Layout

```
[Door-Standard]  [Tunnel-Corner]
      ↑
      
[Table] × 5  (only 1-2 have items)
[Chair] × 20 (arranged formally)

[Locker] × 16 (one wall, only 2-3 show use)

[Vending-Machine] (broken, "OUT OF ORDER")

[Shelf] (mostly empty)

[Trash-Bin] × 2

Props: Scattered mugs, 1 clipboard, 2 hardhats
```

### Hallway Layout (Liminal Space)

```
[Door-Standard] ← From Checkpoint
      ↓
[Tunnel-Straight] × 4 (16m total)

Ceiling: [Light-Fluorescent] every 4m (4 total) - Normal mode
         [Light-Emergency-Red] same positions - Emergency mode
         [Pipe-Ceiling-Run] running length (continuous)
Walls:   [Vent-Grate] × 4 (alternating sides, every 4m)
         [Junction-Box] × 2 (mid-points)
         [Lever-Wall] × 1 (entrance, labeled "EMERGENCY LIGHTING")
         [Switch-Lighting-Emergency] × 1 (entrance, near door)
         
Floor:   [Floor-Stripe-Yellow] both edges (guides path)

      ↓
[Door-Standard] ← To Break Room
```
**Lighting Toggle Interaction**:
1. Player approaches lever labeled "EMERGENCY LIGHTING" near hallway entrance
2. Pull lever down (animation)
3. Fluorescent lights fade out over 0.5s
4. Emergency red lights fade in over 0.5s
5. Atmosphere completely transforms
6. Pull lever up to restore normal lightingransforms
7. Can toggle back anytime

**Liminal Effect Achieved Through**:
- Repetitive identical lighting (every 4m)
- Minimal variation (just vents and junction boxes)
- Yellow floor stripes creating false perspective depth
- Length feels longer than it is (16m stretched by sameness)
- Fluorescent hum and footstep echo (audio, not assets)
- No windows, no variation, no relief
- **NEW**: Dual lighting modes create two distinct psychological states

---

## Why This Set Works for MVP

### Minimal But Complete
- Two distinct spaces showing different Corporate aspects
- Can walk, interact, observe
- Sets visual tone immediately

### Atmospheric Victory
- Immediately communicates theme: oppressive bureaucracy, massive scale disparity, dying infrastructure
- Empty cafeteria conveys population collapse better than exposition
- Liminal hallway creates unease through repetition and isolation
- **Lighting toggle transforms same space into two different emotional experiences**
- Player feels the "built for 50K, holds 5K" truth viscerally through empty corridors

### Modular & Reusable
- Tunnel pieces can build longer corridors
- Furniture recombines for other rooms
- Props scatter throughout colony
- All assets serve multiple contexts

### Atmospheric Victory
- Immediately communicates theme: oppressive bureaucracy, massive scale disparity, dying infrastructure
- Empty cafeteria conveys population collapse better than exposition
- Liminal hallway creates unease through repetition and isolation
- Player feels the "built for 50K, holds 5K" truth viscerally through empty corridors

### Expandable Foundation
- Easy to add: warning signs, more furniture variants, lighting fixtures
- Natural next steps: Union aesthetic set, abandoned zones, C-7 corruption
- Same modular system scales to full game

---

## NPC Consideration (Minimal for MVP)

For MVP, consider **one** simple NPC model:

#### `npc_worker_corporate.glb` (800-1200 polys)
- Generic worker in coveralls
- Simple rig (10-15 bones)
- T-pose export
- Can stand/idle animate in-engine
- Vertex-colored uniform (Corporate blue/gray)

Place 1-2 in scenes as static/idle for scale reference and life. No AI needed for MVP.

---

## Success Criteria

After building this set, you should have:
✅ Two rooms players can walk through  
✅ Connecting liminal hallway that creates unease  
✅ Dynamic lighting toggle (normal/emergency modes)  
✅ Doors they can open (simple interaction)  
✅ Objects they can inspect (lockers, terminals)  
✅ Clear visual identity (Corporate dystopia)  
✅ Atmospheric lighting (fluorescent oppression + eerie red alternative)  
✅ Environmental storytelling (empty = population crisis)  
✅ Liminal space effect (repetition, isolation, wrong scale)  
✅ Foundation for expansion (modular pieces)on, wrong scale)  
✅ Foundation for expansion (modular pieces)

**This is the MINIMUM to make the theme feel real.**

---

## Next Asset Sets (Future)

After Corporate MVP works:

- **Union Industrial**: Welding marks, jury-rigged repairs, solidarity posters
- **Frontier Makeshift**: Scavenged materials, personal touches, chaos
- **Abandoned Zones**: Ice/frost, darkness, emptiness, failure
- **C-7 Infected**: Crystal growth, impossible geometry, horror
- **Surface Ruins**: Ross 154 skybox, giant spaceport hangars, maintenance bays
But for now: **32 models, 2 rooms + liminal hallway, dual lighting modes, Corporate style. Ship it.**

---

## Design Notes: Liminal Space Psychology

### Why the Hallway Matters

**Liminal Theory**: Transitional spaces (hallways, waiting rooms, stairwells) designed for passage, not habitation. When *too empty* or *too familiar*, they trigger unease.

**Our Implementation**:
- **Repetition**: Same light every 4m, same vent placement, endless pipes
- **Scale Wrongness**: Built for crowd flow, used by one person
- **Purposeless Duration**: 16m feels longer due to lack of variation
- **Fluorescent Oppression**: Cool white light washing out detail
- **Acoustic Void**: Hard surfaces echo footsteps unnaturally
- **Dual Lighting**: Toggle creates two distinct psychological states from same geometry

### The Lighting Toggle

**Why It Matters**:
- Transforms player perception without changing geometry
- Same hallway becomes two different experiences
- Teaches interaction with environment
- Sets up power failure horror later

**Normal Mode (Fluorescent)**:
- Oppressive but "safe" - can see everything
- Clinical, sterile, uncomfortable
- The kind of lighting you want to escape
- But you can navigate confidently

**Emergency Mode (Red)**:
- Eerie, shadowy, uncertain
- Depth perception skewed by monochrome red
- Familiar space becomes threatening
- Movement feels different, suspicious
- Where is that shadow from?

**Gameplay Benefit**: 
- Teaches player to dread empty spaces (sets up abandoned zone horror later)
- Makes populated rooms feel like relief (control checkpoint = safety?)
- Establishes colony's ghost-town nature without text
- 5 simple models create huge psychological impact through repetition

**Cost/Benefit**:
- Only ~430 extra polygons for all 6 liminal models (reusing lever for toggle)
- Instance heavily (4 fluorescent, 4 red emergency, 4 vents, 4 pipes, 2 junction boxes, 2 stripe sections, 1 lever)
- Total hallway geometry: ~1,400 tris for 16m of oppressive atmosphere with dual modes
- Best bang-for-buck in entire asset set
- **Reuse Win**: Same lever model serves utility + lighting toggle (just different labels/context)
