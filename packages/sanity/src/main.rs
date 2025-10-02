use macroquad::prelude::*;

#[macroquad::main("Sanity")]
async fn main() {
    loop {
        clear_background(RED);
        draw_text("WSL + Rust works!", 20.0, 20.0, 30.0, WHITE);
        next_frame().await;
    }
}
