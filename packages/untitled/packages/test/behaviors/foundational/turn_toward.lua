-- Foundational behavior: Turn toward a target position
api.behaviors.register("turn_toward", function(params)
    local entity_id = params.entity_id
    local turn_speed = params.turn_speed or 5.0
    local target_x = params.target_x or 0.0
    local target_y = params.target_y or 0.0
    -- If search_radius is provided, will find nearest entity instead of using fixed target
    local search_radius = params.search_radius or nil

    return {
        on_spawn = function(world)
            if search_radius then
                world:log("🎯 Turn toward (dynamic targeting, radius: " .. search_radius .. ")")
            else
                world:log("🎯 Turn toward (" .. target_x .. ", " .. target_y .. ")")
            end
        end,

        update = function(world, dt)
            local my_pos = world:get_position()
            if not my_pos then return end

            local target_pos_x = target_x
            local target_pos_y = target_y

            -- If search_radius is set, find nearest entity
            if search_radius then
                local nearby = world:query_nearby(search_radius)
                if nearby and #nearby > 0 then
                    local closest = nearby[1]
                    for i = 2, #nearby do
                        if nearby[i].distance < closest.distance then
                            closest = nearby[i]
                        end
                    end
                    target_pos_x = closest.x
                    target_pos_y = closest.y
                else
                    return -- No target found
                end
            end

            -- Calculate angle to target
            local dx = target_pos_x - my_pos.x
            local dy = target_pos_y - my_pos.y
            local target_angle = math.atan2(dy, dx)

            -- Get current rotation
            local rot = world:get_rotation()
            if not rot then return end

            -- Calculate angle difference
            local current_angle = rot.yaw
            local angle_diff = target_angle - current_angle

            -- Normalize to [-pi, pi]
            while angle_diff > math.pi do
                angle_diff = angle_diff - 2 * math.pi
            end
            while angle_diff < -math.pi do
                angle_diff = angle_diff + 2 * math.pi
            end

            -- Apply angular velocity to turn toward target
            if math.abs(angle_diff) > 0.01 then
                local angular_vel = angle_diff * turn_speed
                -- Clamp to prevent overshooting
                local max_vel = turn_speed * 2
                if angular_vel > max_vel then angular_vel = max_vel end
                if angular_vel < -max_vel then angular_vel = -max_vel end
                world:set_angular_velocity(angular_vel)
            else
                world:set_angular_velocity(0)
            end
        end
    }
end)
