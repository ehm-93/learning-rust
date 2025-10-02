use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{
    components::*,
    events::*,
    constants::*,
    resources::*,
    sounds::*,
    player::{Player, Dash},
};

/// Detects collisions between projectiles and other entities, emitting impact events
pub fn detect_projectile_collisions(
    mut collision_events: EventReader<CollisionEvent>,
    mut impact_events: EventWriter<ProjectileImpactEvent>,
    projectiles: Query<&Projectile>,
) {
    for collision_event in collision_events.read() {
        if let CollisionEvent::Started(entity1, entity2, _) = collision_event {
            // Check if either entity is a projectile
            let projectile_and_other = if projectiles.contains(*entity1) {
                Some((*entity1, *entity2))
            } else if projectiles.contains(*entity2) {
                Some((*entity2, *entity1))
            } else {
                None
            };

            if let Some((projectile, target)) = projectile_and_other {
                impact_events.write(ProjectileImpactEvent {
                    projectile,
                    target,
                });
            }
        }
    }
}

/// Handles projectile impact events by applying damage and effects
pub fn handle_projectile_impacts(
    mut impact_events: EventReader<ProjectileImpactEvent>,
    mut damage_events: EventWriter<DamageEvent>,
    mut commands: Commands,
    projectile_query: Query<(&Transform, &Projectile)>,
    enemy_query: Query<&Transform, (With<Enemy>, Without<Projectile>)>,
    player_query: Query<&Transform, (With<Player>, Without<Projectile>)>,
    mut enemy_velocities: Query<&mut Velocity, With<Enemy>>,
) {
    for impact in impact_events.read() {
        if let Ok((projectile_transform, projectile_data)) = projectile_query.get(impact.projectile) {
            // Handle enemy being hit by player projectile
            if let Ok(target_transform) = enemy_query.get(impact.target) {
                if projectile_data.team == Team::Player {
                    // Calculate knockback direction
                    let direction = (target_transform.translation.truncate() -
                                   projectile_transform.translation.truncate()).normalize_or_zero();

                    // Apply knockback to enemy
                    if let Ok(mut enemy_velocity) = enemy_velocities.get_mut(impact.target) {
                        enemy_velocity.linvel += direction * KNOCKBACK_FORCE;
                    }

                    // Deal damage to enemy
                    damage_events.write(DamageEvent {
                        target: impact.target,
                        damage: PROJECTILE_DAMAGE,
                    });
                }
            }

            // Handle player being hit by enemy projectile
            if let Ok(_target_transform) = player_query.get(impact.target) {
                if projectile_data.team == Team::Enemy {
                    // Deal damage to player
                    damage_events.write(DamageEvent {
                        target: impact.target,
                        damage: ENEMY_BULLET_DAMAGE,
                    });
                }
            }
        }

        // Clean up the projectile (use try_despawn to avoid errors if already despawned)
        if let Ok(mut entity) = commands.get_entity(impact.projectile) {
            entity.try_despawn();
        }
    }
}

/// Detects collisions between enemies and players for contact damage
pub fn detect_enemy_player_collisions(
    mut collision_events: EventReader<CollisionEvent>,
    mut damage_events: EventWriter<DamageEvent>,
    players: Query<(Entity, &Dash), With<Player>>,
    enemies: Query<Entity, With<Enemy>>,
) {
    for collision_event in collision_events.read() {
        if let CollisionEvent::Started(entity1, entity2, _) = collision_event {
            // Check if collision is between enemy and player
            let collision_pair = if enemies.contains(*entity1) {
                // Find player by entity
                players.iter().find(|(entity, _)| entity == entity2).map(|(player_entity, dash)| (*entity1, player_entity, dash))
            } else if enemies.contains(*entity2) {
                // Find player by entity
                players.iter().find(|(entity, _)| entity == entity1).map(|(player_entity, dash)| (*entity2, player_entity, dash))
            } else {
                None
            };

            if let Some((_enemy, player, dash)) = collision_pair {
                // Only deal damage if player is not invincible (not dashing with iframes)
                if !dash.is_invincible {
                    damage_events.write(DamageEvent {
                        target: player,
                        damage: ENEMY_CONTACT_DAMAGE,
                    });
                }
            }
        }
    }
}

/// Processes damage events by applying damage to entities
pub fn process_damage(
    mut damage_events: EventReader<DamageEvent>,
    mut hit_flash_events: EventWriter<HitFlashEvent>,
    mut health_query: Query<&mut Health>,
    enemy_query: Query<&Enemy>,
) {
    for damage_event in damage_events.read() {
        if let Ok(mut health) = health_query.get_mut(damage_event.target) {
            health.take_damage(damage_event.damage);

            // Trigger hit flash for enemies
            if enemy_query.contains(damage_event.target) {
                hit_flash_events.write(HitFlashEvent {
                    target: damage_event.target,
                });
            }
        }
    }
}

/// Cleans up entities that have died (health <= 0)
pub fn cleanup_dead_entities(
    mut commands: Commands,
    health_query: Query<(Entity, &Health, Option<&Enemy>, Option<&Player>)>,
    mut score: ResMut<Score>,
    mut game_state: ResMut<GameState>,
) {
    for (entity, health, enemy, player) in health_query.iter() {
        if health.is_dead() {
            // Award points for killing enemies
            if let Some(enemy_component) = enemy {
                let points = match enemy_component.archetype {
                    EnemyArchetype::SmallMelee => 10,
                    EnemyArchetype::BigMelee => 50,
                    EnemyArchetype::Shotgunner => 30,
                    EnemyArchetype::Sniper => 75,
                    EnemyArchetype::MachineGunner => 40,
                };
                score.current += points;
            }

            // Check if player died - set game over but don't despawn player
            if player.is_some() {
                *game_state = GameState::GameOver;
                // Don't despawn the player - we need them for restart
                continue;
            }

            // Only despawn enemies and other entities, not the player
            commands.entity(entity).despawn();
        }
    }
}

/// Cleans up projectiles that have exceeded their lifetime
pub fn cleanup_projectiles(
    mut commands: Commands,
    time: Res<Time>,
    mut projectiles: Query<(Entity, &mut Projectile)>,
) {
    for (entity, mut projectile) in projectiles.iter_mut() {
        projectile.lifetime.tick(time.delta());

        if projectile.lifetime.finished() {
            commands.entity(entity).try_despawn();
        }
    }
}

/// Handles hit flash events by adding flash component to enemies
pub fn handle_hit_flash(
    mut hit_flash_events: EventReader<HitFlashEvent>,
    mut commands: Commands,
    enemy_query: Query<(Entity, &MeshMaterial2d<ColorMaterial>, &Health), With<Enemy>>,
    materials: Res<Assets<ColorMaterial>>,
) {
    for flash_event in hit_flash_events.read() {
        // Check if the entity still exists, is an enemy, and is still alive
        if let Ok((entity, material_handle, health)) = enemy_query.get(flash_event.target) {
            // Only add hit flash if the enemy is still alive (prevents adding flash to dead entities)
            if !health.is_dead() {
                // Get the original color from the material
                if let Some(material) = materials.get(&material_handle.0) {
                    let original_color = material.color;

                    // Add hit flash component - entity is guaranteed to exist since we just queried it
                    commands.entity(entity).insert(HitFlash {
                        timer: Timer::from_seconds(HIT_FLASH_DURATION, TimerMode::Once),
                        original_color,
                    });
                }
            }
        }
    }
}

/// Updates hit flash effects over time
pub fn update_hit_flash(
    mut commands: Commands,
    time: Res<Time>,
    mut flash_query: Query<(Entity, &mut HitFlash, &MeshMaterial2d<ColorMaterial>)>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (entity, mut hit_flash, material_handle) in flash_query.iter_mut() {
        hit_flash.timer.tick(time.delta());

        if let Some(material) = materials.get_mut(&material_handle.0) {
            if hit_flash.timer.finished() {
                // Flash finished, restore original color and remove component
                material.color = hit_flash.original_color;
                commands.entity(entity).remove::<HitFlash>();
            } else {
                // Flash in progress, interpolate between flash color and original
                let progress = hit_flash.timer.elapsed_secs() / hit_flash.timer.duration().as_secs_f32();
                let flash_color = Color::srgb(1.0, 0.3, 0.3); // Bright red flash

                // Interpolate from flash color back to original
                material.color = Color::srgb(
                    flash_color.to_srgba().red * (1.0 - progress) + hit_flash.original_color.to_srgba().red * progress,
                    flash_color.to_srgba().green * (1.0 - progress) + hit_flash.original_color.to_srgba().green * progress,
                    flash_color.to_srgba().blue * (1.0 - progress) + hit_flash.original_color.to_srgba().blue * progress,
                );
            }
        }
    }
}

/// Handles grenade fuse timers and triggers explosions
pub fn handle_grenade_explosions(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    time: Res<Time>,
    mut grenades: Query<(Entity, &Transform, &mut Grenade)>,
    mut explosion_events: EventWriter<GrenadeExplosionEvent>,
    game_sounds: Res<GameSounds>,
) {
    for (entity, transform, mut grenade) in grenades.iter_mut() {
        grenade.fuse_timer.tick(time.delta());

        if grenade.fuse_timer.finished() {
            let explosion_pos = transform.translation.truncate();

            // Trigger explosion event
            explosion_events.write(GrenadeExplosionEvent {
                position: explosion_pos,
                damage: GRENADE_DAMAGE,
                radius: GRENADE_EXPLOSION_RADIUS,
                team: grenade.team,
            });

            // Spawn visual explosion effect
            commands.spawn((
                Mesh2d(meshes.add(Circle::new(EXPLOSION_START_SIZE))),
                MeshMaterial2d(materials.add(Color::srgba(1.0, 1.0, 0.0, 0.8))), // Yellow with some transparency
                Transform::from_translation(explosion_pos.extend(0.2)), // Slightly above other objects
                ExplosionEffect {
                    timer: Timer::from_seconds(EXPLOSION_DURATION, TimerMode::Once),
                    start_radius: EXPLOSION_START_SIZE,
                    end_radius: EXPLOSION_END_SIZE,
                },
            ));

            // Play explosion sound effect
            play_sound(&mut commands, game_sounds.explosion_01.clone(), 0.8);

            // Remove the grenade
            commands.entity(entity).despawn();
        }
    }
}

/// Handles grenade explosion events by dealing area damage
pub fn process_grenade_explosions(
    mut explosion_events: EventReader<GrenadeExplosionEvent>,
    mut damage_events: EventWriter<DamageEvent>,
    mut hit_flash_events: EventWriter<HitFlashEvent>,
    entities: Query<(Entity, &Transform, Option<&Enemy>, Option<&Player>)>,
) {
    for explosion in explosion_events.read() {
        // Find all entities within the explosion radius
        for (entity, transform, enemy, player) in entities.iter() {
            let distance = explosion.position.distance(transform.translation.truncate());

            if distance <= explosion.radius {
                // Check team affiliation to avoid friendly fire
                let should_damage = match explosion.team {
                    Team::Player => {
                        // Player grenades can damage enemies but not the player
                        enemy.is_some()
                    },
                    Team::Enemy => {
                        // Enemy grenades can damage players but not enemies
                        player.is_some()
                    },
                };

                if should_damage {
                    // Calculate damage falloff based on distance
                    let damage_multiplier = 1.0 - (distance / explosion.radius);
                    let final_damage = explosion.damage * damage_multiplier;

                    // Deal damage
                    damage_events.write(DamageEvent {
                        target: entity,
                        damage: final_damage,
                    });

                    // Trigger hit flash for enemies
                    if enemy.is_some() {
                        hit_flash_events.write(HitFlashEvent {
                            target: entity,
                        });
                    }
                }
            }
        }
    }
}

/// System to stop grenades when they reach minimum speed
pub fn manage_grenade_speed(
    mut grenade_query: Query<&mut Velocity, With<Grenade>>,
) {
    for mut velocity in grenade_query.iter_mut() {
        let speed = velocity.linvel.length();

        // If grenade is moving slower than minimum speed, stop it completely
        if speed > 0.0 && speed < GRENADE_MIN_SPEED {
            velocity.linvel = Vec2::ZERO;
            velocity.angvel = 0.0;
        }
    }
}

/// System to update explosion visual effects
pub fn update_explosion_effects(
    mut commands: Commands,
    time: Res<Time>,
    mut explosion_query: Query<(Entity, &mut ExplosionEffect, &mut Mesh2d, &MeshMaterial2d<ColorMaterial>)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (entity, mut explosion, mut mesh2d, material_handle) in explosion_query.iter_mut() {
        explosion.timer.tick(time.delta());

        if explosion.timer.finished() {
            // Remove the explosion effect when finished
            commands.entity(entity).despawn();
        } else {
            // Calculate progress (0.0 to 1.0)
            let progress = explosion.timer.elapsed_secs() / explosion.timer.duration().as_secs_f32();

            // Interpolate radius from start to end
            let current_radius = explosion.start_radius + (explosion.end_radius - explosion.start_radius) * progress;

            // Update the mesh to the new size
            mesh2d.0 = meshes.add(Circle::new(current_radius));

            // Fade out the explosion (reduce alpha over time)
            if let Some(material) = materials.get_mut(&material_handle.0) {
                let alpha = 0.8 * (1.0 - progress); // Start at 0.8, fade to 0
                material.color = Color::srgba(1.0, 1.0, 0.0, alpha);
            }
        }
    }
}
