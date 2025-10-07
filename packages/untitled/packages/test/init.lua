-- Test package for Phase 1 verification
-- This should register itself with the game

api.log("Hello from test package!")

api.register("test", {
    behavior = "test_behavior",
    message = "Phase 1 working!"
})
