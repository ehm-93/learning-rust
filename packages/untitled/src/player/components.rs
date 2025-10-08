use bevy::prelude::*;
use crate::constants::*;

/// Player marker component
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Player;

/// Dash ability component
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Dash {
    pub cooldown_timer: Timer,
    pub dash_timer: Timer,
    pub iframe_timer: Timer,
    pub is_dashing: bool,
    pub is_invincible: bool,
    pub dash_direction: Vec2,
}

impl Dash {
    pub fn new() -> Self {
        use crate::constants::*;
        Self {
            cooldown_timer: Timer::from_seconds(DASH_COOLDOWN, TimerMode::Once),
            dash_timer: Timer::from_seconds(DASH_DURATION, TimerMode::Once),
            iframe_timer: Timer::from_seconds(DASH_IFRAME_DURATION, TimerMode::Once),
            is_dashing: false,
            is_invincible: false,
            dash_direction: Vec2::ZERO,
        }
    }

    pub fn can_dash(&self) -> bool {
        self.cooldown_timer.finished() && !self.is_dashing
    }

    pub fn start_dash(&mut self, direction: Vec2) {
        if self.can_dash() {
            self.is_dashing = true;
            self.is_invincible = true;
            self.dash_direction = direction.normalize_or_zero();
            self.dash_timer.reset();
            self.iframe_timer.reset();
            self.cooldown_timer.reset();
        }
    }
}

/// Grenade throwing capability component
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct GrenadeThrower {
    pub cooldown_timer: Timer,
}

impl GrenadeThrower {
    pub fn new() -> Self {
        use crate::constants::*;
        Self {
            cooldown_timer: Timer::from_seconds(GRENADE_THROW_COOLDOWN, TimerMode::Once),
        }
    }

    pub fn can_throw(&self) -> bool {
        self.cooldown_timer.finished()
    }

    pub fn throw_grenade(&mut self) {
        self.cooldown_timer.reset();
    }
}

/// Complete player bundle with all necessary components for spawning
#[derive(Bundle)]
pub struct PlayerBundle {
    pub player: Player,
    pub team: crate::components::Team,
    pub health: crate::components::Health,
    pub dash: Dash,
    pub grenade_thrower: GrenadeThrower,
    pub inventory: crate::inventory::Inventory,
    pub chunk_loader: crate::world::chunks::ChunkLoader,
    pub fow_revealer: crate::combat::FowRevealer,

    // Visual components
    pub mesh: Mesh2d,
    pub material: MeshMaterial2d<ColorMaterial>,
    pub transform: Transform,
    pub visibility: Visibility,

    // Physics components
    pub rigid_body: bevy_rapier2d::prelude::RigidBody,
    pub collider: bevy_rapier2d::prelude::Collider,
    pub velocity: bevy_rapier2d::prelude::Velocity,
    pub locked_axes: bevy_rapier2d::prelude::LockedAxes,
    pub active_events: bevy_rapier2d::prelude::ActiveEvents,
}

impl PlayerBundle {
    pub fn new(
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<ColorMaterial>>,
        position: Vec3,
    ) -> Self {
        // Create mesh and material handles
        let mesh_handle = meshes.add(Circle::new(PLAYER_RADIUS));
        let material_handle = materials.add(Color::WHITE);

        Self {
            player: Player,
            team: crate::components::Team::Player,
            health: crate::components::Health::new(PLAYER_MAX_HEALTH),
            dash: Dash::new(),
            grenade_thrower: GrenadeThrower::new(),
            inventory: crate::inventory::Inventory::player_inventory(),
            chunk_loader: crate::world::chunks::ChunkLoader::new(16),
            fow_revealer: crate::combat::FowRevealer::new(24),

            // Visual components
            mesh: Mesh2d(mesh_handle),
            material: MeshMaterial2d(material_handle),
            transform: Transform::from_translation(position),
            visibility: Visibility::Visible,

            // Physics components
            rigid_body: bevy_rapier2d::prelude::RigidBody::Dynamic,
            collider: bevy_rapier2d::prelude::Collider::ball(PLAYER_RADIUS),
            velocity: bevy_rapier2d::prelude::Velocity::zero(),
            locked_axes: bevy_rapier2d::prelude::LockedAxes::ROTATION_LOCKED,
            active_events: bevy_rapier2d::prelude::ActiveEvents::COLLISION_EVENTS,
        }
    }
}
