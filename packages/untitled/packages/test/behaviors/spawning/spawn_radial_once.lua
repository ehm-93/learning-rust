-- Foundational behavior: Spawn entities radially once
api.behaviors.register("spawn_radial_once", function(params)
    local entity_id = params.entity_id
    local count = params.count or 8
    local radius = params.radius or 50.0
    local projectile_speed = params.projectile_speed or 80.0
    local projectile_spin = params.projectile_spin or 5.0

    local has_spawned = false

    return {
        on_spawn = function(world)
            world:log("💫 Radial spawner (one-time, count: " .. count .. ", radius: " .. radius .. ")")

            -- Get parent position and rotation
            local pos = world:get_position()
            if not pos then return end

            local rot = world:get_rotation()
            local parent_yaw = rot and rot.yaw or 0

            -- Spawn N projectiles in a circle
            for i = 1, count do
                local angle = parent_yaw + (i / count) * 2 * math.pi
                local spawn_x = pos.x + math.cos(angle) * radius
                local spawn_y = pos.y + math.sin(angle) * radius

                -- Spawn entity facing outward from parent
                world:spawn_entity(spawn_x, spawn_y, 0, angle)
            end

            has_spawned = true
            world:log("💥 Spawned " .. count .. " projectiles in radial pattern")
        end,

        update = function(world, dt)
            -- One-time spawn, nothing to update
        end
    }
end)
