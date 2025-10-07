-- Test package for Phase 2 stress testing
-- Demonstrates Lua behaviors and composition with Rust behaviors

api.log("🚀 Test package loading...")

-- Simple behavior: Spinner - rotates the entity
api.behaviors.register("spinner", function(params)
    local entity_id = params.entity_id
    local speed = params.rotation_speed or 2.0

    return {
        on_spawn = function(world)
            world:log("🔄 Spinner behavior spawned (speed: " .. speed .. ")")
        end,

        update = function(world, dt)
            local rot = world:get_rotation()
            if rot then
                world:set_rotation(rot.roll, rot.pitch, rot.yaw + speed * dt)
            end
        end
    }
end)

-- Behavior that slowly scales up then despawns
api.behaviors.register("pulse_and_die", function(params)
    local entity_id = params.entity_id
    local lifetime = params.lifetime or 3.0
    local elapsed = 0

    return {
        on_spawn = function(world)
            world:log("💓 Pulse behavior spawned (lifetime: " .. lifetime .. "s)")
        end,

        update = function(world, dt)
            elapsed = elapsed + dt

            -- Pulse scale over time (this would need a scale component in practice)
            local progress = elapsed / lifetime
            local scale = 1.0 + math.sin(progress * math.pi * 4) * 0.3

            -- Move in a circle
            local pos = world:get_position()
            if pos then
                local angle = elapsed * 2.0
                local radius = 2.0
                world:set_position(
                    math.cos(angle) * radius,
                    pos.y,
                    math.sin(angle) * radius
                )
            end

            -- Despawn when time is up
            if elapsed >= lifetime then
                world:log("💀 Pulse lifetime expired, despawning...")
                world:despawn()
            end
        end,

        on_despawn = function(world)
            world:log("👋 Pulse behavior cleaned up")
        end
    }
end)

-- Behavior that seeks nearby entities
api.behaviors.register("seeker", function(params)
    local entity_id = params.entity_id
    local search_radius = params.search_radius or 10.0
    local seek_speed = params.seek_speed or 3.0

    return {
        on_spawn = function(world)
            world:log("🎯 Seeker behavior spawned")
        end,

        update = function(world, dt)
            local nearby = world:query_nearby(search_radius)

            if nearby and #nearby > 0 then
                -- Find closest entity
                local closest = nearby[1]
                for i = 2, #nearby do
                    if nearby[i].distance < closest.distance then
                        closest = nearby[i]
                    end
                end

                -- Move toward it
                local my_pos = world:get_position()
                if my_pos and closest then
                    local dx = closest.x - my_pos.x
                    local dz = closest.z - my_pos.z
                    local dist = math.sqrt(dx*dx + dz*dz)

                    if dist > 0.1 then
                        local move_amount = seek_speed * dt
                        world:set_position(
                            my_pos.x + (dx / dist) * move_amount,
                            my_pos.y,
                            my_pos.z + (dz / dist) * move_amount
                        )
                    end
                end
            end
        end
    }
end)

-- Behavior that logs when it collides with things
api.behaviors.register("collision_logger", function(params)
    local entity_id = params.entity_id
    local collision_count = 0

    return {
        on_spawn = function(world)
            world:log("📋 Collision logger ready")
        end,

        on_collision_enter = function(world, other_entity)
            collision_count = collision_count + 1
            world:log("💥 Collision #" .. collision_count .. " with entity " .. tostring(other_entity))
        end,

        on_collision_exit = function(world, other_entity)
            world:log("👋 Stopped colliding with entity " .. tostring(other_entity))
        end,

        on_despawn = function(world)
            world:log("📊 Total collisions: " .. collision_count)
        end
    }
end)

-- Composite behavior: Combines spinner and seeker
-- Note: This would compose multiple behaviors if we had the composition system
api.behaviors.register("spinning_seeker", function(params)
    local entity_id = params.entity_id

    -- In a full implementation, this would instantiate and compose
    -- both spinner and seeker behaviors. For now, we'll implement both inline.
    local search_radius = 8.0
    local seek_speed = 2.0
    local rotation_speed = 3.0

    return {
        on_spawn = function(world)
            world:log("🌀 Spinning Seeker spawned (composite behavior)")
        end,

        update = function(world, dt)
            -- Seeker logic
            local nearby = world:query_nearby(search_radius)
            if nearby and #nearby > 0 then
                local closest = nearby[1]
                for i = 2, #nearby do
                    if nearby[i].distance < closest.distance then
                        closest = nearby[i]
                    end
                end

                local my_pos = world:get_position()
                if my_pos and closest then
                    local dx = closest.x - my_pos.x
                    local dz = closest.z - my_pos.z
                    local dist = math.sqrt(dx*dx + dz*dz)

                    if dist > 0.1 then
                        local move_amount = seek_speed * dt
                        world:set_position(
                            my_pos.x + (dx / dist) * move_amount,
                            my_pos.y,
                            my_pos.z + (dz / dist) * move_amount
                        )
                    end
                end
            end

            -- Spinner logic
            local rot = world:get_rotation()
            if rot then
                world:set_rotation(rot.roll, rot.pitch, rot.yaw + rotation_speed * dt)
            end
        end
    }
end)

api.log("✅ Test package loaded successfully!")
api.log("📦 Registered behaviors: spinner, pulse_and_die, seeker, collision_logger, spinning_seeker")
