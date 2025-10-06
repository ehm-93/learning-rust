use bevy::prelude::*;
use mlua::Lua;
use std::collections::HashMap;

// === Phase 1: Core Behaviors ===

/// Lifetime behavior - despawns after duration
pub const LIFETIME_BEHAVIOR: &str = r#"
    function update(entity, dt, config)
        local max_lifetime = config.lifetime or 2.0

        entity.elapsed_time = (entity.elapsed_time or 0.0) + dt

        if entity.elapsed_time >= max_lifetime then
            entity.should_despawn = true
        end

        return entity
    end
"#;

/// Area damage behavior - damages in radius
pub const AREA_BEHAVIOR: &str = r#"
    function update(entity, dt, config)
        local radius = config.radius or 50.0
        local damage = config.damage or 25.0

        -- Simple area damage logic - mark for area damage system
        if entity.should_explode then
            entity.area_damage = {
                radius = radius,
                damage = damage,
                x = entity.x,
                y = entity.y
            }
            entity.should_despawn = true
        end

        return entity
    end
"#;

/// Homing behavior - seeks target
pub const HOMING_BEHAVIOR: &str = r#"
    function update(entity, dt, config)
        local turn_rate = config.turn_rate or 2.0
        local target_x = config.target_x or 0.0
        local target_y = config.target_y or 0.0

        -- Calculate angle to target
        local dx = target_x - entity.x
        local dy = target_y - entity.y
        local target_angle = math.atan2(dy, dx)

        -- Current rotation
        local current_angle = entity.rotation or 0.0

        -- Calculate shortest rotation direction
        local angle_diff = target_angle - current_angle
        while angle_diff > math.pi do
            angle_diff = angle_diff - 2 * math.pi
        end
        while angle_diff < -math.pi do
            angle_diff = angle_diff + 2 * math.pi
        end

        -- Turn towards target
        local max_turn = turn_rate * dt
        if math.abs(angle_diff) <= max_turn then
            entity.rotation = target_angle
        else
            entity.rotation = current_angle + (angle_diff > 0 and max_turn or -max_turn)
        end

        return entity
    end
"#;

// === Phase 1: Effect System ===

/// Behavior execution type - either native Rust or Lua script
#[derive(Clone, Debug)]
pub enum BehaviorType {
    /// Native Rust behavior for maximum performance
    Rust(String),
    /// Lua script behavior for flexibility
    Lua(String),
}

/// Represents an effect that combines multiple behaviors with configuration
#[derive(Clone, Debug)]
pub struct Effect {
    pub behaviors: Vec<BehaviorType>,
    pub config: HashMap<String, f32>,
}

/// Rust implementation of projectile behavior for high performance
fn execute_rust_projectile_behavior(
    entity_data: &mut HashMap<String, f32>,
    dt: f32,
    config: &HashMap<String, f32>,
) {
    let speed = config.get("speed").copied().unwrap_or(100.0);
    let angle = entity_data.get("rotation").copied().unwrap_or(0.0);
    let x = entity_data.get("x").copied().unwrap_or(0.0);
    let y = entity_data.get("y").copied().unwrap_or(0.0);

    // Move forward based on velocity and direction
    let dx = angle.cos() * speed * dt;
    let dy = angle.sin() * speed * dt;

    let new_x = x + dx;
    let new_y = y + dy;

    entity_data.insert("x".to_string(), new_x);
    entity_data.insert("y".to_string(), new_y);

    // Simple collision check against world bounds
    let max_bound = 1000.0;
    if new_x < -max_bound || new_x > max_bound || new_y < -max_bound || new_y > max_bound {
        entity_data.insert("should_despawn".to_string(), 1.0);
    }
}

/// Execute a native Rust behavior
fn execute_rust_behavior(
    behavior_name: &str,
    entity_data: &mut HashMap<String, f32>,
    dt: f32,
    config: &HashMap<String, f32>,
) -> Result<(), String> {
    match behavior_name {
        "projectile" => {
            execute_rust_projectile_behavior(entity_data, dt, config);
            Ok(())
        }
        _ => Err(format!("Unknown Rust behavior: {}", behavior_name)),
    }
}

impl Effect {
    pub fn new(behaviors: Vec<BehaviorType>, config: HashMap<String, f32>) -> Self {
        Self { behaviors, config }
    }

    /// Helper to create effect with mixed Rust and Lua behaviors
    pub fn with_rust_and_lua(rust_behaviors: Vec<&str>, lua_behaviors: Vec<&str>, config: HashMap<String, f32>) -> Self {
        let mut behaviors = Vec::new();

        for behavior in rust_behaviors {
            behaviors.push(BehaviorType::Rust(behavior.to_string()));
        }

        for behavior in lua_behaviors {
            behaviors.push(BehaviorType::Lua(behavior.to_string()));
        }

        Self { behaviors, config }
    }
}

/// Component for entities using the Phase 1 effect system
#[derive(Component)]
pub struct EffectControlled {
    pub effect: Effect,
    pub entity_data: HashMap<String, f32>,
    pub start_time: f32,
}

impl EffectControlled {
    pub fn new(effect: Effect, time: f32) -> Self {
        let mut entity_data = HashMap::new();

        // Initialize common entity data fields
        entity_data.insert("x".to_string(), 0.0);
        entity_data.insert("y".to_string(), 0.0);
        entity_data.insert("rotation".to_string(), 0.0);
        entity_data.insert("elapsed_time".to_string(), 0.0);
        entity_data.insert("should_despawn".to_string(), 0.0);
        entity_data.insert("should_explode".to_string(), 0.0);

        Self {
            effect,
            entity_data,
            start_time: time,
        }
    }
}

/// System that executes effect behaviors for entities with EffectControlled component
pub fn effect_update_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut EffectControlled)>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();

    for (entity, mut transform, mut effect_controlled) in query.iter_mut() {
        // Update entity data with current transform
        effect_controlled.entity_data.insert("x".to_string(), transform.translation.x);
        effect_controlled.entity_data.insert("y".to_string(), transform.translation.y);
        effect_controlled.entity_data.insert("rotation".to_string(), transform.rotation.to_euler(EulerRot::ZYX).0);

        let elapsed_time = time.elapsed_secs() - effect_controlled.start_time;
        effect_controlled.entity_data.insert("elapsed_time".to_string(), elapsed_time);

        // Execute each behavior in the effect
        let behaviors = effect_controlled.effect.behaviors.clone();
        let config = effect_controlled.effect.config.clone();

        for behavior in &behaviors {
            match behavior {
                BehaviorType::Rust(behavior_name) => {
                    // Execute native Rust behavior for maximum performance
                    if let Err(e) = execute_rust_behavior(
                        behavior_name,
                        &mut effect_controlled.entity_data,
                        dt,
                        &config,
                    ) {
                        error!("Failed to execute Rust behavior '{}': {}", behavior_name, e);
                    }
                }
                BehaviorType::Lua(behavior_name) => {
                    // Execute Lua behavior for flexibility
                    let behavior_script = match behavior_name.as_str() {
                        "lifetime" => LIFETIME_BEHAVIOR,
                        "area" => AREA_BEHAVIOR,
                        "homing" => HOMING_BEHAVIOR,
                        _ => {
                            warn!("Unknown Lua behavior: {}", behavior_name);
                            continue;
                        }
                    };

                    if let Err(e) = execute_lua_behavior(behavior_script, &mut effect_controlled.entity_data, dt, &config) {
                        error!("Failed to execute Lua behavior '{}': {}", behavior_name, e);
                    }
                }
            }
        }

        // Update transform from entity data
        transform.translation.x = effect_controlled.entity_data.get("x").copied().unwrap_or(0.0);
        transform.translation.y = effect_controlled.entity_data.get("y").copied().unwrap_or(0.0);

        let rotation = effect_controlled.entity_data.get("rotation").copied().unwrap_or(0.0);
        transform.rotation = Quat::from_rotation_z(rotation);

        // Handle despawning
        if effect_controlled.entity_data.get("should_despawn").copied().unwrap_or(0.0) > 0.0 {
            commands.entity(entity).despawn();
        }

        // Handle area damage (simplified for Phase 1)
        if effect_controlled.entity_data.contains_key("area_damage") {
            info!("Area damage triggered at ({}, {})",
                  effect_controlled.entity_data.get("x").unwrap_or(&0.0),
                  effect_controlled.entity_data.get("y").unwrap_or(&0.0));
            // In a full implementation, this would trigger an area damage system
        }
    }
}

/// Execute a single Lua behavior script on entity data
fn execute_lua_behavior(
    script: &str,
    entity_data: &mut HashMap<String, f32>,
    dt: f32,
    config: &HashMap<String, f32>,
) -> Result<(), mlua::Error> {
    let lua = Lua::new();

    // Load the behavior script
    lua.load(script).exec()?;

    // Get the update function
    let update_func: mlua::Function = lua.globals().get("update")?;

    // Convert entity data to Lua table
    let entity_table = lua.create_table()?;
    for (key, value) in entity_data.iter() {
        entity_table.set(key.clone(), *value)?;
    }

    // Convert config to Lua table
    let config_table = lua.create_table()?;
    for (key, value) in config.iter() {
        config_table.set(key.clone(), *value)?;
    }

    // Call the update function
    let result_table: mlua::Table = update_func.call((entity_table, dt, config_table))?;

    // Update entity data from result, handling mixed types
    for pair in result_table.pairs::<String, mlua::Value>() {
        let (key, value) = pair?;
        match value {
            mlua::Value::Number(n) => {
                entity_data.insert(key, n as f32);
            }
            mlua::Value::Boolean(b) => {
                // Convert boolean to f32 (false = 0.0, true = 1.0)
                entity_data.insert(key, if b { 1.0 } else { 0.0 });
            }
            mlua::Value::Integer(i) => {
                entity_data.insert(key, i as f32);
            }
            _ => {
                // Skip other types like strings, tables, etc.
                warn!("Ignoring unsupported Lua value type for key '{}'", key);
            }
        }
    }

    Ok(())
}

// === Phase 1: Test Effects ===

/// Create a basic fireball effect (Rust projectile + Lua lifetime)
pub fn create_fireball_effect() -> Effect {
    let mut config = HashMap::new();
    config.insert("speed".to_string(), 300.0);
    config.insert("lifetime".to_string(), 2.0);

    Effect::with_rust_and_lua(
        vec!["projectile"],  // Use Rust projectile for performance
        vec!["lifetime"],    // Use Lua lifetime for flexibility
        config,
    )
}

/// Create a homing rocket effect (Rust projectile + Lua lifetime + Lua homing)
pub fn create_homing_rocket_effect(target_x: f32, target_y: f32) -> Effect {
    let mut config = HashMap::new();
    config.insert("speed".to_string(), 200.0);
    config.insert("lifetime".to_string(), 5.0);
    config.insert("turn_rate".to_string(), 2.0);
    config.insert("target_x".to_string(), target_x);
    config.insert("target_y".to_string(), target_y);

    Effect::with_rust_and_lua(
        vec!["projectile"],           // Use Rust projectile for performance
        vec!["lifetime", "homing"],   // Use Lua for complex behaviors
        config,
    )
}

/// Setup function for homing rocket barrage stress test
pub fn setup_homing_barrage_test(
    mut commands: Commands,
    time: Res<Time>,
) {
    let current_time = time.elapsed_secs();

    // Spawn player (simple representation)
    commands.spawn((
        Transform::from_xyz(-400.0, 0.0, 0.0),
        GlobalTransform::default(),
        Visibility::default(),
        Sprite {
            color: Color::srgb(0.2, 0.8, 0.2),
            custom_size: Some(Vec2::splat(30.0)),
            ..default()
        },
    ));

    // Spawn multiple enemies in formation
    let enemy_positions = vec![
        (300.0, -200.0),
        (350.0, -100.0),
        (400.0, 0.0),
        (350.0, 100.0),
        (300.0, 200.0),
        (450.0, -150.0),
        (500.0, 0.0),
        (450.0, 150.0),
    ];

    for (x, y) in &enemy_positions {
        commands.spawn((
            Transform::from_xyz(*x, *y, 0.0),
            GlobalTransform::default(),
            Visibility::default(),
            Sprite {
                color: Color::srgb(0.8, 0.2, 0.2),
                custom_size: Some(Vec2::splat(25.0)),
                ..default()
            },
        ));
    }

    // Spawn EXTREME barrage of homing rockets - stress test with 100+ rockets!
    let mut rocket_spawn_positions = Vec::new();

    // Create a massive swarm from multiple spawn rings
    for ring in 0..6 {
        let base_distance = 250.0 + (ring as f32 * 75.0);
        let rockets_in_ring = 16 + (ring * 4); // More rockets in outer rings

        for i in 0..rockets_in_ring {
            let angle = (i as f32 / rockets_in_ring as f32) * 2.0 * std::f32::consts::PI;
            let x = base_distance * angle.cos();
            let y = base_distance * angle.sin();
            rocket_spawn_positions.push((x, y));
        }
    }

    // Add some extra clusters for chaos
    let cluster_positions = [
        (600.0, 0.0), (650.0, 100.0), (650.0, -100.0),
        (700.0, 200.0), (700.0, -200.0), (750.0, 0.0),
        (800.0, 150.0), (800.0, -150.0), (850.0, 75.0), (850.0, -75.0)
    ];

    for &(cx, cy) in &cluster_positions {
        // Create tight clusters of 8 rockets each
        for i in 0..8 {
            let angle = (i as f32 / 8.0) * 2.0 * std::f32::consts::PI;
            let offset = 30.0;
            let x = cx + offset * angle.cos();
            let y = cy + offset * angle.sin();
            rocket_spawn_positions.push((x, y));
        }
    }

    info!("Spawning {} homing rockets for barrage stress test!", rocket_spawn_positions.len());

    for (i, (x, y)) in rocket_spawn_positions.iter().enumerate() {
        // Create homing rocket with slight delay per rocket for wave effect
        let delay_factor = (i as f32) * 0.05; // 50ms delay between rockets
        let homing_rocket = create_homing_rocket_effect(-400.0, 0.0); // All target player position
        let mut rocket_controlled = EffectControlled::new(homing_rocket, current_time + delay_factor);

        rocket_controlled.entity_data.insert("x".to_string(), *x);
        rocket_controlled.entity_data.insert("y".to_string(), *y);
        rocket_controlled.entity_data.insert("rotation".to_string(), std::f32::consts::PI); // Start facing left

        // Vary rocket colors for visual variety
        let color = match i % 4 {
            0 => Color::srgb(0.8, 0.3, 0.8), // Purple
            1 => Color::srgb(0.9, 0.4, 0.2), // Orange-red
            2 => Color::srgb(0.7, 0.2, 0.9), // Magenta
            _ => Color::srgb(0.6, 0.3, 0.7), // Purple-pink
        };

        commands.spawn((
            Transform::from_xyz(*x, *y, 0.0),
            GlobalTransform::default(),
            Visibility::default(),
            Sprite {
                color,
                custom_size: Some(Vec2::splat(8.0)), // Smaller rockets for performance
                ..default()
            },
            rocket_controlled,
        ));
    }

    info!("Homing rocket barrage stress test setup complete - {} rockets targeting player!", rocket_spawn_positions.len());
}

/// Setup function for Phase 1 combat test
pub fn setup_phase1_combat_test(
    mut commands: Commands,
    time: Res<Time>,
) {
    let current_time = time.elapsed_secs();

    // Spawn player (simple representation)
    commands.spawn((
        Transform::from_xyz(-400.0, 0.0, 0.0),
        GlobalTransform::default(),
        Visibility::default(),
        Sprite {
            color: Color::srgb(0.2, 0.8, 0.2),
            custom_size: Some(Vec2::splat(30.0)),
            ..default()
        },
    ));

    // Spawn enemy (simple representation)
    commands.spawn((
        Transform::from_xyz(400.0, 0.0, 0.0),
        GlobalTransform::default(),
        Visibility::default(),
        Sprite {
            color: Color::srgb(0.8, 0.2, 0.2),
            custom_size: Some(Vec2::splat(30.0)),
            ..default()
        },
    ));

    // Spawn player projectile (fireball moving right)
    let fireball = create_fireball_effect();
    let mut fireball_controlled = EffectControlled::new(fireball, current_time);
    fireball_controlled.entity_data.insert("x".to_string(), -350.0);
    fireball_controlled.entity_data.insert("y".to_string(), 0.0);
    fireball_controlled.entity_data.insert("rotation".to_string(), 0.0); // Right

    commands.spawn((
        Transform::from_xyz(-350.0, 0.0, 0.0),
        GlobalTransform::default(),
        Visibility::default(),
        Sprite {
            color: Color::srgb(1.0, 0.5, 0.0),
            custom_size: Some(Vec2::splat(15.0)),
            ..default()
        },
        fireball_controlled,
    ));

    // Spawn enemy projectile (homing rocket targeting player area)
    let homing_rocket = create_homing_rocket_effect(-400.0, 0.0);
    let mut rocket_controlled = EffectControlled::new(homing_rocket, current_time);
    rocket_controlled.entity_data.insert("x".to_string(), 350.0);
    rocket_controlled.entity_data.insert("y".to_string(), 0.0);
    rocket_controlled.entity_data.insert("rotation".to_string(), std::f32::consts::PI); // Left

    commands.spawn((
        Transform::from_xyz(350.0, 0.0, 0.0),
        GlobalTransform::default(),
        Visibility::default(),
        Sprite {
            color: Color::srgb(0.8, 0.3, 0.8),
            custom_size: Some(Vec2::splat(12.0)),
            ..default()
        },
        rocket_controlled,
    ));

    info!("Phase 1 combat test setup complete - player fireball vs enemy homing rocket");
}

// Phase 0 setup function removed - using Phase 1 effect system instead
