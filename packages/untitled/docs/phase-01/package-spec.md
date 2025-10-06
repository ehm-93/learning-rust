# Effect-Behavior Package System: A Complete Walkthrough

All code examples are illustrative and not final.

## The First Launch

When you launch the game, the package system springs to life before the main menu appears. The PackageLoader scans the `packages/` directory, looking for two types of entries: subdirectories and .zip files. For each one it finds, it checks for a `package.toml` file - this is the only requirement that defines something as a package.

Let's say it finds both `packages/core/` (a directory) and `packages/fire_magic.zip` (a zipped package). The loader reads their manifest files and discovers that Fire Magic requires Core version ^1.0.0. It resolves the dependency graph - Core must load first. This prevents the chaos of a fire spell trying to use a projectile behavior that doesn't exist yet.

With dependencies sorted, the loader creates a Lua environment for each package. Each package gets its own isolated Lua state, but they all receive the same global `api` object - this is their window into the game engine. The packages can't see each other's internals, only communicate through the game systems via the API.

The loader executes `packages/core/init.lua`. The init script finds a global `api` waiting for it and uses it to register behaviors and effects:

```lua
-- init.lua - 'api' is already global, provided by the engine
-- Core package keeps it simple - behaviors are just inline Lua

api.behaviors.register("projectile", {
    update = function(entity, dt)
        entity.position = entity.position + entity.velocity * dt
    end,
    on_collision = function(entity, other)
        api.damage.deal(other, entity.damage)
        api.effects.despawn(entity.id)
    end
})

api.behaviors.register("lifetime", {
    init_state = function() return { age = 0 } end,
    update = function(entity, dt)
        entity.state.age = entity.state.age + dt
        if entity.state.age > entity.config.duration then
            api.effects.despawn(entity.id)
        end
    end
})
```

Next, Fire Magic's init.lua runs. It has its own ideas about organization:

```lua
-- fire_magic/init.lua - different package, different style
local behaviors = require("fire_behaviors")  -- Load from another file
local effects = load_json("data/effects.json")  -- Parse JSON data

-- Register a new damage type
api.damage_types.register("fire", {
    color = "#ff4400",
    resist_stat = "fire_resistance"  
})

-- Register behaviors from the module
for name, behavior in pairs(behaviors) do
    api.behaviors.register("fire_" .. name, behavior)
end

-- Register effects from JSON
for id, def in pairs(effects) do
    api.effects.register(id, def)
end
```

The beauty is that each package organizes itself however it wants. Core uses inline Lua. Fire Magic loads from multiple files and parses JSON. Another package might generate its content procedurally. The engine doesn't care - it just runs init.lua and lets the package use the API.

This entire process takes under 100 milliseconds. By the time you see the main menu, every package is loaded, all behaviors registered, ready for combat.

## The Architecture Philosophy

The system separates **behaviors** from **effects** for a crucial reason. Behaviors are the verbs - they define how things move and act. Effects are the nouns - they combine behaviors with specific parameters to create actual attacks.

A projectile behavior knows how to move something forward, but it doesn't know or care if it's an arrow, a fireball, or a healing orb. That's just configuration. This separation creates infinite variety from finite behaviors.

The package structure is deliberately minimal. Each package needs exactly two files:
- `package.toml` - metadata and dependencies
- `init.lua` - entry point that registers content

Everything else is up to the package author. Want to stuff everything in init.lua? Fine. Prefer a complex directory structure with dozens of files? Also fine. Want to embed a SQLite database and query it for effect definitions? Weird, but sure, go ahead. The engine just calls init.lua with a global `api` and lets the package handle the rest.

## The Life of a Fireball  

You're in combat and press the fire button. Your weapon references the "fireball" effect registered by the fire_magic package. Here's what happens.

The input system catches your click and triggers a PlayerActionEvent. The combat system looks up "fireball" in the effect registry - Fire Magic registered it during initialization:

```lua
-- How fire_magic registered this effect
api.effects.register("fireball", {
    behaviors = {"projectile", "lifetime", "fire_trail"},
    config = {
        speed = 600,
        duration = 3.0,
        damage = 25,
        damage_type = "fire"
    },
    on_hit = {
        spawn = "fire_explosion",
        despawn_self = true
    }
})
```

The EffectSpawner creates an entity with the three behaviors attached. Each behavior gets its own state table and configuration. The entity starts with velocity based on your aim direction and the configured speed.

Every frame - 60 times per second - the behavior processor updates all entities with behaviors. It calls each behavior's update function:

```lua
-- Projectile behavior moves it forward
update(entity, dt)  -- moves by velocity * dt

-- Lifetime behavior tracks age
update(entity, dt)  -- increments age, despawns if expired

-- Fire_trail behavior creates particles
update(entity, dt)  -- spawns fire particles behind projectile
```

When the collision system (still in Rust for performance) detects the fireball hit an enemy, it triggers the on_collision callback for each behavior. The projectile behavior deals damage. The fire_trail behavior applies burning status. 

The effect's on_hit configuration says to spawn "fire_explosion", so that effect spawns at the impact point. The explosion is just another effect with its own behaviors - probably an "area" behavior that damages everything in radius.

The original fireball despawns. The explosion runs for its duration, then despawns too. Half a second of real time, dozens of behavior updates, one spectacular impact.

## Creating New Packages

You want to add orbital strikes to the game. You create `packages/orbital_strikes.zip` with this structure:

```
orbital_strikes.zip
├── package.toml
├── init.lua
└── orbital.lua  # Your choice to organize this way
```

Your package.toml declares your package:

```toml
[package]
name = "orbital_strikes"
version = "1.0.0"
description = "Adds missiles that orbit before striking"

[dependencies]
core = "^1.0.0"  # Need projectile behavior from core
```

Your init.lua registers the new behavior:

```lua
-- orbital_strikes/init.lua
local orbital = require("orbital")  -- Load your behavior code

api.behaviors.register("orbiting", orbital.behavior)

api.effects.register("orbital_missile", {
    behaviors = {"orbiting", "projectile"},  -- Mix with core projectile
    config = {
        orbit_radius = 100,
        orbit_duration = 2,
        speed = 1000,  -- For the strike phase
        damage = 50
    }
})
```

You drop the .zip file into packages/ while the game is running. If hot-reload is enabled, the watcher detects the new package, loads it, resolves dependencies (core is already loaded), and runs init.lua. Your orbital missiles are instantly available.

You spawn one with the debug console: `/spawn orbital_missile`. It circles the nearest enemy twice, then strikes. The orbit is too wide. You update orbital.lua, re-zip, copy over the old file. The hot-reload triggers, re-runs your init.lua, and the next missile you spawn has a tighter orbit. 

Five second iteration time. No compilation. No restart.

## The Magic of Hot-Reload

You're tweaking the fire spread behavior. The game runs in windowed mode, your text editor beside it. You edit a value and save.

The filesystem watcher detects a change in `packages/fire_magic/`. It identifies this as a package directory and queues a reload. On the next frame, the reload system:

1. Clears all registrations from fire_magic
2. Creates a fresh Lua state
3. Provides the global `api`
4. Runs init.lua again
5. Fire_magic re-registers everything with new values

Existing effects keep their old behavior closures until they despawn. New effects get the updated behaviors. You can hot-reload while projectiles are mid-flight without crashes.

The same works for zipped packages, though you need to replace the .zip file rather than editing files inside it. Some developers work with directories during development, then zip for distribution.

## When Effects Chain

The cluster rocket demonstrates effect chaining across three levels:

1. **Rocket** flies forward, on hit spawns "cluster_explosion"
2. **Cluster explosion** immediately spawns 8 "cluster_bombs" in different directions  
3. **Cluster bombs** fly outward, on hit spawn "fire_puddle"
4. **Fire puddles** burn for 5 seconds

Each effect only knows about its immediate successor. The chain emerges naturally:

```lua
-- How these might be registered
api.effects.register("cluster_rocket", {
    behaviors = {"projectile", "lifetime"},
    on_hit = { spawn = "cluster_explosion" }
})

api.effects.register("cluster_explosion", {
    behaviors = {"cluster_spawn"},  -- Custom behavior that spawns 8 bombs
    config = { count = 8, effect = "cluster_bomb" }
})

api.effects.register("cluster_bomb", {
    behaviors = {"projectile", "lifetime"},
    config = { speed = 400, duration = 1 },
    on_hit = { spawn = "fire_puddle" }
})
```

The effect limit system (500 active effects max) prevents runaway chains. Oldest effects despawn early if limits are exceeded. This keeps performance stable while allowing spectacular cascades.

## The Package API

The global `api` is deliberately limited but powerful. Packages can:

- **Register** behaviors, effects, damage types, status effects
- **Query** the world for entities, distances, line of sight
- **Spawn** effects with position and configuration  
- **Deal** damage through the proper calculation pipeline
- **Create** particles and visual effects
- **Play** sounds with spatial positioning

Packages cannot:
- Access the filesystem (except through api.assets)
- Modify entities directly (must go through API)
- See other packages' internals
- Access network or system resources
- Break game invariants

This isn't sandboxing for security (we trust package authors), but architectural boundaries that keep the game stable and maintainable.

## Performance in Practice

You profile the game with 200+ projectiles and see Lua taking 8ms per frame. The "homing" behavior is the culprit - complex math, used by many effects.

Core package offers a solution - it can register native implementations:

```lua
-- core/init.lua
if api.native_available("homing") then
    api.behaviors.register_native("homing")  -- Use Rust implementation
else
    api.behaviors.register("homing", lua_homing)  -- Fallback to Lua
end
```

The native version runs 10x faster. The behavior update time drops to 1ms. But modders can still create custom behaviors in Lua - only the hot path needs optimization.

This hybrid approach gives both performance and flexibility. Core behaviors ship as native code. Exotic modded behaviors stay in Lua until they prove popular enough to optimize.

## Version Resolution

The package system supports semantic versioning to prevent breaking changes:

- `^1.2.3` - Any 1.x version >= 1.2.3 (won't jump to 2.0.0)
- `~1.2.3` - Only 1.2.x versions (won't jump to 1.3.0)  
- `>=1.2.3` - Minimum version, any newer is fine
- `1.2.3` - Exactly this version

When loading packages, the system:
1. Reads all package.toml files
2. Builds a dependency graph with version constraints
3. Resolves compatible versions
4. Detects conflicts (package A needs core ^1.0, package B needs core ^2.0)
5. Loads in dependency order

This means old packages keep working even as core systems evolve. A package written for core 1.0 still works when core is at 1.5, but the system prevents loading if someone tries to use it with core 2.0 (which has breaking changes).

## Shipping to Players

For release, some packages become part of the game executable. Core behaviors get compiled to Lua bytecode and embedded. They still go through the package system but load from memory instead of disk.

Players add their own packages to a user directory:
- Windows: `%APPDATA%/GameName/packages/`
- Mac: `~/Library/Application Support/GameName/packages/`
- Linux: `~/.local/share/GameName/packages/`

The game loads built-in packages first, then user packages. The package manager UI lets players:
- Enable/disable packages
- See dependency conflicts
- Check for updates
- Monitor performance impact

Since we trust package authors (no sandboxing), the game includes clear warnings about only installing packages from trusted sources. But the architectural boundaries mean a badly-written package might crash, but can't corrupt saves or access personal files.

## The Foundation for More

This effect-behavior system handles combat, but the architecture extends naturally:

- **Enemy AI** - Behaviors that control enemy movement and decisions
- **Items** - Effects that trigger on pickup or use
- **Environment** - Behaviors for moving platforms, traps, hazards  
- **Progression** - Status effects that persist between runs

Each follows the same pattern:
1. Package provides init.lua
2. Init script gets global `api`
3. Package registers its content however it wants
4. Game provides the runtime

The key insight: behaviors and effects are just one application of a general principle. Separate data (what to do) from logic (how to do it). Make logic composable. Let creativity emerge from combinations.

That's the real power - not just adding fireballs without recompiling, but players creating effects you never imagined, sharing them, building on each other's work. The game becomes a platform for creativity rather than a fixed experience.

---

# Package Loading System: Development Phases

## Core Principle
Build the package infrastructure that enables the effect-behavior system. Each package gets an isolated Lua state with a shared `api` global that connects to the game engine.

## Phase 0: Lua Runtime with API Bridge
**Goal:** Lua VMs with game API access  
**Delivers:** 
- mlua integrated with Bevy
- Create isolated Lua states per package
- Global `api` object injected into each state
- Basic API methods: `api.log()`, `api.register()`

**Success:** Lua can call `api.log("Hello from package")`

## Phase 1: Package Loading from Disk
**Goal:** Load packages with standard structure  
**Delivers:**
```
packages/
  test/
    package.toml    # name, version
    init.lua        # entry point
```
- Scan `packages/` directory
- Load package.toml for metadata
- Execute init.lua with `api` global

**Success:** Test package registers itself via `api.register("test", data)`

## Phase 2: Registration System
**Goal:** Packages can register game content  
**Delivers:**
- `api.behaviors.register(name, table)`
- `api.effects.register(name, table)` 
- `api.damage_types.register(name, table)`
- Store registrations in Rust-side registries

**Success:** Core package registers projectile behavior

## Phase 3: Multi-Format Support
**Goal:** Load from directories and .zip files  
**Delivers:**
- Load `packages/core/` (directory)
- Load `packages/fire_magic.zip` (archive)
- Both use same package.toml + init.lua structure
- Package can `require()` its own files

**Success:** Fire_magic.zip loads and registers content

## Phase 4: Dependency Resolution
**Goal:** Load packages in dependency order  
**Delivers:**
```toml
[dependencies]
core = "^1.0.0"
```
- Parse dependencies from package.toml
- Build dependency graph
- Topological sort
- Load packages in correct order

**Success:** Fire_magic loads after core it depends on

## Phase 5: Semantic Versioning
**Goal:** Version constraints that work  
**Delivers:**
- Support `^1.2.3` (compatible with 1.x.x)
- Support `~1.2.3` (patch updates only)
- Support `>=1.2.3` (minimum version)
- Version conflict detection
- Prevent incompatible packages from loading

**Success:** Old packages work with newer compatible core versions

## Phase 6: Hot-Reload System
**Goal:** Reload packages without restart  
**Delivers:**
- File watcher on `packages/` directory
- On change: clear package registrations
- Create fresh Lua state with `api`
- Re-run init.lua
- Existing entities keep old closures until despawn

**Success:** Edit behavior, see changes in <1 second

## Phase 7: Extended API Surface
**Goal:** Full game API for packages  
**Delivers:**
- `api.effects.spawn(name, config)`
- `api.query.entities(filter)`
- `api.damage.deal(target, amount)`
- `api.particles.create(type, position)`
- `api.sound.play(name, position)`
- `api.native_available(behavior)` for hybrid Lua/Rust

**Success:** Package can spawn effects and query world

## Phase 8: Package Isolation
**Goal:** Packages can't break each other  
**Delivers:**
- Each package gets isolated Lua state
- Packages can't see each other's globals
- Only communicate through `api`
- Filesystem access only through `api.assets`
- No network or system access

**Success:** Broken package can't corrupt others

## Phase 9: User Package Directories
**Goal:** User-installed packages  
**Delivers:**
- Load built-in packages from game directory
- Then load from user directories:
  - Windows: `%APPDATA%/GameName/packages/`
  - Mac: `~/Library/Application Support/GameName/packages/`
  - Linux: `~/.local/share/GameName/packages/`
- User packages can override built-ins

**Success:** Players install packages without modifying game

## Phase 10: Package Manager UI
**Goal:** Visual package management  
**Delivers:**
- List all loaded packages with versions
- Enable/disable packages
- Show dependency graph
- Display version conflicts
- Performance impact per package
- Error messages for failed loads

**Success:** Non-programmer can manage their mods

## Critical Path

**Minimum Viable (Phase 0-3):** 
- Packages can register behaviors/effects
- Core systems work through Lua

**Effect System (Phase 4-6):**
- Dependencies enable package ecosystems
- Versions prevent breaking changes
- Hot-reload enables rapid iteration

**Production Ready (Phase 7-8):**
- Full API for complex behaviors
- Isolation prevents crashes

**Player Facing (Phase 9-10):**
- Easy installation
- Visual management

## API Design

The `api` global is the only bridge between packages and game:

```lua
-- What every init.lua sees
api = {
    -- Registration
    behaviors = { register = function(name, def) end },
    effects = { register = function(name, def) end, spawn = function(...) end },
    damage_types = { register = function(name, def) end },
    
    -- Queries
    query = { entities = function(filter) end, distance = function(...) end },
    
    -- Actions
    damage = { deal = function(target, amount) end },
    particles = { create = function(...) end },
    sound = { play = function(...) end },
    
    -- Utilities
    log = function(msg) end,
    native_available = function(name) end,
    assets = { read = function(path) end },
}
```

## Package Structure Freedom

Packages organize themselves however they want:

```lua
-- Simple: everything in init.lua
api.behaviors.register("projectile", {
    update = function(entity, dt) 
        -- inline implementation
    end
})

-- Modular: load from files  
local behaviors = require("behaviors/all")
for name, def in pairs(behaviors) do
    api.behaviors.register(name, def)
end

-- Data-driven: parse JSON/TOML
local effects = parse_json(api.assets.read("effects.json"))
for id, effect in pairs(effects) do
    api.effects.register(id, effect)
end
```

The engine only cares that init.lua runs and uses the API.
