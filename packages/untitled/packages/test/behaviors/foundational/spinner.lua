-- Simple behavior: Spinner - rotates the entity using angular velocity
api.behaviors.register("spinner", function(params)
    local entity_id = params.entity_id
    local speed = params.rotation_speed or 2.0

    return {
        on_spawn = function(world)
            world:log("🔄 Spinner behavior spawned (speed: " .. speed .. ")")
            -- Set angular velocity once
            world:set_angular_velocity(speed)
        end,

        update = function(world, dt)
            -- Angular velocity is continuous, no need to update each frame
            -- unless we want to change the speed dynamically
        end
    }
end)
