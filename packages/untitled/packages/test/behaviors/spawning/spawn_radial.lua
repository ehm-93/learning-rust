-- Foundational behavior: Spawn entities radially at intervals
api.behaviors.register("spawn_radial", function(params)
    local entity_id = params.entity_id
    local count = params.count or 8  -- Number of entities to spawn
    local radius = params.radius or 50.0  -- Distance from center
    local interval = params.interval or 1.0  -- Seconds between spawns
    local projectile_speed = params.projectile_speed or 80.0
    local projectile_spin = params.projectile_spin or 5.0
    local projectile_lifetime = params.projectile_lifetime or 3.0

    local elapsed = 0
    local spawned_count = 0

    return {
        on_spawn = function(world)
            world:log("🎆 Radial spawner (count: " .. count .. ", radius: " .. radius .. ", interval: " .. interval .. "s)")
        end,

        update = function(world, dt)
            elapsed = elapsed + dt

            -- Check if it's time to spawn the next wave
            if elapsed >= interval and spawned_count < count then
                elapsed = elapsed - interval  -- Reset timer

                -- Get parent position
                local pos = world:get_position()
                if not pos then return end

                -- Spawn N projectiles in a circle
                for i = 1, count do
                    local angle = (i / count) * 2 * math.pi
                    local spawn_x = pos.x + math.cos(angle) * radius
                    local spawn_y = pos.y + math.sin(angle) * radius

                    -- Spawn entity facing outward
                    local entity_id = world:spawn_entity(spawn_x, spawn_y, 0, angle)

                    -- TODO: Attach behaviors to spawned entity
                    -- For now, they'll just be static entities
                    -- In a full implementation, you'd want to register the entity
                    -- with behaviors like "spinning_projectile"
                end

                spawned_count = spawned_count + 1
                world:log("💥 Spawned wave " .. spawned_count .. " of " .. count .. " projectiles")
            end
        end
    }
end)
