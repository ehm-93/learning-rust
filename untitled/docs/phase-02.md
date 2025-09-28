# Phase 2 Technical Specification - Depth & Extraction

## Mission Statement
Add **vertical progression** and **extraction mechanics** to the validated combat foundation. Answer: "Can we create the core 'go deeper or extract safely' decision that drives roguelike engagement?"

## Phase 1 Validation Results ✅
- **Core combat loop**: Proven fun and engaging twin-stick foundation
- **Enemy variety**: Successfully creates tactical decisions under pressure
- **Team-based system**: Scales cleanly for additional complexity
- **Physics foundation**: Responsive and reliable
- **Architecture**: Clean, extensible foundation ready for depth systems

## Phase 2 Focus: Single Core Addition
Transform the static arena into a **multi-floor dungeon** where players choose how deep to venture before extracting to safety.

## Core System: Depth Progression

### Floor-Based Structure
Replace infinite spawning with **structured descent** through increasingly dangerous floors.

**Floor Mechanics:**
- **Floor 1-3**: Tutorial depths with current enemy density and rewards
- **Floor 4-6**: Increased enemy spawns, slightly better loot drops
- **Floor 7-10**: New enemy behaviors, valuable equipment appears
- **Floor 11+**: Escalating challenge for experienced players

**Extraction Points:**
- **Stairway down**: Proceed to next floor (point of commitment)
- **Stairway up**: Return to previous floor (escape route)
- **Emergency exit**: Instant return to surface, lose most loot (panic button)

**The Core Decision:**
At each floor, player chooses: *Extract safely with current loot* or *Risk everything for better rewards deeper*

### Supporting System: Basic Loot

**Simple Loot Scaling:**
- **Floor 1**: Health pickups, basic score items
- **Floor 3**: Temporary weapon upgrades (faster fire rate, larger bullets)
- **Floor 5**: Permanent upgrades (extra health, dash cooldown reduction)
- **Floor 7+**: Rare items that significantly change gameplay

**Extraction Mechanics:**
- **Successful extraction**: Keep all collected loot, add to persistent progression
- **Death**: Lose all loot but keep knowledge of floor layouts and enemy patterns
- **Emergency extraction**: Keep 50% of loot, can be used once per run

**Inventory Constraint:**
- **Limited slots**: Can only carry 3-5 items at once, must choose what to keep
- **Drop/pickup**: Can abandon items for better finds, creating meaningful decisions

## Technical Implementation Strategy

### Minimal Architecture Changes

#### New Components
- **CurrentFloor**: Resource tracking current depth (starts at 1)
- **Loot**: Component for pickupable items with rarity/type data
- **Inventory**: Player component with limited item slots
- **Stairway**: Component marking floor transition points

#### New Systems
- **Floor Management**: Spawns appropriate enemies/loot based on current floor
- **Loot Spawning**: Places items around the level based on floor depth
- **Extraction Logic**: Handles successful returns and loot persistence
- **UI Updates**: Shows current floor, inventory status, extraction options

#### Reuse Existing Systems
- **Combat**: No changes needed, works perfectly for all floors
- **Enemy AI**: Same archetypes, just different spawn rates per floor
- **Physics**: All current movement and collision systems unchanged
- **Sound/UI**: Minor additions, core systems remain the same

### Quality of Life Improvements

#### Visual Polish
- **Particle Systems**: Muzzle flashes, impact effects, environmental ambiance
- **Screen Effects**: Screen shake, damage indicators, ability cooldown visualization
- **UI Improvements**: Minimap, threat indicators, upgrade preview system
- **Visual Feedback**: Clear indication of player progression and power growth

#### Audio Enhancement
- **Dynamic Music**: Intensity-based soundtrack that responds to gameplay
- **3D Audio**: Positional audio for better spatial awareness
- **Voice/SFX**: Enemy callouts, ability usage feedback, environmental audio
- **Adaptive Mixing**: Audio levels that adjust based on action intensity

## Future Architecture Considerations

### Phase 3 Preparation
- **Multiplayer Foundation**: Architecture supports future co-op implementation
- **Procedural Generation**: Systems designed for algorithmic content creation
- **Modding Support**: Clear interfaces for community content creation
- **Platform Expansion**: Code structure enables console/mobile porting

---

**Core Philosophy**: Add the single most important roguelike element - meaningful risk/reward decisions - while keeping everything else that works.

**Success Definition**: Players experience the tension of "I could extract safely now, or risk it all for potentially better loot one floor deeper" and find this decision compelling enough to drive repeated play.
