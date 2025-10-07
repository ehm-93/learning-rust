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
