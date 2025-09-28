use bevy::prelude::*;
use crate::{
    components::*,
};

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
