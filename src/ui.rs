use macroquad::prelude::*;

use crate::game::{Game, GameState};
use crate::map::{SCREEN_H, SCREEN_W, SIDEBAR_X, TOP_BAR, tile_at};
use crate::tower::TowerKind;

pub fn tower_button(i: usize) -> Rect {
    Rect::new(
        SIDEBAR_X + 10.0,
        TOP_BAR + 34.0 + i as f32 * 74.0,
        220.0,
        66.0,
    )
}

pub fn upgrade_button() -> Rect {
    Rect::new(SIDEBAR_X + 10.0, SCREEN_H - 112.0, 106.0, 40.0)
}

pub fn sell_button() -> Rect {
    Rect::new(SIDEBAR_X + 124.0, SCREEN_H - 112.0, 106.0, 40.0)
}

pub fn wave_button() -> Rect {
    Rect::new(SIDEBAR_X + 10.0, SCREEN_H - 60.0, 220.0, 48.0)
}

pub fn pause_button() -> Rect {
    Rect::new(SCREEN_W - 230.0, 8.0, 106.0, 32.0)
}

pub fn speed_button() -> Rect {
    Rect::new(SCREEN_W - 116.0, 8.0, 106.0, 32.0)
}

pub fn handle_input(game: &mut Game) {
    if game.state != GameState::Playing {
        if is_key_pressed(KeyCode::R) || is_key_pressed(KeyCode::Enter) {
            *game = Game::new();
        }
        return;
    }

    let hotkeys = [KeyCode::Key1, KeyCode::Key2, KeyCode::Key3, KeyCode::Key4];
    for (key, kind) in hotkeys.into_iter().zip(TowerKind::ALL) {
        if is_key_pressed(key) {
            choose_build(game, kind);
        }
    }
    if is_key_pressed(KeyCode::Escape) || is_mouse_button_pressed(MouseButton::Right) {
        game.build_choice = None;
        game.selected = None;
    }
    if is_key_pressed(KeyCode::Space) || is_key_pressed(KeyCode::N) {
        game.start_wave();
    }
    if is_key_pressed(KeyCode::F) {
        cycle_speed(game);
    }
    if is_key_pressed(KeyCode::P) {
        game.paused = !game.paused;
    }
    if is_key_pressed(KeyCode::U) {
        game.upgrade_selected();
    }
    if is_key_pressed(KeyCode::S) || is_key_pressed(KeyCode::Delete) {
        game.sell_selected();
    }

    if !is_mouse_button_pressed(MouseButton::Left) {
        return;
    }
    let mouse: Vec2 = mouse_position().into();

    if let Some((col, row)) = tile_at(mouse) {
        if let Some(kind) = game.build_choice {
            // Hold shift to keep placing the same tower.
            let keep = is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift);
            if game.build(kind, col, row) && !keep {
                game.build_choice = None;
            }
        } else {
            game.selected = game.tower_at(col, row);
        }
        return;
    }

    for (i, kind) in TowerKind::ALL.into_iter().enumerate() {
        if tower_button(i).contains(mouse) {
            choose_build(game, kind);
        }
    }
    if upgrade_button().contains(mouse) {
        game.upgrade_selected();
    } else if sell_button().contains(mouse) {
        game.sell_selected();
    } else if wave_button().contains(mouse) {
        game.start_wave();
    } else if pause_button().contains(mouse) {
        game.paused = !game.paused;
    } else if speed_button().contains(mouse) {
        cycle_speed(game);
    }
}

fn choose_build(game: &mut Game, kind: TowerKind) {
    game.selected = None;
    game.build_choice = if game.build_choice == Some(kind) {
        None
    } else {
        Some(kind)
    };
}

fn cycle_speed(game: &mut Game) {
    game.speed = match game.speed {
        1 => 2,
        2 => 3,
        _ => 1,
    };
}
