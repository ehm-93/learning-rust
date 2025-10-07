-- Simplified composite behavior using the new api.behaviors.compose() helper
-- This demonstrates how to create a composite behavior with much less boilerplate

-- Note: This assumes turn_toward and accelerate_forward behaviors exist
-- The compose helper will automatically pass through all lifecycle calls

-- Define the composite behavior (this is the BehaviorDefinition)
api.behaviors.compose(
    "seeker",                      -- Composite behavior name
    {                              -- List of sub-behavior names to compose
        "turn_toward",
        "accelerate_forward"
    }
)

-- Later, when you want to use it, instantiate with params:
-- local seeker = api.behaviors.create("seeker", {
--     search_radius = 100.0,
--     turn_speed = 8.0,
--     acceleration = 100.0,
--     max_speed = 60.0
-- })
--
-- The params will be passed to both turn_toward and accelerate_forward factories!

-- That's it! The compose helper creates a behavior definition that:
-- - Accepts params at instantiation time (like any other behavior)
-- - Creates instances of each sub-behavior with those params
-- - Passes through all lifecycle calls (on_spawn, update, on_despawn, collision events)
-- - Handles the boilerplate of calling each sub-behavior in order
--
-- This is equivalent to the manual implementation but much more concise!
