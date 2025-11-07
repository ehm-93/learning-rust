use bevy::prelude::*;
use std::env;

mod editor;
mod game;

fn main() {
    let args: Vec<String> = env::args().collect();
    let use_editor = args.contains(&"--editor".to_string());

    let mut app = App::new();

    if use_editor {
        app.add_plugins(editor::EditorPlugin);
    } else {
        app.add_plugins(game::GamePlugin);
    }

    app.run();
}
