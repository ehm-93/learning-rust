use std::time::Duration;

use macroquad::prelude::{
    get_frame_time,
    clear_background,
    draw_text,
    next_frame,
    get_fps,
    BLACK,
    WHITE,
};
use untitled::{game::GameState, systems::{BehaviorSystem, CollisionSystem, CombatSystem, MovementSystem}};

#[macroquad::main("Untitled")]
async fn main() {
    // Initialize game state
    let mut game_state = GameState::new(
        Duration::from_millis(10),
        vec![
            Box::new(BehaviorSystem::new()),
            Box::new(CombatSystem::new()),
            Box::new(MovementSystem::new()),
            Box::new(CollisionSystem::new()),
        ]
    );

    loop {
        let dt = get_frame_time();

        // Update game logic
        game_state.tick(dt);

        // Render everything
        clear_background(BLACK);
        game_state.render();

        // Debug info
        draw_text(&format!("FPS: {:.0}", get_fps()), 10.0, 20.0, 20.0, WHITE);
        draw_text(&format!("Entities: {}", game_state.entity_count()), 10.0, 40.0, 20.0, WHITE);

        next_frame().await;
    }
}
