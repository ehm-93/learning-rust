# Behavior System Specification

## Overview
Composable action system for Bevy ECS with Lua modding support. Behaviors are lifecycle-aware components that can be atomic or composite. Supports both compile-time tuple composition (Rust) and runtime dynamic composition (Lua).

## Core Types

```rust
type Params = HashMap<String, ParamValue>;

enum ParamValue {
  Float(f32),
  Int(i32),
  Vec3(Vec3),
  String(String),
  EntityId(Entity),
  BehaviorId(String),  // Reference to another behavior
}

// Single trait for all behaviors
trait Behavior {
  fn on_spawn(&mut self, world: &mut WorldApi) {}
  fn on_update(&mut self, world: &mut WorldApi, dt: f32) {}
  fn on_despawn(&mut self, world: &mut WorldApi) {}
  fn on_collision_enter(&mut self, world: &mut WorldApi, other: Entity) {}
  fn on_collision_stay(&mut self, world: &mut WorldApi, other: Entity) {}
  fn on_collision_exit(&mut self, world: &mut WorldApi, other: Entity) {}
}

// Constructor that captures params and returns behavior
type BehaviorDefinition = fn(Params) -> Box<dyn Behavior>;

// World API for behaviors to interact with ECS
struct WorldApi {
  entity: Entity,
  world: &mut World,
}

impl WorldApi {
  fn add_component<T: Component>(&mut self, component: T);
  fn get_component<T: Component>(&self) -> Option<&T>;
  fn get_component_mut<T: Component>(&mut self) -> Option<&mut T>;
  fn spawn_behavior(&mut self, name: &str, params: Params) -> Entity;
  fn despawn(&mut self);
}
```

## Behavior Composition

### CompositeBehavior - Runtime Composition

```rust
// CompositeBehavior - composes behaviors at runtime (used by Lua)
struct CompositeBehavior {
  behaviors: Vec<Box<dyn Behavior>>,
}

impl Behavior for CompositeBehavior {
  fn on_spawn(&mut self, world: &mut WorldApi) {
    for behavior in &mut self.behaviors {
      behavior.on_spawn(world);
    }
  }
  
  fn on_update(&mut self, world: &mut WorldApi, dt: f32) {
    for behavior in &mut self.behaviors {
      behavior.on_update(world, dt);
    }
  }
  
  fn on_despawn(&mut self, world: &mut WorldApi) {
    for behavior in &mut self.behaviors {
      behavior.on_despawn(world);
    }
  }
  
  fn on_collision_enter(&mut self, world: &mut WorldApi, other: Entity) {
    for behavior in &mut self.behaviors {
      behavior.on_collision_enter(world, other);
    }
  }
  
  fn on_collision_stay(&mut self, world: &mut WorldApi, other: Entity) {
    for behavior in &mut self.behaviors {
      behavior.on_collision_stay(world, other);
    }
  }
  
  fn on_collision_exit(&mut self, world: &mut WorldApi, other: Entity) {
    for behavior in &mut self.behaviors {
      behavior.on_collision_exit(world, other);
    }
  }
}
```

### IntoBehaviors - Compile-time Tuple Composition

```rust
// Trait for converting definitions into behavior lists (used by Rust)
trait IntoBehaviors {
  fn into_behaviors(self, params: &Params, registry: &BehaviorRegistry) -> Vec<Box<dyn Behavior>>;
}

// Single definition
impl IntoBehaviors for BehaviorDefinition {
  fn into_behaviors(self, params: &Params, _registry: &BehaviorRegistry) -> Vec<Box<dyn Behavior>> {
    vec![self(params.clone())]
  }
}

// Tuple of definitions (implement for tuples up to length 12)
impl<T1, T2> IntoBehaviors for (T1, T2) 
where 
  T1: IntoBehaviors, 
  T2: IntoBehaviors 
{
  fn into_behaviors(self, params: &Params, registry: &BehaviorRegistry) -> Vec<Box<dyn Behavior>> {
    let mut behaviors = self.0.into_behaviors(params, registry);
    behaviors.extend(self.1.into_behaviors(params, registry));
    behaviors
  }
}

// Similar impls for (T1, T2, T3), (T1, T2, T3, T4), etc. up to 12 elements
```

## Registry System

```rust
struct BehaviorRegistry {
  behaviors: HashMap<String, Box<dyn Fn(&Params) -> Vec<Box<dyn Behavior>>>>,
}

impl BehaviorRegistry {
  // Register with tuple syntax (Rust)
  fn register<T: IntoBehaviors + 'static>(&mut self, name: &str, defs: T) {
    let factory = Box::new(move |params: &Params| {
      defs.into_behaviors(params, self)
    });
    self.behaviors.insert(name.to_string(), factory);
  }
  
  // Instantiate by name
  fn instantiate(&self, name: &str, params: Params) -> Vec<Box<dyn Behavior>>;
  
  // Runtime composition (Lua)
  fn compose(&self, specs: Vec<(String, Params)>) -> Box<dyn Behavior> {
    let behaviors = specs.iter()
      .flat_map(|(name, params)| self.instantiate(name, params.clone()))
      .collect();
    Box::new(CompositeBehavior { behaviors })
  }
  
  // Spawn entity with behavior
  fn spawn(&self, name: &str, params: Params, world: &mut World) -> Entity;
}
```

## Native Rust Example

```rust
// Movement behavior - atomic, uses on_spawn
struct MovementBehavior {
  speed: f32,
  direction: Vec3,
}

impl Behavior for MovementBehavior {
  fn on_spawn(&mut self, world: &mut WorldApi) {
    world.add_component(Velocity {
      value: self.direction.normalize() * self.speed
    });
  }
}

fn movement_def(params: Params) -> Box<dyn Behavior> {
  Box::new(MovementBehavior {
    speed: params.get_f32("speed").unwrap_or(10.0),
    direction: params.get_vec3("direction").unwrap_or(Vec3::ZERO),
  })
}

// Homing behavior - atomic, uses on_update
struct HomingBehavior {
  target: Entity,
  strength: f32,
}

impl Behavior for HomingBehavior {
  fn on_update(&mut self, world: &mut WorldApi, dt: f32) {
    if let Some(mut velocity) = world.get_component_mut::<Velocity>() {
      if let Some(target_transform) = world.world.get::<Transform>(self.target) {
        let current_transform = world.get_component::<Transform>().unwrap();
        let direction = (target_transform.translation - current_transform.translation).normalize();
        velocity.value = velocity.value.lerp(direction * velocity.value.length(), self.strength * dt);
      }
    }
  }
}

fn homing_def(params: Params) -> Box<dyn Behavior> {
  Box::new(HomingBehavior {
    target: params.get_entity("target").unwrap(),
    strength: params.get_f32("strength").unwrap_or(5.0),
  })
}

// OnHit behavior - atomic, uses collision hooks
struct OnHitBehavior {
  spawn_behavior: String,
  damage: f32,
}

impl Behavior for OnHitBehavior {
  fn on_collision_enter(&mut self, world: &mut WorldApi, other: Entity) {
    let position = world.get_component::<Transform>().unwrap().translation;
    world.spawn_behavior(&self.spawn_behavior, Params::from([
      ("position", position.into()),
      ("damage", self.damage.into()),
    ]));
    world.despawn();
  }
}

fn on_hit_def(params: Params) -> Box<dyn Behavior> {
  Box::new(OnHitBehavior {
    spawn_behavior: params.get_behavior_id("spawn_behavior").unwrap(),
    damage: params.get_f32("damage").unwrap_or(0.0),
  })
}

// Register behaviors
fn setup(mut registry: ResMut<BehaviorRegistry>) {
  // Atomic behaviors
  registry.register("movement", movement_def);
  registry.register("homing", homing_def);
  registry.register("on_hit", on_hit_def);
  
  // Composite behaviors using tuple syntax (Rust)
  registry.register("fireball", (
    movement_def,
    on_hit_def,
  ));
  
  registry.register("homing_missile", (
    movement_def,
    homing_def,
    on_hit_def,
  ));
  
  // Or inline
  registry.register("explosion", (
    |params: Params| -> Box<dyn Behavior> {
      Box::new(DamageAreaBehavior {
        radius: params.get_f32("radius").unwrap_or(5.0),
        damage: params.get_f32("damage").unwrap_or(100.0),
      })
    },
    |params: Params| -> Box<dyn Behavior> {
      Box::new(TimeoutBehavior {
        duration: params.get_f32("duration").unwrap_or(0.5),
      })
    },
  ));
}
```

## Bevy Systems Integration

```rust
// Component to store behavior instances on entity
#[derive(Component)]
struct BehaviorComponent {
  behaviors: Vec<Box<dyn Behavior>>,
}

impl BehaviorComponent {
  fn on_spawn(&mut self, world: &mut WorldApi) {
    for behavior in &mut self.behaviors {
      behavior.on_spawn(world);
    }
  }
  
  fn on_update(&mut self, world: &mut WorldApi, dt: f32) {
    for behavior in &mut self.behaviors {
      behavior.on_update(world, dt);
    }
  }
  
  fn on_despawn(&mut self, world: &mut WorldApi) {
    for behavior in &mut self.behaviors {
      behavior.on_despawn(world);
    }
  }
  
  fn on_collision_enter(&mut self, world: &mut WorldApi, other: Entity) {
    for behavior in &mut self.behaviors {
      behavior.on_collision_enter(world, other);
    }
  }
  
  fn on_collision_stay(&mut self, world: &mut WorldApi, other: Entity) {
    for behavior in &mut self.behaviors {
      behavior.on_collision_stay(world, other);
    }
  }
  
  fn on_collision_exit(&mut self, world: &mut WorldApi, other: Entity) {
    for behavior in &mut self.behaviors {
      behavior.on_collision_exit(world, other);
    }
  }
}

// System to call on_update for all behaviors
fn update_behaviors(
  mut query: Query<(Entity, &mut BehaviorComponent)>,
  mut world: World,
  time: Res<Time>,
) {
  for (entity, mut behavior_comp) in query.iter_mut() {
    let mut api = WorldApi { entity, world: &mut world };
    behavior_comp.on_update(&mut api, time.delta_seconds());
  }
}

// System to handle collision events and call hooks (Rapier integration)
fn handle_collisions(
  mut collision_events: EventReader<CollisionEvent>,
  mut query: Query<&mut BehaviorComponent>,
  mut world: World,
) {
  for event in collision_events.read() {
    match event {
      CollisionEvent::Started(e1, e2, _flags) => {
        if let Ok(mut behavior) = query.get_mut(*e1) {
          let mut api = WorldApi { entity: *e1, world: &mut world };
          behavior.on_collision_enter(&mut api, *e2);
        }
        if let Ok(mut behavior) = query.get_mut(*e2) {
          let mut api = WorldApi { entity: *e2, world: &mut world };
          behavior.on_collision_enter(&mut api, *e1);
        }
      },
      CollisionEvent::Stopped(e1, e2, _flags) => {
        if let Ok(mut behavior) = query.get_mut(*e1) {
          let mut api = WorldApi { entity: *e1, world: &mut world };
          behavior.on_collision_exit(&mut api, *e2);
        }
        if let Ok(mut behavior) = query.get_mut(*e2) {
          let mut api = WorldApi { entity: *e2, world: &mut world };
          behavior.on_collision_exit(&mut api, *e1);
        }
      },
    }
  }
}
```

## Lua Integration (Future)

### Design Constraints for Lua Compatibility

Both composition patterns support Lua:

**Key compatibility points:**
- `CompositeBehavior` for runtime composition - Lua can call `compose()`
- Single `Behavior` trait to wrap - Lua tables implement hooks
- `Params` as `HashMap` - direct Lua table mapping
- String-based registry - Lua can reference Rust behaviors by name

**Lua integration path (deferred):**
```rust
struct LuaBehavior {
  lua_table: LuaTable,
}

impl Behavior for LuaBehavior {
  fn on_spawn(&mut self, world: &mut WorldApi) {
    if let Ok(func) = self.lua_table.get::<_, LuaFunction>("on_spawn") {
      func.call::<_, ()>(world_to_lua(world)).ok();
    }
  }
  // ... similar for other hooks
}

fn register_lua_behavior(name: String, lua_fn: LuaFunction) -> BehaviorDefinition {
  move |params| {
    let lua_table = lua_fn.call::<_, LuaTable>(params_to_lua(&params)).unwrap();
    Box::new(LuaBehavior { lua_table })
  }
}

// Expose compose() to Lua
fn lua_compose(registry: &BehaviorRegistry, specs: LuaTable) -> Box<dyn Behavior> {
  let specs: Vec<(String, Params)> = specs.pairs()
    .map(|pair: (usize, LuaTable)| {
      let (_, spec) = pair.unwrap();
      let name = spec.get::<_, String>("name").unwrap();
      let params = lua_table_to_params(spec);
      (name, params)
    })
    .collect();
  registry.compose(specs)
}
```

**Example Lua API (target design):**
```lua
-- Atomic behavior
register_behavior("spiral", function(params)
  return {
    on_spawn = function(world)
      world:add_component("SpiralVelocity", {
        speed = params.speed,
        rotation = params.rotation
      })
    end
  }
end)

-- Composite behavior using runtime composition
register_behavior("magic_missile", function(params)
  return compose({
    { name = "movement", speed = params.speed, direction = params.direction },
    { name = "homing", target = params.target },
    { name = "spiral", rotation = 2.0 },
    { name = "on_hit", spawn_behavior = "explosion", damage = params.damage }
  })
end)

-- Spawn behavior
spawn_behavior("magic_missile", {
  position = Vec3(0, 0, 0),
  speed = 20.0,
  direction = Vec3(1, 0, 0),
  target = player_entity,
  damage = 100
})
```

Current implementation focuses on native Rust. Lua bridge added later without architectural changes.

## Param Helper Methods

```rust
impl Params {
  fn get_f32(&self, key: &str) -> Result<f32>;
  fn get_int(&self, key: &str) -> Result<i32>;
  fn get_vec3(&self, key: &str) -> Result<Vec3>;
  fn get_string(&self, key: &str) -> Result<String>;
  fn get_entity(&self, key: &str) -> Result<Entity>;
  fn get_behavior_id(&self, key: &str) -> Result<String>;
  
  fn subset(&self, keys: &[&str]) -> Params;
}
```

## Execution Flow

1. **Registration**: 
   - Rust: `register("name", (def1, def2))` - tuple converted to behavior list
   - Lua: `register("name", fn)` - fn returns `compose(...)` result
2. **Instantiation**: Registry factory returns `Vec<Box<dyn Behavior>>`
3. **Spawn**: Create entity, add `BehaviorComponent`, call `on_spawn()` on all behaviors
4. **Update Loop**: Bevy system calls `on_update(dt)` on all behaviors each frame
5. **Collision**: Rapier fires events, calls collision hooks on all behaviors
6. **Cleanup**: Call `on_despawn()` on all behaviors, remove entity

```
BehaviorRegistry::spawn("fireball", params)
  ↓ Rust tuple registration
(movement_def, on_hit_def).into_behaviors(params) -> Vec<Box<dyn Behavior>>
  ↓ OR Lua runtime composition
registry.compose([("movement", params), ("on_hit", params)]) -> CompositeBehavior
  ↓ spawn entity with BehaviorComponent
entity.behaviors[0].on_spawn(world) + entity.behaviors[1].on_spawn(world)
  ↓ each frame
for behavior in entity.behaviors { behavior.on_update(world, dt) }
  ↓ on collision
for behavior in entity.behaviors { behavior.on_collision_enter(world, other) }
  ↓ cleanup
for behavior in entity.behaviors { behavior.on_despawn(world) }
```

## Implementation Prompt

Implement this dual-composition behavior system with the following requirements:

1. **Core Trait**: 
   - Define `Behavior` trait with 6 lifecycle hooks (all default empty impls)
   - Define `BehaviorDefinition` type alias
   - Implement `CompositeBehavior` for runtime composition
   - Implement `IntoBehaviors` trait for single definitions and tuples (up to 12 elements)

2. **Core Data Structures**: 
   - Implement `Params`, `ParamValue` with helper methods
   - Implement `WorldApi` wrapper around Bevy `World`
   - Implement `BehaviorComponent` storing `Vec<Box<dyn Behavior>>` with delegation methods

3. **Registry System**: 
   - Build `BehaviorRegistry` as Bevy Resource
   - Implement `register()` accepting `impl IntoBehaviors` (for Rust tuple syntax)
   - Implement `compose()` for runtime composition (for Lua)
   - Implement `instantiate()` returning `Vec<Box<dyn Behavior>>`
   - `spawn()` creates entity, adds `BehaviorComponent`, calls `on_spawn()`

4. **Atomic Behaviors**: Implement core behaviors as structs:
   - `MovementBehavior` - uses `on_spawn` to add Velocity component
   - `HomingBehavior` - uses `on_update` to adjust velocity toward target
   - `OnHitBehavior` - uses `on_collision_enter` to spawn behavior and despawn
   - `DamageAreaBehavior` - uses `on_spawn` to add area damage component
   - `TimeoutBehavior` - uses `on_update` to track duration and despawn

5. **Composite Behaviors**: Implement example composite registrations:
   - `projectile` - `(movement_def, on_hit_def, timeout_def)`
   - `explosion` - `(damage_area_def, timeout_def)`
   - `homing_missile` - `(movement_def, homing_def, on_hit_def)`

6. **Bevy Systems**: 
   - `update_behaviors` - calls `on_update()` on all behaviors in BehaviorComponents each frame
   - `handle_collisions` - listens to Rapier `CollisionEvent`, calls collision hooks on both entities
   - `spawn_behaviors` - handles behavior spawning queue
   - Integration with Rapier physics/collision detection

7. **WorldApi Methods**:
   - `add_component<T>()` - insert component on entity
   - `get_component<T>()` - read component from entity
   - `get_component_mut<T>()` - mutably access component
   - `spawn_behavior()` - create new entity with behavior
   - `despawn()` - remove entity

8. **Error Handling**: Graceful handling of missing params, invalid behavior names

Focus on supporting both composition patterns: tuple syntax for Rust ergonomics, `CompositeBehavior` + `compose()` for Lua runtime composition. The system should work seamlessly with both approaches.
