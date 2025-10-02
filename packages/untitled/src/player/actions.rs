use bevy::prelude::*;

/// Player-specific action events
#[derive(Event, Debug, Clone)]
pub struct PlayerActionEvent {
    pub action: PlayerAction,
    pub state: ActionState,
    pub value: f32,           // 0.0-1.0 for analog inputs, 0.0 or 1.0 for digital
    pub world_position: Option<Vec2>, // World coordinates for positional actions
}

/// Player actions that can be performed
#[derive(Debug, Clone, PartialEq)]
pub enum PlayerAction {
    // Movement
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    Dash,

    // Combat
    Shoot,
    ThrowGrenade,
    Reload,

    // Interaction
    Interact,

    // Camera
    Look(Vec2), // Mouse delta for camera control
}

/// State of a player action
#[derive(Debug, Clone, PartialEq)]
pub enum ActionState {
    Started,    // Action just began
    Ongoing,    // Action is continuing
    Completed,  // Action just ended
}

/// Resource that holds player input bindings
#[derive(Resource)]
pub struct PlayerInputBindings {
    // Movement keys
    pub move_up: KeyCode,
    pub move_down: KeyCode,
    pub move_left: KeyCode,
    pub move_right: KeyCode,
    pub dash: KeyCode,

    // Combat
    pub shoot: MouseButton,
    pub throw_grenade: MouseButton,
    pub reload: KeyCode,

    // Interaction
    pub interact: KeyCode,
}

impl Default for PlayerInputBindings {
    fn default() -> Self {
        Self {
            // WASD movement
            move_up: KeyCode::KeyW,
            move_down: KeyCode::KeyS,
            move_left: KeyCode::KeyA,
            move_right: KeyCode::KeyD,
            dash: KeyCode::Space,

            // Mouse combat
            shoot: MouseButton::Left,
            throw_grenade: MouseButton::Right,
            reload: KeyCode::KeyR,

            // Interaction
            interact: KeyCode::KeyE,
        }
    }
}

impl PlayerActionEvent {
    pub fn new(action: PlayerAction, state: ActionState, value: f32) -> Self {
        Self {
            action,
            state,
            value,
            world_position: None,
        }
    }

    pub fn with_world_position(mut self, position: Vec2) -> Self {
        self.world_position = Some(position);
        self
    }

    /// Check if this action just started
    pub fn just_started(&self) -> bool {
        matches!(self.state, ActionState::Started)
    }

    /// Check if this action is active (started or ongoing)
    pub fn is_active(&self) -> bool {
        matches!(self.state, ActionState::Started | ActionState::Ongoing)
    }

    /// Check if this action just completed
    pub fn just_completed(&self) -> bool {
        matches!(self.state, ActionState::Completed)
    }

    /// Get the action value as a boolean (for digital actions)
    pub fn as_bool(&self) -> bool {
        self.value > 0.5
    }

    /// Get the action as a direction vector (for movement actions)
    pub fn as_direction(&self) -> Vec2 {
        match self.action {
            PlayerAction::MoveUp => Vec2::Y * self.value,
            PlayerAction::MoveDown => Vec2::NEG_Y * self.value,
            PlayerAction::MoveLeft => Vec2::NEG_X * self.value,
            PlayerAction::MoveRight => Vec2::X * self.value,
            _ => Vec2::ZERO,
        }
    }
}
