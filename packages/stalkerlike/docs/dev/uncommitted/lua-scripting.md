# Lua Scripting

## Overview

Enable designers to create custom behaviors, quest logic, and one-off interactions without requiring engine recompilation. Lua scripts act as **glue code** between engine-implemented systems and designer-created content.

**Goal**: Rapid iteration on gameplay content while keeping core systems performant and safe.

---

## Why Lua?

### Designer Empowerment
- **No Rebuild Required**: Edit script, see changes immediately
- **Simple Syntax**: Easy to learn, hard to break things
- **Self-Contained**: Scripts live with the content they modify
- **Examples-Driven**: Copy-paste patterns for common tasks

### Technical Benefits
- **Sandboxed**: Scripts can't crash engine or corrupt save files
- **Hot-Reload**: Test changes without restarting game
- **Lightweight**: Fast to load, minimal memory overhead
- **Established**: Proven pattern in game development (WoW, Roblox, LÖVE, etc.)

### Scope Discipline
Scripts handle **content-specific logic only**:
- "When player opens this specific chest, check for keycard"
- "When alarm triggers, spawn 3 enemies at these locations"
- "This lever controls these specific lights"

Scripts DO NOT handle **core systems**:
- Movement, combat, inventory (engine responsibility)
- Physics, rendering, networking (Rust's domain)
- Performance-critical operations (keep in compiled code)

---

## What Problems Does This Solve?

### Problem 1: One-Off Interactions
**Without Lua**: Every unique puzzle/door/container requires new Rust code, rebuild, test cycle  
**With Lua**: Designer writes 10-line script, iterates in seconds

### Problem 2: Quest Complexity
**Without Lua**: Complex sequences hardcoded, brittle, hard to modify  
**With Lua**: Quest logic lives with level data, easy to tweak and debug

### Problem 3: Content-Code Coupling
**Without Lua**: Changing "alarm spawns 3 enemies" requires programmer time  
**With Lua**: Designer edits script directly, no engine changes needed

### Problem 4: Iteration Speed
**Without Lua**: Change → Rebuild → Restart → Test (minutes)  
**With Lua**: Change → Hot-reload → Test (seconds)

---

## Integration Points

### Prefab Scripts
Attach custom behavior to reusable objects.

**Location**: `assets/prefabs/<prefab_name>/*.lua`

**Use Cases**:
- Locked containers requiring specific items
- Switches/levers with custom effects
- Interactive terminals with unique responses
- Traps with specific trigger conditions

**Common Events**:
- When spawned in world
- When player interacts
- When opened/closed
- When damaged/destroyed

### Level Scripts
Define level-specific sequences and behaviors.

**Location**: `assets/levels/<level_name>/*.lua`

**Use Cases**:
- Intro/outro cinematics
- Alarm sequences and reinforcement spawning
- Environmental events (lights flickering, doors locking)
- Quest progression triggers

**Common Events**:
- Level loaded
- Player enters/exits zone
- Custom trigger volumes activated
- Quest flags changed

---

## Capability Categories

### Entity Manipulation
**What**: Find, move, enable/disable, modify entities in the scene  
**Why**: Scripts need to affect world state based on player actions

**Examples**:
- Open specific door when puzzle solved
- Enable lights when generator powered
- Spawn enemies at predefined locations
- Destroy objects after time limit

### Player Interaction
**What**: Read player state, modify inventory, show messages  
**Why**: Scripts respond to and affect player directly

**Examples**:
- Check if player has required keycard
- Give quest items upon completion
- Display context-specific messages
- Track player position for proximity triggers

### Audio/Visual Feedback
**What**: Play sounds, trigger effects, control lighting  
**Why**: Scripts create atmosphere and communicate state changes

**Examples**:
- Play "unlock" sound when door opens
- Flash red lights during alarm
- Spawn particle effects for explosions
- Fade music based on danger level

### Timing & Sequences
**What**: Delay actions, create timed sequences, schedule events  
**Why**: Many interactions happen over time, not instantly

**Examples**:
- Wait 5 seconds before spawning enemies
- Flash lights every 0.5 seconds for alarm
- Show dialogue messages in sequence
- Delay door closure after player passes through

### State Management
**What**: Set/check flags, track quest progress, save custom data  
**Why**: Scripts need to remember what happened across play sessions

**Examples**:
- Track if alarm has been triggered before
- Remember puzzle solutions player discovered
- Count enemies killed for quest objective
- Mark checkpoints reached

---

## Design Principles

### 1. Designer-First API
Scripts should read like English, not programming jargon.

**Good**: `player:has_item("keycard_blue")`  
**Bad**: `InventorySystem::Query(player_entity).HasItemID(0x4F2A)`

### 2. Fail Gracefully
Scripts failing shouldn't break the game, just log the issue.

**Outcome**: Designer sees error in console, game keeps running, fix script and hot-reload.

### 3. Examples Over Documentation
Provide working examples for every common pattern. Designers copy-paste and modify.

**Outcome**: Faster onboarding, fewer "how do I..." questions, more time creating content.

### 4. Limited Scope
Don't expose everything. Only what designers actually need.

**Outcome**: Simpler API, fewer ways to cause problems, clearer mental model.

---

## Example Use Cases

### Use Case 1: Puzzle Door
**Need**: Door opens only when three levers in correct positions  
**Solution**: Script checks lever states, unlocks door when pattern matches  
**Benefit**: Designer tweaks lever positions/pattern without programmer help

### Use Case 2: Timed Alarm
**Need**: When player trips alarm, lights flash and enemies spawn after delay  
**Solution**: Script triggers audio, flashes lights on timer, spawns enemies after 5 seconds  
**Benefit**: Designer adjusts timing, spawn locations, enemy count freely

### Use Case 3: Locked Chest
**Need**: Chest requires specific keycard, consumes it on open  
**Solution**: Script checks player inventory, removes keycard if present, unlocks chest  
**Benefit**: Designer changes required item without code changes

### Use Case 4: Quest Trigger
**Need**: When player enters zone for first time, show dialogue and set quest flag  
**Solution**: Script checks flag, shows timed messages, sets flag to prevent repeat  
**Benefit**: Designer writes all dialogue and timing in one place

### Use Case 5: Environmental Storytelling
**Need**: Lights flicker and sparks fly when approaching damaged junction box  
**Solution**: Script detects player proximity, triggers particle effects and audio  
**Benefit**: Artist places effect, designer fine-tunes trigger distance and frequency

---

## Success Metrics

### For Designers
- ✅ Can create new interactions without asking programmers
- ✅ Can iterate on quest logic in under 1 minute (edit → hot-reload → test)
- ✅ Clear error messages guide toward solutions
- ✅ Example scripts cover 80% of common needs

### For Programmers  
- ✅ Content changes don't require engine rebuilds
- ✅ Scripts can't corrupt game state or crash engine
- ✅ Performance bottlenecks stay in Rust, not Lua
- ✅ API surface stable, rarely needs expansion

### For Players
- ✅ More varied, polished interactions (designers iterate faster)
- ✅ Fewer bugs from quick content fixes (no recompile needed)
- ✅ Richer quest content (designers prototype freely)

---

## What This Enables

### During Development
- **Prototype Fast**: Test quest ideas without committing to Rust implementation
- **Art Direction**: Lighting designers script sequences without programmer bottleneck
- **Quest Design**: Writers implement dialogue trees and branching directly
- **Level Design**: Environmental storytelling through scripted interactions

### After Shipping
- **Bug Fixes**: Patch broken quest logic without game update
- **Content Updates**: Add new quests/interactions post-launch
- **Community Content**: Potential modding support (if desired)
- **Balancing**: Tweak difficulty, timing, rewards based on player feedback

---

## Constraints & Limitations

### What Scripts Cannot Do
- Modify core gameplay systems (movement speed, combat mechanics, etc.)
- Access file system or network (security/stability)
- Directly manipulate ECS or physics engine (safety)
- Run continuously every frame (performance)

### Why These Constraints Matter
**Safety**: Scripts can't accidentally corrupt saves or crash game  
**Performance**: Core loops stay fast in Rust  
**Simplicity**: Smaller API surface = easier to learn  
**Stability**: Engine changes don't break existing scripts

---

## Implementation Scope

### Phase 1: MVP (Core Features)
**Goal**: Enable basic prefab customization and level sequences

**Capabilities**:
- Event callbacks (on_use, on_open, on_trigger)
- Entity queries (find by name, get components)
- Player interaction (inventory, messages)
- Timers (delay, repeat)
- Audio playback

**Success**: Designer can create locked chest, puzzle door, alarm sequence without programmer help

### Phase 2: Quest Support
**Goal**: Enable complex quest chains and state management

**Capabilities**:
- Quest flag system (persistent state)
- Dialogue sequences
- Objective tracking
- NPC interaction triggers

**Success**: Designer implements multi-stage quest entirely in Lua

### Phase 3: Polish
**Goal**: Improve debugging and iteration experience

**Capabilities**:
- In-editor script testing with mock entities
- Visual debugger (breakpoints, variable inspection)
- Performance profiling for script hotspots
- Better error messages with fix suggestions

**Success**: Designer fixes script bugs in <1 minute average

---

## Why Not Other Solutions?

### Why Not Visual Scripting?
**Pros**: Non-programmers friendly, graph-based  
**Cons**: Harder to version control, slower to edit, more complex to implement  
**Decision**: Lua provides better text-based workflow, easier git diffs, simpler MVP

### Why Not Just Rust?
**Pros**: One language, maximum performance  
**Cons**: Compile times kill iteration, designers need programmer help  
**Decision**: Rust for systems, Lua for content = best of both worlds

### Why Not Configuration Files (YAML/JSON)?
**Pros**: Simple data, no logic  
**Cons**: Can't express complex behaviors, limited to predefined patterns  
**Decision**: YAML for data, Lua for behavior = clear separation

---

## Measuring Success

### Quantitative Goals
- **<30 seconds** from script edit to seeing result in-game (hot-reload)
- **<5 minutes** for designer to learn pattern from example
- **>80%** of content interactions scriptable without Rust changes
- **Zero** game crashes from script errors (sandboxing works)

### Qualitative Goals
- Designers feel **empowered** to solve own problems
- Programmers feel **freed** from content implementation bottleneck  
- Players experience **richer** interactions from faster iteration
- Codebase stays **clean** with clear system/content boundaries

---

## Conclusion

Lua scripting is the **multiplier** that transforms engine capabilities into diverse player experiences. By enabling designers to iterate at the speed of thought, we create more content, fix bugs faster, and maintain clear architectural boundaries.

**Remember**: The engine provides the vocabulary. Scripts write the story.

## Conclusion

Lua scripting is the **multiplier** that transforms engine capabilities into diverse player experiences. By enabling designers to iterate at the speed of thought, we create more content, fix bugs faster, and maintain clear architectural boundaries.

**Remember**: The engine provides the vocabulary. Scripts write the story.
