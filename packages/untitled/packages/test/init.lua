-- Test package for Phase 2 stress testing
-- Demonstrates Lua behaviors and composition with Rust behaviors
-- Organized with separate files for each behavior

api.log("🚀 Test package loading...")

-- Note: For now, just inline all behaviors until we can pass the package root
-- In the future, we'll use proper module loading
api.log("📦 Loading behaviors (inline mode - split files available for reference)")

-- Load foundational behaviors
api.log("📦 Loading foundational behaviors...")
load_behavior("behaviors/foundational/turn_toward.lua")
load_behavior("behaviors/foundational/accelerate_forward.lua")
load_behavior("behaviors/foundational/impulse.lua")
load_behavior("behaviors/foundational/constant_spin.lua")
load_behavior("behaviors/foundational/spinner.lua")

-- Load composite behaviors
api.log("📦 Loading composite behaviors...")
load_behavior("behaviors/composite/seeker.lua")
load_behavior("behaviors/composite/spinning_seeker.lua")
load_behavior("behaviors/composite/spinning_projectile.lua")
load_behavior("behaviors/composite/pulse_and_die.lua")
load_behavior("behaviors/composite/collision_logger.lua")

-- Load spawning behaviors
api.log("📦 Loading spawning behaviors...")
load_behavior("behaviors/spawning/spawn_radial.lua")
load_behavior("behaviors/spawning/spawn_radial_once.lua")

api.log("✅ Test package loaded successfully!")
api.log("📦 Foundational behaviors: turn_toward, accelerate_forward, impulse, constant_spin, spinner")
api.log("📦 Spawning behaviors: spawn_radial, spawn_radial_once")
api.log("📦 Composite behaviors: seeker, spinning_seeker, spinning_projectile, pulse_and_die, collision_logger")
