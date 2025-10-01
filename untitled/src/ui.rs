use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use crate::{
    components::*,
    resources::*,
    world::{reset_to_cathedral, cleanup_game_entities, cleanup_dungeon_entities},
    player::{Player, FireTimer},
};

// Tooltip system module
pub mod tooltip;

/// Sets up the health bar UI elements
pub fn setup_health_bar(
    mut commands: Commands,
) {
    // Create the health bar container
    commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            left: Val::Px(20.0),
            bottom: Val::Px(20.0),
            width: Val::Px(200.0),
            height: Val::Px(20.0),
            border: UiRect::all(Val::Px(2.0)),
            ..default()
        })
        .with_children(|parent| {
            // Health bar background
            parent.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
            ));

            // Health bar fill
            parent.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.8, 0.2, 0.2)),
                HealthBar,
            ));
        })
        .insert((
            BorderColor(Color::srgb(0.6, 0.6, 0.6)),
            BackgroundColor(Color::NONE),
        ));
}

/// Updates the health bar based on player health
pub fn update_health_bar(
    player_query: Query<&Health, With<Player>>,
    mut health_bar_query: Query<&mut Node, With<HealthBar>>,
) {
    if let Ok(player_health) = player_query.single() {
        if let Ok(mut health_bar_node) = health_bar_query.single_mut() {
            let health_percentage = (player_health.current / player_health.max) * 100.0;
            health_bar_node.width = Val::Percent(health_percentage);

            // Change color based on health level
            // This will be handled by a separate system for the BackgroundColor component
        }
    }
}

/// Updates the health bar color based on health percentage
pub fn update_health_bar_color(
    player_query: Query<&Health, With<Player>>,
    mut health_bar_query: Query<&mut BackgroundColor, With<HealthBar>>,
) {
    if let Ok(player_health) = player_query.single() {
        if let Ok(mut health_bar_color) = health_bar_query.single_mut() {
            let health_percentage = player_health.current / player_health.max;

            // Color transitions: Red -> Yellow -> Green
            let color = if health_percentage > 0.6 {
                // Green to yellow transition
                let t = (health_percentage - 0.6) / 0.4;
                Color::srgb(1.0 - t * 0.2, 0.8, 0.2)
            } else if health_percentage > 0.3 {
                // Yellow to red transition
                let t = (health_percentage - 0.3) / 0.3;
                Color::srgb(1.0, 0.8 * t + 0.2, 0.2)
            } else {
                // Red
                Color::srgb(0.8, 0.2, 0.2)
            };

            health_bar_color.0 = color;
        }
    }
}

/// Sets up the score display UI
pub fn setup_score_display(
    mut commands: Commands,
) {
    // Create score display in top-right corner
    commands.spawn((
        Text::new("Score: 0"),
        TextFont {
            font_size: 24.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(20.0),
            right: Val::Px(20.0),
            ..default()
        },
        ScoreText,
    ));
}

/// Updates the score display text
pub fn update_score_display(
    score: Res<Score>,
    mut query: Query<&mut Text, With<ScoreText>>,
) {
    if let Ok(mut text) = query.single_mut() {
        **text = format!("Score: {}", score.current);
    }
}

/// Sets up the game over overlay
pub fn setup_game_over_overlay(
    mut commands: Commands,
    score: Res<Score>,
) {
    // Semi-transparent dark overlay
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(0.0),
            left: Val::Px(0.0),
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
        GameOverOverlay,
    ))
    .with_children(|parent| {
        // Game Over title
        parent.spawn((
            Text::new("GAME OVER"),
            TextFont {
                font_size: 48.0,
                ..default()
            },
            TextColor(Color::srgb(1.0, 0.2, 0.2)),
            Node {
                margin: UiRect::bottom(Val::Px(20.0)),
                ..default()
            },
        ));

        // Final score
        parent.spawn((
            Text::new(format!("Final Score: {}", score.current)),
            TextFont {
                font_size: 32.0,
                ..default()
            },
            TextColor(Color::WHITE),
            Node {
                margin: UiRect::bottom(Val::Px(30.0)),
                ..default()
            },
        ));

        // Restart button
        parent.spawn((
            Button,
            Node {
                width: Val::Px(200.0),
                height: Val::Px(60.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0.2, 0.6, 0.2)),
            RestartButton,
        ))
        .with_children(|button| {
            button.spawn((
                Text::new("RESTART"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
    });
}

/// Handles restart button clicks
pub fn handle_restart_button(
    mut interaction_query: Query<&Interaction, (Changed<Interaction>, With<RestartButton>)>,
    mut commands: Commands,
    score: ResMut<Score>,
    game_state: ResMut<GameState>,
    overlay_query: Query<Entity, With<GameOverOverlay>>,
    entities_query: Query<Entity, (Or<(With<Enemy>, With<Projectile>)>, Without<Player>, Without<MainCamera>)>,
    dungeon_query: Query<Entity, With<DungeonWall>>,
    floor_query: Query<Entity, (With<Mesh2d>, Without<Player>, Without<MainCamera>, Without<DungeonWall>, Without<Enemy>)>,
    fire_timer: ResMut<FireTimer>,
    scene_manager: ResMut<crate::world::scenes::SceneManager>,
) {
    let mut should_restart = false;

    // Check if any restart button was pressed
    for interaction in &mut interaction_query {
        if *interaction == Interaction::Pressed {
            should_restart = true;
            break;
        }
    }

    if should_restart {
        println!("Restarting game - generating new dungeon!");

        // Remove game over overlay
        for entity in overlay_query.iter() {
            commands.entity(entity).despawn();
        }

        // Clean up all game entities (enemies, projectiles)
        cleanup_game_entities(&mut commands, &entities_query);

        // Clean up the entire dungeon (walls, floors) for complete regeneration
        cleanup_dungeon_entities(&mut commands, &dungeon_query, &floor_query);

        // Reset to Cathedral scene using scene system
        reset_to_cathedral(
            scene_manager,
            score,
            game_state,
            fire_timer,
        );
    }
}

/// Shows game over overlay when game state changes
pub fn show_game_over_overlay(
    commands: Commands,
    game_state: Res<GameState>,
    score: Res<Score>,
    overlay_query: Query<Entity, With<GameOverOverlay>>,
) {
    if *game_state == GameState::GameOver && overlay_query.is_empty() {
        setup_game_over_overlay(commands, score);
    }
}
