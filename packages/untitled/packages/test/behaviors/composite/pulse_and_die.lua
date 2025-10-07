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
