# Effect-Behavior Package System: Development Phases

## Core Principle
Build the game mechanics first, then the infrastructure to support them. Each phase enables the next.

## Phase 0: Proof of Concept
**Goal:** Validate Lua can control entities  
**Delivers:** Lua runtime embedded in Bevy, entities moving via Lua functions
**Enables Phase 1:** Foundation to build actual game behaviors in Lua

## Phase 1: Core Behaviors  
**Goal:** Prove combat works through Lua behaviors  
**Builds on Phase 0:** Uses Lua runtime to implement real game mechanics
**Delivers:** Projectile, lifetime, area, and homing behaviors as Lua strings in Rust
**Enables Phase 2:** Behaviors ready to be combined into effects

## Phase 2: Hardcoded Effects
**Goal:** Combine behaviors into complete combat effects
**Builds on Phase 1:** Composes behaviors into fireball, rocket, grenade effects
**Delivers:** Working combat system using behavior composition
**Enables Phase 3:** Effects work, now need to externalize from code

## Phase 3: File Loading & Packages
**Goal:** Load packages from disk instead of hardcoding
**Builds on Phase 2:** Takes working effects and loads them from files
**Delivers:** package.toml + init.lua system, support for .zip and directories
**Enables Phase 4:** Files can be modified without recompiling

## Phase 4: Hot-Reload
**Goal:** Change files and see results immediately  
**Builds on Phase 3:** Watches the file system packages were loaded from
**Delivers:** <1 second iteration time for behavior tweaking
**Enables Phase 5:** Fast iteration makes complex packages feasible

## Phase 5: Dependencies & Versions
**Goal:** Packages can build on other packages
**Builds on Phase 3:** Extends package.toml with version and dependencies
**Delivers:** Packages can use behaviors/effects from other packages
**Enables Phase 6:** Need version resolution to prevent conflicts

## Phase 6: Version Resolution
**Goal:** Handle semantic versioning constraints properly
**Builds on Phase 5:** Implements ^, ~, >= operators for dependency versions
**Delivers:** Old packages keep working as dependencies update
**Enables Phase 7+:** Stable foundation for ecosystem growth

## Phase 7: Native Optimization
**Goal:** Performance-critical code runs at native speed
**Builds on Phase 1:** Ports Lua behaviors to Rust, keeping same interface
**Delivers:** 500+ projectiles at 60fps via hybrid Lua/Rust system
**Enables Phase 8:** Performance headroom for complex effects

## Phase 8: Effect Chaining
**Goal:** Effects can spawn other effects naturally
**Builds on Phase 2+7:** Uses effect system with performance for chains
**Delivers:** Cluster bombs, explosions that spawn more effects
**Enables Phase 10:** Foundation for creative combat variety

## Phase 9: Player Tools
**Goal:** Make the system accessible to non-programmers
**Builds on Phase 4+5+6:** Exposes reload, packages, and versions in UI
**Delivers:** Package manager, conflict resolver, performance monitor
**Enables Phase 10:** Players can easily try community content

## Phase 10: Content Expansion  
**Goal:** Prove the system enables creative variety
**Builds on Everything:** Uses all systems to create 20+ unique effects
**Delivers:** Spirals, boomerangs, chain lightning, orbital shields
**Proves Success:** Combat feels fresh and moddable

## Architecture Flow
```
Phase 0-2: Prove concept → Build mechanics → Compose effects (all hardcoded)
Phase 3-6: Externalize → Iterate fast → Share packages → Resolve versions  
Phase 7-8: Optimize core → Enable complexity
Phase 9-10: Polish tools → Expand content
```

## Critical Path
**Minimum Viable:** 0→1→2→3 (game works with external files)  
**Your Requirements:** +5→6 (version resolution for ecosystem)  
**Performance:** +7 (native optimization for ship quality)
**Polish:** +4,8,9,10 (nice but not essential)