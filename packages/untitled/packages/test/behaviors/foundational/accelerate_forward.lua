-- Foundational behavior: Accelerate forward based on current rotation
api.behaviors.register("accelerate_forward", function(params)
    local entity_id = params.entity_id
    local acceleration = params.acceleration or 50.0
    local max_speed = params.max_speed or 100.0

    return {
        on_spawn = function(world)
            world:log("⚡ Accelerate forward (accel: " .. acceleration .. ", max: " .. max_speed .. ")")
        end,

        update = function(world, dt)
            local rot = world:get_rotation()
            if not rot then return end

            -- Calculate forward direction based on rotation
            local forward_x = math.cos(rot.yaw)
            local forward_y = math.sin(rot.yaw)

            -- Get current velocity
            local vel = world:get_velocity()
            if not vel then return end

            -- Apply acceleration in forward direction
            local new_vx = vel.x + forward_x * acceleration * dt
            local new_vy = vel.y + forward_y * acceleration * dt

            -- Clamp to max speed
            local speed = math.sqrt(new_vx * new_vx + new_vy * new_vy)
            if speed > max_speed then
                local scale = max_speed / speed
                new_vx = new_vx * scale
                new_vy = new_vy * scale
            end

            world:set_velocity(new_vx, new_vy)
        end
    }
end)
