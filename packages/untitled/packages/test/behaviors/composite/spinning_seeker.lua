-- Composite behavior: Combines constant spinning with seeking
-- Demonstrates composing multiple foundational behaviors
api.behaviors.register("spinning_seeker", function(params)
    local entity_id = params.entity_id
    local rotation_speed = params.rotation_speed or 3.0
    local search_radius = params.search_radius or 200.0
    local turn_speed = params.turn_speed or 8.0
    local acceleration = params.acceleration or 100.0
    local max_speed = params.max_speed or 60.0

    -- Create sub-behaviors
    local spinner = api.behaviors.create("spinner", {
        entity_id = entity_id,
        rotation_speed = rotation_speed
    })

    local turn_behavior = api.behaviors.create("turn_toward", {
        entity_id = entity_id,
        turn_speed = turn_speed,
        search_radius = search_radius
    })

    local accel_behavior = api.behaviors.create("accelerate_forward", {
        entity_id = entity_id,
        acceleration = acceleration,
        max_speed = max_speed
    })

    return {
        on_spawn = function(world)
            world:log("🌀 Spinning Seeker spawned (composite: spinner + turn + accelerate)")
            if spinner and spinner.on_spawn then
                spinner.on_spawn(world)
            end
            if turn_behavior and turn_behavior.on_spawn then
                turn_behavior.on_spawn(world)
            end
            if accel_behavior and accel_behavior.on_spawn then
                accel_behavior.on_spawn(world)
            end
        end,

        update = function(world, dt)
            -- Execute all sub-behaviors
            if spinner and spinner.update then
                spinner.update(world, dt)
            end
            if turn_behavior and turn_behavior.update then
                turn_behavior.update(world, dt)
            end
            if accel_behavior and accel_behavior.update then
                accel_behavior.update(world, dt)
            end
        end,

        on_despawn = function(world)
            if spinner and spinner.on_despawn then
                spinner.on_despawn(world)
            end
            if turn_behavior and turn_behavior.on_despawn then
                turn_behavior.on_despawn(world)
            end
            if accel_behavior and accel_behavior.on_despawn then
                accel_behavior.on_despawn(world)
            end
        end
    }
end)
