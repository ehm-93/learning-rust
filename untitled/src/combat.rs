use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{
    components::*,
    events::*,
    constants::*,
    resources::*,
};

/// Detects collisions between projectiles and other entities, emitting impact events
pub fn detect_projectile_collisions(
    mut collision_events: EventReader<CollisionEvent>,
    mut impact_events: EventWriter<ProjectileImpactEvent>,
    projectiles: Query<&Projectile>,
    players: Query<&Player>,
    enemies: Query<&Enemy>,
    obstacles: Query<&Obstacle>,
    boundaries: Query<&Boundary>,
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
                if let Ok(projectile_data) = projectiles.get(projectile) {
                    // Handle team-based collision logic
                    let should_collide = if obstacles.contains(target) || boundaries.contains(target) {
                        true // Always collide with obstacles and boundaries
                    } else if players.contains(target) && projectile_data.team == Team::Enemy {
                        true // Enemy bullets can hit players
                    } else if enemies.contains(target) && projectile_data.team == Team::Player {
                        true // Player bullets can hit enemies
                    } else {
                        false // No friendly fire
                    };

                    if should_collide {
                        impact_events.write(ProjectileImpactEvent {
                            projectile,
                            target,
                        });
                    }
                }
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

        // Clean up the projectile
        if let Ok(mut entity) = commands.get_entity(impact.projectile) {
            entity.despawn();
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
            commands.entity(entity).despawn();
        }
    }
}

/// Handles hit flash events by adding flash component to enemies
pub fn handle_hit_flash(
    mut hit_flash_events: EventReader<HitFlashEvent>,
    mut commands: Commands,
    enemy_query: Query<&MeshMaterial2d<ColorMaterial>, With<Enemy>>,
    materials: Res<Assets<ColorMaterial>>,
) {
    for flash_event in hit_flash_events.read() {
        if let Ok(material_handle) = enemy_query.get(flash_event.target) {
            // Get the original color from the material
            if let Some(material) = materials.get(&material_handle.0) {
                let original_color = material.color;
                
                // Add hit flash component
                commands.entity(flash_event.target).insert(HitFlash {
                    timer: Timer::from_seconds(HIT_FLASH_DURATION, TimerMode::Once),
                    original_color,
                });
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
