use macroquad::prelude::*;

#[macroquad::main("Twin Stick Shooter")]
async fn main() {
    let mut player_x = screen_width() / 2.0;
    let mut player_y = screen_height() / 2.0;
    let player_speed = 200.0;

    loop {
        // Handle input
        if is_key_down(KeyCode::W) { player_y -= player_speed * get_frame_time(); }
        if is_key_down(KeyCode::S) { player_y += player_speed * get_frame_time(); }
        if is_key_down(KeyCode::A) { player_x -= player_speed * get_frame_time(); }
        if is_key_down(KeyCode::D) { player_x += player_speed * get_frame_time(); }

        // Keep player on screen
        player_x = player_x.clamp(10.0, screen_width() - 10.0);
        player_y = player_y.clamp(10.0, screen_height() - 10.0);

        // Clear and draw
        clear_background(BLACK);
        draw_circle(player_x, player_y, 10.0, WHITE);

        next_frame().await;
    }
}
