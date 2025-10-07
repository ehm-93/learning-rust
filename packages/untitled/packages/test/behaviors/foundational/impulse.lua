-- Foundational behavior: Set velocity once (impulse-like)
-- Does not maintain velocity, lets physics take over
api.behaviors.register("impulse", function(params)
    local entity_id = params.entity_id
    local velocity_x = params.velocity_x or 0.0
    local velocity_y = params.velocity_y or 0.0
    local use_facing = params.use_facing or false
    local speed = params.speed or 50.0

    return {
        on_spawn = function(world)
            local final_vx = velocity_x
            local final_vy = velocity_y

            -- If use_facing is true, calculate velocity from entity's rotation
            if use_facing then
                local rot = world:get_rotation()
                if rot then
                    final_vx = math.cos(rot.yaw) * speed
                    final_vy = math.sin(rot.yaw) * speed
                end
            end

            -- Set velocity once
            world:set_velocity(final_vx, final_vy)
            world:log("💨 Impulse applied (" .. string.format("%.1f", final_vx) .. ", " .. string.format("%.1f", final_vy) .. ")")
        end,

        update = function(world, dt)
            -- No update needed - velocity persists via physics
        end
    }
end)
