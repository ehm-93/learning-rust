-- Test package for Phase 1-2 verification
-- This should register a behavior with the game

api.log("Hello from test package!")

-- Register a simple test behavior that uses the WorldApi
-- api.behaviors.register takes a factory function that receives params and returns a behavior definition
api.behaviors.register("test_behavior", function(params)
    -- Entity ID is now passed via params instead of world:entity_id()
    local entity_id = params.entity_id

    api.log("test_behavior factory called for entity: " .. tostring(entity_id))

    -- Return the behavior definition table
    return {
        -- Called when behavior is added to an entity
        on_spawn = function(world)
            api.log("test_behavior: on_spawn called!")
            world:log("My entity ID is: " .. tostring(entity_id))

            -- Test position get/set
            local pos = world:get_position()
            if pos and pos.x then
                world:log("Starting position: (" .. pos.x .. ", " .. pos.y .. ", " .. pos.z .. ")")
            end
        end,

        -- Called every frame
        update = function(world, dt)
            -- Move the entity slightly each frame for testing
            local pos = world:get_position()
            if pos and pos.x then
                world:set_position(pos.x + dt * 0.1, pos.y, pos.z)

                -- Query nearby entities
                local nearby = world:query_nearby(5.0)
                if nearby and #nearby > 0 then
                    world:log("Found " .. #nearby .. " nearby entities")
                end
            end
        end,

        -- Called when behavior is removed
        on_despawn = function(world)
            world:log("test_behavior: on_despawn called for entity " .. tostring(entity_id))
        end
    }
end)

api.log("test_behavior registered successfully!")
