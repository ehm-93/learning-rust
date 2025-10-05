# Combat System Refactor - Foundation Specification

## Philosophy

**Build extensible foundation first, add magic later**

Current system is hardcoded chaos. Before adding elements, spells, and interactions, we need a clean foundation that treats combat as generic effects, not "bullets" and "grenades."

## Design Guardrails

**Preventing System Creep**

The refactor creates a clean foundation. Here's how to keep it clean:

### The Modifier Trap

**Wrong approach:**
```
// DON'T DO THIS
enum Modifier {
  Homing { turn_speed: f32 },
  Exploding { radius: f32 },
  Splitting { count: u8 },
  Chaining { max_chains: u8 },
  // ... 50 more special cases
}
```

This recreates the hardcoded mess we're escaping.

**Right approach:**

Modifiers are behaviors composed from existing systems:

- **Homing**: MovingEffect + VelocitySteering component (general-purpose entity following)
- **Exploding**: MovingEffect + SpawnOnDespawn(EffectDefId) component
- **Splitting**: Just spawn multiple effects with spread angles (in spawner logic)
- **Chaining**: EffectPayload with chain_count, ActiveEffect does the targeting

**Rule:** If you're adding a Modifier variant, you're probably doing it wrong. Add a component or compose existing ones.

### Data-Driven Test

Before adding code for new effect behavior, ask:

1. Can I express this with existing components? (usually yes)
2. Can I compose multiple effects? (spawn effect A that spawns effect B)
3. Is this a general-purpose behavior? (steering, spawning, chaining)

If answer to any is yes, it's data/composition, not code.

### Performance Boundaries

**Acceptable:**
- 500 active effects (measured, runs 60fps)
- 100 EffectEvents per frame (batched)
- 1000 SimpleProjectiles (pooled)

**Warning signs:**
- Frame time spikes when spawning effects → need pooling
- Collision detection >2ms → need spatial hashing
- Effect count climbing unbounded → need limits

**Hard limits:**
- Max 500 total effects (despawn oldest)
- Max 200 EffectEvents per frame (drop excess, log warning)
- Max 1000 SimpleProjectiles (refuse spawn, reuse pool)

Monitor in debug UI. If limits hit regularly, tune numbers or optimize, don't remove limits.

### Scope Discipline

**Phase 0-1 scope (this refactor):**
- Generic effects
- Data-driven definitions
- Basic delivery methods
- Performance optimization
- Debug tooling

**Explicitly NOT in scope:**
- Elements (future)
- Spell books (future)
- Terrain destruction (future)
- Complex AI tactics (future)
- Procedural generation (future)

When someone says "while we're refactoring combat, let's add...", the answer is no. Foundation first.

### Success Metrics

**Code health:**
- Zero effect-specific logic in core systems
- All new attacks = data file entries
- No constants for damage/stats

**Performance:**
- 60fps with 200+ active effects
- <1ms resolution time per frame
- Pooling prevents allocation spikes

**Developer experience:**
- Hot-reload works reliably
- Effect iteration takes seconds, not minutes
- Debug commands cover common cases

**Gameplay:**
- Combat feels better than before
- Enemy variety obvious from attacks
- Different builds feel distinct

If any metric degrades, stop and fix before continuing.

---

## Immediate Goals

### What We're Building Now

**Generic Effect System**
- Any entity can apply effects to any other entity
- Effects defined by data, not hardcoded types
- Collision detection agnostic to entity types
- Damage/knockback/status as composable effects

**Flexible Combat State**
- Entities have health, resistances, modifiers
- Faction system (not hardcoded teams)
- Status effects framework (ready for elements later)
- Clean separation: logic vs visuals

**Data-Driven Configuration**
- No more PROJECTILE_DAMAGE constants
- Effect definitions in data structures
- Enemy abilities reference effect configs
- Easy to add new attack types without code changes

### What We're NOT Building Yet

**Elements & Interactions**
- No fire/ice/lightning system
- No ground effects
- No element combos
- Foundation doesn't care about element types

**Spell Books & Pages**
- No spell book equipment system
- No page collection/modification
- Keep current shoot/grenade controls
- Foundation ready for spells later

**Advanced Mechanics**
- No destructible terrain
- No complex modifier chains
- No combo detection
- Just clean, extensible basics

---

## System Architecture Overview

**How Everything Wires Together**

### The Flow: Input → Effect → Resolution → State

**Player Perspective:**
1. Player clicks to attack
2. Input system emits PlayerActionEvent
3. Action handler reads player's current attack config (EffectDefinition)
4. EffectSpawner creates MovingEffect entity with EffectPayload
5. MovingEffect travels, collides, or expires
6. On collision/expiry, emits EffectEvent with target + effect data
7. EffectResolver reads EffectEvent, applies changes to CombatState
8. Visual/audio systems react to state changes

**Enemy Perspective:**
Same flow. AI triggers effects based on archetype's EffectDefinitions.

**Example: Basic Projectile**
```
Player clicks → PlayerActionEvent(Shoot)
  ↓
ActionHandler reads: PlayerAttackConfig (current weapon/spell)
  ↓
Finds: EffectDefinition {
  damage: 10,
  delivery: Projectile,
  speed: 800,
  knockback: 200
}
  ↓
EffectSpawner creates MovingEffect entity
  Components: MovingEffect, EffectPayload, Velocity, Collider
  ↓
Collision system detects hit with Enemy
  ↓
Emits: EffectEvent { source, target, effect_data }
  ↓
EffectResolver:
  - Applies 10 damage to enemy.CombatState
  - Applies knockback velocity
  - Despawns projectile
  ↓
VisualSystem sees damage:
  - Hit flash
  - Damage number
  - Impact sound
```

### Key Separation of Concerns

**Input Layer** (player/input.rs, player/actions.rs)
- Translates raw input into PlayerActionEvents
- No combat logic

**Effect Layer** (combat/effects.rs - new)
- Generic EffectSpawner creates entities from EffectDefinitions
- MovingEffect handles travel, lifetime, collision
- EffectPayload defines what happens on trigger
- Delivery methods: projectile/beam/area/instant

**Resolution Layer** (combat/resolver.rs - new)
- EffectResolver applies effects to entities
- Reads effect data, modifies CombatState
- Emits events for visuals/audio

**State Layer** (components, resources)
- CombatState: health, shields, damage modifiers per entity
- Faction: who fights whom (replaces Team)
- StatusEffect: generic status framework (burning/frozen/etc later)

**Presentation Layer** (visuals, audio, UI)
- Reacts to state changes
- Hit flashes, particles, damage numbers
- Decoupled from combat logic

---

## What Changes

### Remove
- Team enum
- Projectile/Grenade components
- spawn_enemy_bullet, spawn_shotgun_spread functions
- shoot_projectiles, throw_grenades hardcoded systems
- Hardcoded damage constants (PROJECTILE_DAMAGE, etc)
- ProjectileImpactEvent, GrenadeExplosionEvent

### Refactor
- Collision: generic effect collision, not projectile-specific
- Health: becomes CombatState with modifiers
- AI: references EffectDefinitions, not hardcoded attacks
- Player actions: trigger effects via config, not hardcoded spawn

### Build New
- EffectDefinition data structure
- MovingEffect component (replaces Projectile/Grenade)
- EffectPayload component (what effect does)
- EffectEvent (replaces specific impact events)
- EffectSpawner system
- EffectResolver system
- Faction component
- StatusEffect framework (empty for now, ready for elements)

---

## Implementation Phases

### Phase 0: Combat Foundations Refactor

**Goal:** Tear down hardcoded combat, build extensible foundation.

**Current Problems:**
- `combat.rs`: Team enum, ProjectileImpactEvent, direct damage
- `components.rs`: Projectile/Grenade hardcoded
- `enemy.rs`: spawn_enemy_bullet, spawn_shotgun_spread
- `player/systems.rs`: shoot_projectiles, throw_grenades
- `constants.rs`: PROJECTILE_DAMAGE, ENEMY_BULLET_DAMAGE sprawl
- `events.rs`: weapon-specific events

**0.a: Resolution Pipeline (BUILD THIS FIRST)**

Core logic without worrying about components yet. Easier to test in isolation.

**EffectEvent** (input):
- source: Entity
- targets: Vec<Entity> (can hit multiple)
- effect_id: EffectDefId
- hit_positions: Vec<Vec2> (for visuals)

**Three parallel resolvers:**

- **DamageResolver** → DamageEvent
  - Same-frame multi-hit: sum all damage, apply once
  - Calculation: damage * source.damage_mult * (1.0 - target.resistances[damage_type])
  
- **KnockbackResolver** → KnockbackEvent
  - Same-frame multi-hit: average knockback vectors
  - Prevents absurd knockback stacking
  
- **StatusResolver** → StatusEvent
  - Same-frame multi-status: highest intensity wins for refresh-type, sum for stack-type
  - Handles stacking per StackBehavior

**Appliers** (run after all resolvers):
- DamageApplier: reads DamageEvent, modifies CombatState.health
- KnockbackApplier: reads KnockbackEvent, modifies Velocity
- StatusApplier: reads StatusEvent, adds/updates StatusEffect

**Testing:** Unit tests with mock entities, verify multi-hit resolution logic.

**0.b: Effect Types & Components**

Two fundamentally different effect patterns:

**Instant/Moving Effects** (projectiles, explosions):
- Spawn entity, travel/trigger, emit EffectEvent, despawn
- Components: MovingEffect, EffectPayload

**Continuous Effects** (beams, auras, DoTs):
- Spawn entity, tick damage over time, despawn on duration end
- Components: ActiveEffect, EffectPayload

**MovingEffect** component:
- lifetime: Timer
- pierce_remaining: u8
- collision_behavior: CollisionBehavior (Despawn, Pierce, Bounce)

**ActiveEffect** component:
- duration: Timer
- tick_rate: f32 (damage per second, not per hit)
- affected_entities: HashSet<Entity> (tracks who's currently in effect)
- radius: Option<f32> (for area effects like auras)

**EffectPayload** component (shared by both):
- definition_id: EffectDefId
- source_entity: Entity
- source_faction: FactionId

**Why separate?**
- MovingEffect: collision-based, one-shot damage
- ActiveEffect: time-based, continuous damage
- Different queries, different systems, no confusion

**Lightweight Projectile Optimization:**
For basic bullets with no special behavior, use simplified component:
- SimpleProjectile { damage, faction, lifetime }
- Skips full EffectPayload overhead
- Dedicated fast-path collision system
- Can upgrade to full effect if needed (pierce, special hit, etc)

**0.c: Effect Definitions & Registry**

**EffectDefinition** struct:
- damage: f32
- damage_type: DamageType (Physical, Magical, True)
- knockback: f32
- effect_type: EffectType (Instant, Moving, Continuous)
- delivery: DeliveryMethod
- pierce_count: u8
- tick_rate: Option<f32> (for continuous effects)
- duration: Option<f32> (for continuous effects)
- radius: Option<f32> (for area effects)
- visual_id: AssetId
- audio_id: AssetId

**DeliveryMethod** enum:
- Projectile (MovingEffect with velocity)
- Beam (ActiveEffect with raycast)
- Area (ActiveEffect with radius)
- Instant (immediate EffectEvent, no entity)

**EffectRegistry** resource:
- HashMap<EffectDefId, EffectDefinition>
- Hot-reload support: watch file changes, rebuild registry
- Validation: check for missing assets, invalid ranges

**Effect Definition Editor:**
RON format, human-editable:
```
(
  id: "fireball",
  damage: 25.0,
  damage_type: Magical,
  knockback: 150.0,
  effect_type: Moving,
  delivery: Projectile,
  pierce_count: 0,
  visual_id: "fireball_projectile",
  audio_id: "fire_cast",
)
```

Hot-reload watcher updates registry on file save. Iterate balance without recompile.

**0.d: Effect Spawner**

```
spawn_effect(
  commands: &mut Commands,
  registry: &EffectRegistry,
  definition_id: EffectDefId,
  position: Vec2,
  direction: Vec2,
  source: Entity,
  faction: FactionId
)
```

Reads definition, branches on effect_type:
- **Instant**: Emit EffectEvent immediately, no entity
- **Moving**: Spawn with MovingEffect + velocity
- **Continuous**: Spawn with ActiveEffect + timer

Checks for SimpleProjectile optimization: if no pierce, no special behavior, use fast path.

**0.e: System Architecture**

**Moving Effect Pipeline:**
1. MovingEffectCollision: detects hits, emits EffectEvent
2. Resolvers: process EffectEvent → specific events
3. Appliers: modify components
4. MovingEffectCleanup: despawn expired/depleted effects

**Continuous Effect Pipeline:**
1. ActiveEffectTick: raycast/area check, find targets, emit EffectEvent
2. Resolvers: same as above
3. Appliers: same as above
4. ActiveEffectCleanup: despawn expired effects

**SimpleProjectile Fast Path:**
1. SimpleProjectileCollision: detect hit, calculate damage directly
2. Emit DamageEvent (skip EffectEvent routing)
3. DamageApplier: modify health
4. Despawn projectile

Parallel systems, no interference.

**0.f: Debug Tooling (BUILD EARLY)**

**Debug Commands:**
- `/spawn_effect <id> <direction>` - spawn at cursor
- `/list_effects` - show all registered effects
- `/effect_stats` - current entity counts by type
- `/reload_effects` - force hot-reload

**Performance Metrics** (UI overlay):
- Active MovingEffect count
- Active ActiveEffect count
- SimpleProjectile count
- EffectEvent count this frame
- Average resolution time (ms)

**Balance Tweaking UI:**
- Live edit effect values
- Save changes to definition file
- See changes immediately

**0.g: Data Migration**

After all systems tested:

1. Create effect_definitions.ron with all current attacks
2. Add EffectDefId to enemy archetypes
3. Add EffectDefId to player attack config
4. Test one enemy type with new system
5. Test player with new system
6. Verify identical behavior
7. Migrate remaining enemies
8. Delete old code: Team enum, Projectile/Grenade, spawn functions

**Deliverable Checklist:**
- [ ] EffectRegistry with hot-reload working
- [ ] Resolution pipeline handles multi-hit correctly
- [ ] MovingEffect and ActiveEffect systems independent
- [ ] SimpleProjectile optimization measurably faster
- [ ] Debug commands functional
- [ ] Performance metrics visible
- [ ] Effect editor workflow smooth
- [ ] All enemies migrated
- [ ] Player migrated
- [ ] Old code deleted

---

### Phase 1: Content Variety

**Goal:** Use new foundation to add diverse attacks without code changes.

**1.a: Delivery Method Implementations**

**Projectile** (MovingEffect):
- Linear travel with velocity
- Collision-based triggering
- Pierce logic built-in

**Beam** (ActiveEffect):
- Raycast from source to max range
- Continuous damage while active
- Visual: line renderer from source to hit point
- Stops at first obstacle or max range

**Area** (ActiveEffect):
- Radius check from spawn point
- Continuous damage to all in radius
- Visual: expanding circle or persistent effect
- Can be mobile (attached to entity) or stationary

**Instant** (no entity):
- Immediate damage at target point
- Used for melee attacks, triggers
- No travel time, no entity overhead

**1.b: AI Attack Selection**

**AttackSet** component on enemies:
- attacks: Vec<AttackConfig>

**AttackConfig**:
- effect_id: EffectDefId
- min_range: f32
- max_range: f32
- cooldown: Timer
- priority: u8 (higher = prefer when multiple valid)

**AI Attack System:**
```
1. Filter attacks where player in range
2. Filter attacks where cooldown finished
3. Pick highest priority
4. If none valid, use default movement behavior
5. Trigger via spawn_effect
6. Reset cooldown
```

Simple, data-driven, no special cases per enemy type.

**1.c: Attack Variety (Data Only)**

Create 15+ effects demonstrating system capabilities:

**Fast/Weak:**
- "rapid_shot": 5 damage, 0.1s cooldown, projectile
- "machine_gun": 3 damage, 0.05s cooldown, projectile with spread

**Slow/Strong:**
- "sniper_shot": 50 damage, 2s cooldown, fast projectile
- "heavy_blast": 75 damage, 3s cooldown, slow projectile

**Area Control:**
- "poison_pool": 5 dps, 5s duration, area radius 30
- "fire_wall": 10 dps, 3s duration, area radius 50

**Utility:**
- "chain_lightning": 15 damage, pierce 5, projectile
- "shotgun_blast": spawn 8 projectiles in arc
- "laser_beam": 20 dps, 1s duration, beam

**Special:**
- "delayed_explosion": instant, spawn area effect after 1s delay
- "homing_missile": projectile with seeking behavior (future modifier)

**1.d: Enemy Diversity**

Assign attack sets to archetypes:

**SmallMelee**: rapid_shot (close range fallback)
**BigMelee**: heavy_blast (mid range)
**Shotgunner**: shotgun_blast (close-mid)
**Sniper**: sniper_shot (long range)
**MachineGunner**: machine_gun (mid range), grenade (long range)

Each gets 1-2 attacks, creates variety without new code.

**1.e: Performance Optimization**

**Object Pooling:**
- Pre-allocate 100 SimpleProjectile entities
- Reuse instead of spawn/despawn
- Measured: 30% reduction in frame time spikes

**Batch Processing:**
- Collect all EffectEvents per frame
- Process in single pass per resolver
- Reduces query overhead

**Spatial Hashing:**
- Grid-based collision for MovingEffect
- Only check entities in same/adjacent cells
- Measured: 50% faster collision detection at 200+ projectiles

**Effect Limits:**
- Max 500 active effects total
- Despawn oldest when limit reached
- Prevents runaway spawning

**1.f: Visual & Audio Polish**

**Per Damage Type:**
- Physical: gray particles, thud sound
- Magical: colored particles, whoosh sound
- True: no particles, silent (pure damage)

**Per Delivery:**
- Projectile: trail effect
- Beam: beam shader with glow
- Area: pulsing circle
- Instant: flash at impact point

**Damage Numbers:**
- Color by damage type
- Size scales with damage amount
- Critical hits (future): different color + animation

**Screen Effects:**
- Shake intensity = damage / max_health
- Flash on player hit
- Slow-mo on killing blow (optional)

---

### Phase 2: Player Progression

**Goal:** Player finds better attacks, not just stat upgrades.

**2.a: Attack Items**
- Weapons/spells as equippable items
- Each item has EffectDefinition
- Player can swap attacks
- Different attacks for different situations

**2.b: Loot System**
- Attacks drop from enemies
- Rarity tiers affect stats
- Depth scaling on drop quality

**2.c: Equipment Stats**
- Armor/rings modify CombatState
- Damage multipliers
- Resistances
- Attack speed
- Movement speed

**2.d: Build Diversity**
- Can player make meaningful choices?
- Are different builds viable?
- Does loot feel rewarding?

---

### Phase 3: Advanced Mechanics

**Goal:** Polish and depth without complexity bloat.

**3.a: Status Effects**
- Use StatusEffect framework
- Start simple: slow, stun, DoT
- No element interactions yet
- Test: do statuses create tactics?

**3.b: Destructible Terrain**
- Explosive effects damage walls
- Permanent holes in geometry
- Physics/pathing updates
- Balance cost vs utility

**3.c: Advanced AI**
- Enemies dodge slow projectiles
- Take cover from beams
- Rush when player reloading
- Positioning matters

**3.d: Final Polish**
- Particle systems
- Screen shake
- Camera effects
- Sound mix

---

## Future: Elements & Spells

**When Foundation is Solid:**

The current refactor builds everything needed for the element system:
- StatusEffect framework → elemental statuses (burning, frozen)
- EffectPayload → carries element type
- EffectResolver → handles element interactions
- Faction system → element-agnostic combat rules

**Next Big Step:**
- Add Element enum to EffectPayload
- Implement element interaction matrix in EffectResolver
- Add ground MaterialState system
- Build spell book/page system on top of EffectDefinition

Foundation doesn't care about elements. When we're ready, elements slot in cleanly without refactoring combat core.

---

## Design Decisions

**Why generic effects over hardcoded weapons?**
Infinite variety from finite code. Adding attacks becomes data entry, not programming.

**Why separate Faction from combat logic?**
Multiplayer, neutral NPCs, wildlife, temporary alliances—all possible without touching combat systems.

**Why EffectPayload instead of specific components?**
Composition over inheritance. Mix damage + knockback + slow in one effect without new component types.

**Why StatusEffect framework now if not using it yet?**
Infrastructure is hardest part. Having it ready means elements/debuffs are config changes, not refactors.

**Why no elements yet?**
Risk mitigation. Prove foundation works with simple attacks before adding interaction complexity.

---

## Success Criteria

Foundation succeeds when:
- Adding new attack type = creating EffectDefinition, no code
- Combat feels better than before refactor
- Zero references to "projectile" or "grenade" in core systems
- Can add elements later without touching combat core
- Enemy variety created through data, not new archetypes

Foundation fails if:
- Need code changes to add attack variety
- Combat feels worse than before
- Special cases creep back in
- Would need refactor to add elements

# How the New Combat System Works

### The Core Idea
Instead of hardcoded "bullets" and "grenades", everything is now an **effect** - a bundle of data that says "do X damage, knock back Y amount, maybe apply Z status." 

When you shoot, cast a spell, or an enemy attacks, they're all just triggering effects with different numbers.

### The Flow

**When you click to attack:**
1. Game looks up your current weapon/spell's stats (EffectDefinition)
2. Spawns an effect entity based on those stats
3. Effect travels toward target (or instantly applies, depending on type)
4. On hit: calculates damage based on your stats vs their resistances
5. Applies damage, knockback, any status effects
6. Shows visuals (particles, damage numbers, etc)

**For enemies:** Same exact flow, they just trigger effects from their AI instead of mouse clicks.

### Effect Types

**Moving Effects** (projectiles, thrown objects)
- Travel through space
- Hit things via collision
- Can pierce through multiple enemies
- Examples: arrows, fireballs, rockets

**Active Effects** (sustained damage zones)
- Exist for a duration
- Damage anything inside their area
- Examples: poison clouds, laser beams, auras

**Instant Effects** (immediate impact)
- No travel time, no entity spawned
- Just "target takes damage now"
- Examples: melee hits, hitscan shots

### The Smart Parts

**Factions instead of teams:** Instead of hardcoded "player team vs enemy team", entities belong to factions that can be hostile, neutral, or allied. This means we could add wildlife, NPCs, or PvP without touching combat code.

**Damage types & resistances:** Physical, Magical, or True damage. Enemies can resist certain types. Your armor might give 50% physical resistance but 0% magical. Creates tactical choices.

**Performance optimization:**
- Simple bullets use a fast-path system (SimpleProjectile)
- Complex effects use the full system
- Everything is pooled and recycled
- Hard cap at 500 effects (oldest get deleted)

### What This Enables

**For players:**
- Find weapons with different attack patterns
- Build around damage types (physical vs magic build)
- Equipment that modifies your stats and resistances
- Varied enemy encounters based on their attacks

**For development:**
- New attacks = edit a text file, no coding
- Enemy variety through different effect combinations
- Hot-reload for instant balance testing
- Clean foundation for future element system

### The Guardrails

The system actively prevents itself from becoming a mess:
- No special-case code for specific effects
- Hard performance limits that can't be removed
- Clear scope boundaries (no "while we're at it" additions)
- Everything must be expressible as data

### What It Replaced

The old system had:
- Separate code for projectiles vs grenades
- Hardcoded damage values scattered everywhere  
- Team enum forcing everything to be player or enemy
- No way to add variety without programming

Now it's all unified, data-driven, and extensible. Adding a new attack type means adding a few lines to a config file, not writing new systems.
