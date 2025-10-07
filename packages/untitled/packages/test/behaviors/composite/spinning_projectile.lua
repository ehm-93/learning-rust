-- Composite behavior: Spinning projectile
-- Demonstrates combining constant spin with impulse velocity
api.behaviors.register("spinning_projectile", function(params)
    local entity_id = params.entity_id
    local speed = params.speed or 80.0
    local angular_velocity = params.angular_velocity or 10.0
    local lifetime = params.lifetime or 5.0

    -- Create sub-behaviors
    local impulse = api.behaviors.create("impulse", {
        entity_id = entity_id,
        use_facing = true,
        speed = speed
    })

    local spin = api.behaviors.create("constant_spin", {
        entity_id = entity_id,
        angular_velocity = angular_velocity
    })

    local lifetime_behavior = nil
    if lifetime > 0 then
        -- Would use builtin lifetime behavior, but for now track manually
        local elapsed = 0
        lifetime_behavior = {
            update = function(world, dt)
                elapsed = elapsed + dt
                if elapsed >= lifetime then
                    world:despawn()
                end
            end
        }
    end

    return {
        on_spawn = function(world)
            world:log("🎯 Spinning Projectile spawned (impulse + spin)")
            if impulse and impulse.on_spawn then
                impulse.on_spawn(world)
            end
            if spin and spin.on_spawn then
                spin.on_spawn(world)
            end
        end,

        update = function(world, dt)
            -- Execute sub-behaviors
            if impulse and impulse.update then
                impulse.update(world, dt)
            end
            if spin and spin.update then
                spin.update(world, dt)
            end
            if lifetime_behavior and lifetime_behavior.update then
                lifetime_behavior.update(world, dt)
            end
        end,

        on_despawn = function(world)
            if impulse and impulse.on_despawn then
                impulse.on_despawn(world)
            end
            if spin and spin.on_despawn then
                spin.on_despawn(world)
            end
        end
    }
end)
