-- Foundational behavior: Constant angular velocity (spin forever)
api.behaviors.register("constant_spin", function(params)
    local entity_id = params.entity_id
    local angular_velocity = params.angular_velocity or 2.0

    return {
        on_spawn = function(world)
            world:set_angular_velocity(angular_velocity)
            world:log("🌀 Constant spin (" .. angular_velocity .. " rad/s)")
        end,

        update = function(world, dt)
            -- Maintain constant angular velocity
            world:set_angular_velocity(angular_velocity)
        end
    }
end)
