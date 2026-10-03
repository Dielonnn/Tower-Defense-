mod enemy;
mod game;
mod map;
mod render;
mod tower;
mod ui;
mod wave;

use macroquad::prelude::*;

use game::Game;

fn window_conf() -> Conf {
    Conf {
        window_title: "Rusty Tower Defense".to_owned(),
        window_width: map::SCREEN_W as i32,
        window_height: map::SCREEN_H as i32,
        window_resizable: false,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut game = Game::new();

    loop {
        ui::handle_input(&mut game);

        // Clamp the frame time so a stall doesn't teleport enemies, and run
        // several fixed steps for fast-forward rather than one big one.
        let dt = get_frame_time().min(1.0 / 20.0);
        for _ in 0..game.speed {
            game.update(dt);
        }

        render::draw(&game);
        next_frame().await
    }
}
