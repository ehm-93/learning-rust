pub trait System {
    fn tick_start(&mut self, dt: f32);
    fn tick_work(&mut self) -> bool { false }
    fn tick_end(&mut self) {}
}

mod behavior;
mod collision;
mod combat;
mod movement;

pub use behavior::BehaviorSystem;
pub use collision::CollisionSystem;
pub use combat::CombatSystem;
pub use movement::MovementSystem;
