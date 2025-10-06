use bevy::prelude::*;
use mlua::Lua;

/// Component for entities controlled by Lua scripts
#[derive(Component)]
pub struct LuaControlled {
    pub script: String,
    pub start_time: f32,
}

impl LuaControlled {
    pub fn new(script: String, time: f32) -> Self {
        Self {
            script,
            start_time: time,
        }
    }
}

/// System that executes Lua update functions for entities with LuaControlled component
pub fn lua_update_system(
    mut query: Query<(&mut Transform, &LuaControlled)>,
    time: Res<Time>,
) {
    for (mut transform, lua_controlled) in query.iter_mut() {
        // Calculate elapsed time since the entity was created
        let elapsed_time = time.elapsed_secs() - lua_controlled.start_time;

        // Create a new Lua context for each entity (simple but potentially inefficient for Phase 0)
        let lua = Lua::new();

        // Load the script
        if let Err(e) = lua.load(&lua_controlled.script).exec() {
            error!("Failed to load Lua script: {}", e);
            continue;
        }

        // Get the update function
        let update_func: mlua::Function = match lua.globals().get("update") {
            Ok(func) => func,
            Err(e) => {
                error!("Failed to get 'update' function from Lua script: {}", e);
                continue;
            }
        };

        // Call the update function with current position and time
        let result: Result<(f32, f32), mlua::Error> = update_func.call((
            transform.translation.x,
            transform.translation.y,
            elapsed_time,
        ));

        match result {
            Ok((new_x, new_y)) => {
                transform.translation.x = new_x;
                transform.translation.y = new_y;
            }
            Err(e) => {
                error!("Failed to execute Lua update function: {}", e);
            }
        }
    }
}

/// The test Lua script from the Phase 0 specification - entities circle in place
const CIRCLE_BEHAVIOR: &str = r#"
    function update(x, y, time)
        local angle = time * 2
        local radius = 100
        return x + math.cos(angle) * radius,
               y + math.sin(angle) * radius
    end
"#;

/// Setup function to spawn test entities with Lua circling behavior
pub fn setup_lua_test_entities(
    mut commands: Commands,
    time: Res<Time>,
) {
    let current_time = time.elapsed_secs();

    // Spawn 100 test entities in a grid pattern, each with the circling Lua behavior
    for i in 0..100 {
        let x = (i % 10) as f32 * 100.0 - 450.0; // Grid layout: 10 columns
        let y = (i / 10) as f32 * 100.0 - 450.0;  // 10 rows

        commands.spawn((
            Transform::from_xyz(x, y, 0.0),
            GlobalTransform::default(),
            Visibility::default(),
            // Simple colored square sprite
            Sprite {
                color: Color::srgb(0.3, 0.8, 0.3),
                custom_size: Some(Vec2::splat(20.0)),
                ..default()
            },
            LuaControlled::new(CIRCLE_BEHAVIOR.to_string(), current_time),
        ));
    }

    info!("Spawned 100 Lua-controlled test entities");
}
