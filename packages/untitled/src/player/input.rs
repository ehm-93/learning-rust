use bevy::{prelude::*, input::mouse::MouseWheel};
use crate::player::actions::{PlayerAction, PlayerActionEvent, ActionState, PlayerInputBindings};

/// System that converts raw input into player action events
pub fn player_input_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: EventReader<CursorMoved>,
    mut scroll_events: EventReader<MouseWheel>,
    bindings: Res<PlayerInputBindings>,
    mut action_events: EventWriter<PlayerActionEvent>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
) {
    // Handle movement keys
    handle_movement_input(&keyboard, &bindings, &mut action_events);

    // Handle combat input
    handle_combat_input(&keyboard, &mouse_buttons, &bindings, &mut action_events, &windows, &cameras);

    // Handle camera/look input
    handle_camera_input(&mut mouse_motion, &mut action_events);

    // Handle scroll wheel zoom
    handle_scroll_input(&mut scroll_events, &mut action_events);
}

fn handle_movement_input(
    keyboard: &Res<ButtonInput<KeyCode>>,
    bindings: &Res<PlayerInputBindings>,
    action_events: &mut EventWriter<PlayerActionEvent>,
) {
    // Movement actions
    let movement_bindings = [
        (bindings.move_up, PlayerAction::MoveUp),
        (bindings.move_down, PlayerAction::MoveDown),
        (bindings.move_left, PlayerAction::MoveLeft),
        (bindings.move_right, PlayerAction::MoveRight),
        (bindings.dash, PlayerAction::Dash),
    ];

    for (key, action) in movement_bindings {
        if keyboard.just_pressed(key) {
            action_events.write(PlayerActionEvent::new(action, ActionState::Started, 1.0));
        } else if keyboard.pressed(key) {
            action_events.write(PlayerActionEvent::new(action, ActionState::Ongoing, 1.0));
        } else if keyboard.just_released(key) {
            action_events.write(PlayerActionEvent::new(action, ActionState::Completed, 0.0));
        }
    }
}

fn handle_combat_input(
    keyboard: &Res<ButtonInput<KeyCode>>,
    mouse_buttons: &Res<ButtonInput<MouseButton>>,
    bindings: &Res<PlayerInputBindings>,
    action_events: &mut EventWriter<PlayerActionEvent>,
    windows: &Query<&Window>,
    cameras: &Query<(&Camera, &GlobalTransform)>,
) {
    // Mouse combat actions
    let mouse_bindings = [
        (bindings.shoot, PlayerAction::Shoot),
        (bindings.throw_grenade, PlayerAction::ThrowGrenade),
    ];

    for (button, action) in mouse_bindings {
        if mouse_buttons.just_pressed(button) {
            let mut event = PlayerActionEvent::new(action, ActionState::Started, 1.0);

            // Add world position for targeting
            if let Some(world_pos) = get_mouse_world_position(windows, cameras) {
                event = event.with_world_position(world_pos);
            }

            action_events.write(event);
        } else if mouse_buttons.pressed(button) {
            let mut event = PlayerActionEvent::new(action, ActionState::Ongoing, 1.0);

            // Add world position for targeting
            if let Some(world_pos) = get_mouse_world_position(windows, cameras) {
                event = event.with_world_position(world_pos);
            }

            action_events.write(event);
        } else if mouse_buttons.just_released(button) {
            action_events.write(PlayerActionEvent::new(action, ActionState::Completed, 0.0));
        }
    }

    // Keyboard combat actions
    if keyboard.just_pressed(bindings.reload) {
        action_events.write(PlayerActionEvent::new(PlayerAction::Reload, ActionState::Started, 1.0));
    }

    // Interaction actions
    if keyboard.just_pressed(bindings.interact) {
        action_events.write(PlayerActionEvent::new(PlayerAction::Interact, ActionState::Started, 1.0));
    }
}

fn handle_camera_input(
    mouse_motion: &mut EventReader<CursorMoved>,
    action_events: &mut EventWriter<PlayerActionEvent>,
) {
    for motion in mouse_motion.read() {
        if let Some(delta) = motion.delta {
            if delta.length() > 0.0 {
                action_events.write(PlayerActionEvent::new(
                    PlayerAction::Look(delta),
                    ActionState::Started,
                    delta.length(),
                ));
            }
        }
    }
}

/// Handle scroll wheel input for camera zoom
fn handle_scroll_input(
    scroll_events: &mut EventReader<MouseWheel>,
    action_events: &mut EventWriter<PlayerActionEvent>,
) {
    for scroll in scroll_events.read() {
        if scroll.y > 0.0 {
            // Scroll up = zoom in
            action_events.write(PlayerActionEvent::new(
                PlayerAction::ZoomIn,
                ActionState::Started,
                scroll.y,
            ));
        } else if scroll.y < 0.0 {
            // Scroll down = zoom out
            action_events.write(PlayerActionEvent::new(
                PlayerAction::ZoomOut,
                ActionState::Started,
                -scroll.y,
            ));
        }
    }
}

/// Helper function to get mouse position in world coordinates
fn get_mouse_world_position(
    windows: &Query<&Window>,
    cameras: &Query<(&Camera, &GlobalTransform)>,
) -> Option<Vec2> {
    let window = windows.single().ok()?;
    let (camera, camera_transform) = cameras.single().ok()?;

    let cursor_pos = window.cursor_position()?;
    camera.viewport_to_world_2d(camera_transform, cursor_pos).ok()
}
