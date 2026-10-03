use macroquad::prelude::*;

use crate::enemy::EnemyKind;
use crate::game::{Game, GameState, Menu};
use crate::map::{SCREEN_H, SCREEN_W, SIDEBAR_W, SIDEBAR_X, TOP_BAR, in_map};
use crate::tower::{PATHS, TIERS, TowerKind};

const PAD: f32 = 10.0;
const INNER_W: f32 = SIDEBAR_W - 2.0 * PAD;

// Build panel.

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

pub const SANDBOX_SPAWNS: [EnemyKind; 4] = [
    EnemyKind::Grunt,
    EnemyKind::Runner,
    EnemyKind::Tank,
    EnemyKind::Boss,
];

pub fn sandbox_spawn_button(i: usize) -> Rect {
    let w = (INNER_W - 8.0) / 2.0;
    let x = SIDEBAR_X + PAD + (i % 2) as f32 * (w + 8.0);
    let y = TOP_BAR + 432.0 + (i / 2) as f32 * 36.0;
    Rect::new(x, y, w, 30.0)
}

pub fn sandbox_wave_down() -> Rect {
    Rect::new(SIDEBAR_X + PAD, TOP_BAR + 506.0, 36.0, 30.0)
}

pub fn sandbox_wave_up() -> Rect {
    Rect::new(
        SIDEBAR_X + PAD + INNER_W - 36.0,
        TOP_BAR + 506.0,
        36.0,
        30.0,
    )
}

pub fn sandbox_clear_button() -> Rect {
    Rect::new(SIDEBAR_X + PAD, TOP_BAR + 544.0, INNER_W, 30.0)
}

// Tower panel.

pub fn close_button() -> Rect {
    Rect::new(SCREEN_W - PAD - 28.0, TOP_BAR + 10.0, 28.0, 28.0)
}

pub fn target_button() -> Rect {
    Rect::new(SIDEBAR_X + PAD, TOP_BAR + 120.0, INNER_W, 28.0)
}

pub fn path_card(path: usize) -> Rect {
    Rect::new(
        SIDEBAR_X + PAD,
        TOP_BAR + 156.0 + path as f32 * 92.0,
        INNER_W,
        84.0,
    )
}

pub fn sell_button() -> Rect {
    Rect::new(SIDEBAR_X + PAD, TOP_BAR + 432.0, INNER_W, 36.0)
}

// Always visible.

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

pub fn settings_button() -> Rect {
    Rect::new(SCREEN_W - 530.0, 8.0, 100.0, 32.0)
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

// Settings menu.

pub fn settings_panel() -> Rect {
    Rect::new(
        (SCREEN_W - 460.0) / 2.0,
        (SCREEN_H - 430.0) / 2.0,
        460.0,
        430.0,
    )
}

pub const SETTINGS_ITEMS: usize = 5;

pub fn settings_item(i: usize) -> Rect {
    let p = settings_panel();
    Rect::new(p.x + 40.0, p.y + 66.0 + i as f32 * 54.0, p.w - 80.0, 44.0)
}

// Tower guide.

pub fn guide_panel() -> Rect {
    Rect::new(40.0, 40.0, SCREEN_W - 80.0, SCREEN_H - 80.0)
}

pub fn guide_close() -> Rect {
    let p = guide_panel();
    Rect::new(p.x + p.w - 44.0, p.y + 12.0, 32.0, 32.0)
}

pub fn guide_tab(i: usize) -> Rect {
    let p = guide_panel();
    Rect::new(p.x + 20.0 + i as f32 * 158.0, p.y + 56.0, 150.0, 36.0)
}

pub fn guide_card(path: usize, tier: usize) -> Rect {
    let p = guide_panel();
    let w = (p.w - 40.0 - 2.0 * 16.0) / PATHS as f32;
    Rect::new(
        p.x + 20.0 + path as f32 * (w + 16.0),
        p.y + 236.0 + tier as f32 * 88.0,
        w,
        80.0,
    )
}

pub const PATH_KEYS: [KeyCode; PATHS] = [KeyCode::Comma, KeyCode::Period, KeyCode::Slash];
pub const PATH_KEY_LABELS: [&str; PATHS] = [",", ".", "/"];

pub fn handle_input(game: &mut Game) {
    let mouse: Vec2 = mouse_position().into();
    let click = is_mouse_button_pressed(MouseButton::Left);

    if let Some(menu) = game.menu {
        handle_menu(game, menu, mouse, click);
        return;
    }
    if click && settings_button().contains(mouse) {
        game.menu = Some(Menu::Settings);
        return;
    }

    if game.state != GameState::Playing {
        if is_key_pressed(KeyCode::R) || is_key_pressed(KeyCode::Enter) {
            game.restart(game.map_index, game.sandbox);
        }
        if is_key_pressed(KeyCode::Escape) {
            game.menu = Some(Menu::Settings);
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
    if is_key_pressed(KeyCode::Escape) {
        if game.build_choice.is_some() || game.selected.is_some() {
            game.build_choice = None;
            game.selected = None;
        } else {
            game.menu = Some(Menu::Settings);
        }
    }
    if is_mouse_button_pressed(MouseButton::Right) {
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
    if is_key_pressed(KeyCode::Tab) {
        game.cycle_targeting();
    }
    for (path, key) in PATH_KEYS.into_iter().enumerate() {
        if is_key_pressed(key) {
            game.upgrade_selected(path);
        }
    }
    if is_key_pressed(KeyCode::S) || is_key_pressed(KeyCode::Delete) {
        game.sell_selected();
    }

    if !click {
        return;
    }

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
        } else if target_button().contains(mouse) {
            game.cycle_targeting();
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
        if game.sandbox {
            handle_sandbox_click(game, mouse);
        }
    }
}

fn handle_sandbox_click(game: &mut Game, mouse: Vec2) {
    // Shift-click spawns five at once.
    let count = if is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift) {
        5
    } else {
        1
    };
    for (i, kind) in SANDBOX_SPAWNS.into_iter().enumerate() {
        if sandbox_spawn_button(i).contains(mouse) {
            for _ in 0..count {
                game.sandbox_spawn(kind);
            }
        }
    }
    let step = if count > 1 { 5 } else { 1 };
    if sandbox_wave_down().contains(mouse) {
        game.sandbox_set_wave(game.wave as i32 - step);
    } else if sandbox_wave_up().contains(mouse) {
        game.sandbox_set_wave(game.wave as i32 + step);
    } else if sandbox_clear_button().contains(mouse) {
        game.sandbox_clear();
    }
}

fn handle_menu(game: &mut Game, menu: Menu, mouse: Vec2, click: bool) {
    match menu {
        Menu::Settings => {
            if is_key_pressed(KeyCode::Escape) {
                game.menu = None;
                return;
            }
            if !click {
                return;
            }
            let hit = (0..SETTINGS_ITEMS).find(|&i| settings_item(i).contains(mouse));
            match hit {
                Some(0) => game.night = !game.night,
                Some(1) => game.menu = Some(Menu::Guide(0)),
                Some(2) => game.restart(game.map_index, false),
                Some(3) => game.restart(game.map_index, true),
                Some(_) => game.menu = None,
                None if !settings_panel().contains(mouse) => game.menu = None,
                None => {}
            }
        }
        Menu::Guide(i) => {
            let n = TowerKind::ALL.len();
            if is_key_pressed(KeyCode::Escape) {
                game.menu = Some(Menu::Settings);
            }
            if is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::Tab) {
                game.menu = Some(Menu::Guide((i + 1) % n));
            }
            if is_key_pressed(KeyCode::Left) {
                game.menu = Some(Menu::Guide((i + n - 1) % n));
            }
            if !click {
                return;
            }
            if guide_close().contains(mouse) {
                game.menu = Some(Menu::Settings);
            }
            for j in 0..n {
                if guide_tab(j).contains(mouse) {
                    game.menu = Some(Menu::Guide(j));
                }
            }
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

/// Number of upgrade tiers, for layout code that iterates them.
pub const TIER_COUNT: usize = TIERS as usize;
