use macroquad::prelude::*;

use crate::game::{Game, GameState};
use crate::map::{SCREEN_H, SCREEN_W, SIDEBAR_W, SIDEBAR_X, TOP_BAR, in_map};
use crate::tower::{PATHS, TowerKind};

const PAD: f32 = 10.0;
const INNER_W: f32 = SIDEBAR_W - 2.0 * PAD;

pub fn tower_button(i: usize) -> Rect {
    Rect::new(
        SIDEBAR_X + PAD,
        TOP_BAR + 36.0 + i as f32 * 64.0,
        INNER_W,
        58.0,
    )
}

pub fn cancel_button() -> Rect {
    Rect::new(SIDEBAR_X + PAD, TOP_BAR + 360.0, INNER_W, 36.0)
}

pub fn close_button() -> Rect {
    Rect::new(SCREEN_W - PAD - 28.0, TOP_BAR + 10.0, 28.0, 28.0)
}

pub fn path_card(path: usize) -> Rect {
    Rect::new(
        SIDEBAR_X + PAD,
        TOP_BAR + 150.0 + path as f32 * 92.0,
        INNER_W,
        84.0,
    )
}

pub fn sell_button() -> Rect {
    Rect::new(SIDEBAR_X + PAD, TOP_BAR + 430.0, INNER_W, 36.0)
}

pub fn wave_button() -> Rect {
    Rect::new(SIDEBAR_X + PAD, SCREEN_H - 56.0, 168.0, 46.0)
}

pub fn auto_button() -> Rect {
    Rect::new(
        SIDEBAR_X + PAD + 176.0,
        SCREEN_H - 56.0,
        INNER_W - 176.0,
        46.0,
    )
}

pub fn map_button() -> Rect {
    Rect::new(SCREEN_W - 420.0, 8.0, 180.0, 32.0)
}

pub fn pause_button() -> Rect {
    Rect::new(SCREEN_W - 230.0, 8.0, 106.0, 32.0)
}

pub fn speed_button() -> Rect {
    Rect::new(SCREEN_W - 116.0, 8.0, 106.0, 32.0)
}

pub const PATH_KEYS: [KeyCode; PATHS] = [KeyCode::Comma, KeyCode::Period, KeyCode::Slash];
pub const PATH_KEY_LABELS: [&str; PATHS] = [",", ".", "/"];

pub fn handle_input(game: &mut Game) {
    if game.state != GameState::Playing {
        if is_key_pressed(KeyCode::R) || is_key_pressed(KeyCode::Enter) {
            *game = Game::with_map(game.map_index);
        }
        return;
    }

    let hotkeys = [
        KeyCode::Key1,
        KeyCode::Key2,
        KeyCode::Key3,
        KeyCode::Key4,
        KeyCode::Key5,
    ];
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
    if is_key_pressed(KeyCode::A) {
        game.toggle_auto();
    }
    if is_key_pressed(KeyCode::F) {
        cycle_speed(game);
    }
    if is_key_pressed(KeyCode::P) {
        game.paused = !game.paused;
    }
    if is_key_pressed(KeyCode::M) {
        game.next_map();
    }
    for (path, key) in PATH_KEYS.into_iter().enumerate() {
        if is_key_pressed(key) {
            game.upgrade_selected(path);
        }
    }
    if is_key_pressed(KeyCode::S) || is_key_pressed(KeyCode::Delete) {
        game.sell_selected();
    }

    if !is_mouse_button_pressed(MouseButton::Left) {
        return;
    }
    let mouse: Vec2 = mouse_position().into();

    if in_map(mouse, 0.0) {
        if let Some(kind) = game.build_choice {
            // Hold shift to keep placing the same tower.
            let keep = is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift);
            if game.place(kind, mouse) && !keep {
                game.build_choice = None;
            }
        } else {
            game.selected = game.tower_at(mouse);
        }
        return;
    }

    if wave_button().contains(mouse) {
        game.start_wave();
    } else if auto_button().contains(mouse) {
        game.toggle_auto();
    } else if pause_button().contains(mouse) {
        game.paused = !game.paused;
    } else if speed_button().contains(mouse) {
        cycle_speed(game);
    } else if game.wave == 0 && map_button().contains(mouse) {
        game.next_map();
    } else if game.selected.is_some() {
        if close_button().contains(mouse) {
            game.selected = None;
        } else if sell_button().contains(mouse) {
            game.sell_selected();
        }
        for path in 0..PATHS {
            if path_card(path).contains(mouse) {
                game.upgrade_selected(path);
            }
        }
    } else {
        for (i, kind) in TowerKind::ALL.into_iter().enumerate() {
            if tower_button(i).contains(mouse) {
                choose_build(game, kind);
            }
        }
        if game.build_choice.is_some() && cancel_button().contains(mouse) {
            game.build_choice = None;
        }
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
