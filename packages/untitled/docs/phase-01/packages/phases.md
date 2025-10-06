# Simplified Effect-Behavior Development Phases

## Core Principle
**Build the game mechanics first, then the infrastructure to support them.**

Every phase must leave the game playable. Infrastructure is extracted only when needed.

## Package Architecture
**Every package is just package.toml + init.lua + whatever else the package author wants.**

The game engine:
1. Reads package.toml for metadata and dependencies
2. Executes init.lua with a global `api` object
3. The package uses `api` to register its content

The package author decides how to organize behaviors, effects, assets - could be hardcoded in Lua, loaded from TOML/JSON files, generated procedurally, whatever. The engine doesn't care about internal package structure.

---

## Phase 0: Minimum Viable Lua
**Goal: Can Lua control entities?**

Single test - create 10 entities that circle via Lua:
```rust
// Hardcoded Lua string in Rust
const TEST_BEHAVIOR: &str = r#"
    function update(x, y, time)
        local angle = time * 2
        local radius = 100
        return x + math.cos(angle) * radius, 
               y + math.sin(angle) * radius
    end
"#;
```

Success metric: 60fps with 100 Lua-controlled entities.

---

## Phase 1: Core Behaviors (Hardcoded)
**Goal: Prove combat can work through Lua behaviors**

Write these behaviors as Lua strings in Rust:
- `projectile` - moves forward, checks collision
- `lifetime` - despawns after duration  
- `area` - damages in radius
- `homing` - seeks target

Create test effects by combining behaviors in code:
```rust
// Still hardcoded, but composable
let fireball = Effect {
    behaviors: vec![projectile_lua, lifetime_lua],
    config: hashmap!["speed" => 300.0, "lifetime" => 2.0],
};
```

Success metric: Player can shoot, enemies can shoot back, using only Lua behaviors.

---

## Phase 2: Hardcoded Effects
**Goal: Combine behaviors into working effects**

Create effects by combining behaviors in code:
```rust
// Effects composed from Phase 1 behaviors
let fireball = Effect {
    behaviors: vec!["projectile", "lifetime"],
    config: hashmap![
        "speed" => 300.0,
        "lifetime" => 2.0,
        "damage" => 25.0
    ],
};

let rocket = Effect {
    behaviors: vec!["projectile", "lifetime", "homing"],
    config: hashmap![
        "speed" => 200.0,
        "lifetime" => 5.0,
        "damage" => 50.0,
        "turn_rate" => 2.0
    ],
};
```

Still using hardcoded Lua behaviors from Phase 1.

Success metric: Player and enemies use different effect combinations.

---

## Phase 3: File Loading & Init Functions
**Goal: Load packages from disk with initialization**

Minimum package structure:
```
packages/
  core/               # Directory package
    package.toml
    init.lua
    (anything else the package wants)
  fire_magic.zip      # Zipped package containing:
    package.toml
    init.lua
    (anything else)
```

Support both unzipped directories and .zip files. The game:
1. Scans packages/ for directories and .zip files
2. Loads package.toml for metadata
3. Executes init.lua with global `api`

The init.lua decides how to organize itself:
```lua
-- init.lua gets global 'api' from engine
-- Package decides how to organize itself

-- Could hardcode behaviors
api.behaviors.register("projectile", {
    update = function(entity, dt)
        entity.position = entity.position + entity.velocity * dt
    end
})

-- Could load from files (package's choice)
local lifetime = dofile("behaviors/lifetime.lua")
api.behaviors.register("lifetime", lifetime)

-- Could parse TOML/JSON/whatever (package's choice)
local effects = parse_toml(read_file("effects.toml"))
for name, def in pairs(effects) do
    api.effects.register(name, def)
end
```

No hot-reload, no sandboxing yet, just execute init.lua at startup.

Success metric: Package successfully registers content from both .zip and directory formats.

---

## Phase 4: Hot-Reload Single Files
**Goal: Change file, see result immediately**

- Watch `packages/` directory for changes
- When any file in a package changes:
  - Clear that package's registrations
  - Re-execute its init.lua (gets fresh `api`)
  - Package re-registers everything
- New effects get new behavior
- In-flight effects keep old behavior

Example: Edit fire_magic/fire_behaviors.lua, save. The watcher detects change, re-runs fire_magic/init.lua, which re-loads and re-registers everything.

Success metric: Tweak projectile speed, see change in <1 second.

---

## Phase 5: Package Structure with Manifest
**Goal: Packages declare their identity and dependencies**

Each package requires exactly two files:
```
packages/
  core/
    package.toml    # Metadata
    init.lua        # Entry point
    (rest is up to the package author)
  fire_magic/
    package.toml
    init.lua
    fire_behaviors.lua  # Package's choice
    data/              # Package's choice
      effects.json     # Package's choice
```

Manifest with version:
```toml
[package]
name = "fire_magic"
version = "1.0.0"
description = "Adds fire damage and burning effects"

[dependencies]
core = "^1.0.0"  # Compatible with 1.x
```

The init.lua decides how to load content:
```lua
-- fire_magic/init.lua - global 'api' provided by engine
local fire_behaviors = require("fire_behaviors")
local effects = json.decode(read_file("data/effects.json"))

-- Register however the package wants
api.damage_types.register("fire", {color = "#ff4400"})
api.behaviors.register("burning", fire_behaviors.burning)
for id, effect in pairs(effects) do
    api.effects.register(id, effect)
end
```

Success metric: Fire magic package uses core projectile behavior.

---

## Phase 6: Version Resolution
**Goal: Handle semantic versioning properly**

Implement version resolution:
- `^1.2.3` = compatible with 1.x.x (>=1.2.3 <2.0.0)
- `~1.2.3` = approximately equivalent (>=1.2.3 <1.3.0)
- `1.2.3` = exact version
- `>=1.2.3` = minimum version

Build dependency graph considering versions:
1. Parse all package manifests
2. Resolve version constraints
3. Detect conflicts (package A needs core ^1.0, package B needs core ^2.0)
4. Topological sort respecting versions
5. Load in correct order

Success metric: Correctly handle package that needs newer core version.

---

## Phase 7: Native Behavior Optimization
**Goal: Performance-critical paths in Rust**

Port only the hottest paths:
- Projectile movement
- Collision checking  
- Area damage

Package init.lua can register native or Lua implementations:
```lua
-- core/init.lua - 'api' is global
-- Native implementation available for performance
api.behaviors.register_native("projectile")

-- Lua implementation for flexibility
local spiral = dofile("behaviors/spiral.lua") 
api.behaviors.register("spiral", spiral)

-- Package could even choose based on config
if api.config.prefer_native then
    api.behaviors.register_native("homing")
else
    api.behaviors.register("homing", require("behaviors/homing"))
end
```

Success metric: 500+ projectiles at 60fps.

---

## Phase 8: Effect Chaining
**Goal: Effects spawn other effects naturally**

Extend API for behaviors:
```lua
function on_hit(entity, api, collision_data)
    api.effects.spawn("explosion", {
        position = entity.position,
        source = entity.source
    })
end
```

Add spawn limits to prevent runaway chains.

Success metric: Cluster bomb that spawns more bombs.

---

## Phase 9: Player-Facing Tools
**Goal: Make modding accessible**

- Package manager UI (enable/disable/configure)
- Version conflict resolution UI
- Reload status display
- Error messages in-game
- Performance monitor per package

Success metric: Non-programmer can install and use mods.

---

## Phase 10: Content Expansion
**Goal: Prove the system enables variety**

Create 20 different effects using existing behaviors:
- Spiral shot
- Wave cannon
- Boomerang
- Lightning chain
- Orbit shield

Each in its own package to test version dependencies.

Success metric: Combat feels significantly more varied.
