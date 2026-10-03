use macroquad::prelude::*;

use crate::enemy::EnemyKind;
use crate::game::{Action, Game, GameState, Menu, Request};
use crate::map::{
    MAP_COUNT, MAP_H, MAP_W, Map, SCREEN_H, SCREEN_W, SIDEBAR_W, SIDEBAR_X, TOP_BAR, in_map,
};
use crate::tower::{PATHS, TIERS, TowerKind};

const PAD: f32 = 10.0;
const INNER_W: f32 = SIDEBAR_W - 2.0 * PAD;

// Build panel.

pub fn tower_button(i: usize) -> Rect {
    Rect::new(
        SIDEBAR_X + PAD,
        TOP_BAR + 34.0 + i as f32 * 56.0,
        INNER_W,
        52.0,
    )
}

pub fn cancel_button() -> Rect {
    Rect::new(SIDEBAR_X + PAD, TOP_BAR + 372.0, INNER_W, 34.0)
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
    let y = TOP_BAR + 444.0 + (i / 2) as f32 * 36.0;
    Rect::new(x, y, w, 30.0)
}

pub fn sandbox_wave_down() -> Rect {
    Rect::new(SIDEBAR_X + PAD, TOP_BAR + 518.0, 36.0, 30.0)
}

pub fn sandbox_wave_up() -> Rect {
    Rect::new(
        SIDEBAR_X + PAD + INNER_W - 36.0,
        TOP_BAR + 518.0,
        36.0,
        30.0,
    )
}

pub fn sandbox_clear_button() -> Rect {
    Rect::new(SIDEBAR_X + PAD, TOP_BAR + 556.0, INNER_W, 30.0)
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

// Always visible in game.

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

pub fn stats_button() -> Rect {
    Rect::new(SCREEN_W - 460.0, 8.0, 100.0, 32.0)
}

/// The damage meter, drawn over the top right of the map.
pub fn damage_meter() -> Rect {
    Rect::new(MAP_W - 312.0, TOP_BAR + 10.0, 302.0, 84.0 + 6.0 * 30.0)
}

pub fn settings_button() -> Rect {
    Rect::new(SCREEN_W - 350.0, 8.0, 110.0, 32.0)
}

pub fn pause_button() -> Rect {
    Rect::new(SCREEN_W - 230.0, 8.0, 106.0, 32.0)
}

pub fn speed_button() -> Rect {
    Rect::new(SCREEN_W - 116.0, 8.0, 106.0, 32.0)
}

// Game over / victory overlay.

pub fn retry_button() -> Rect {
    let c = vec2(MAP_W / 2.0, TOP_BAR + MAP_H / 2.0);
    Rect::new(c.x - 230.0, c.y + 64.0, 220.0, 48.0)
}

pub fn new_game_button() -> Rect {
    let c = vec2(MAP_W / 2.0, TOP_BAR + MAP_H / 2.0);
    Rect::new(c.x + 10.0, c.y + 64.0, 220.0, 48.0)
}

// Settings menu.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettingsItem {
    Theme,
    Guide,
    TowerVolume,
    GameVolume,
    Restart,
    MainMenu,
    Close,
}

impl SettingsItem {
    pub fn is_slider(self) -> bool {
        matches!(self, Self::TowerVolume | Self::GameVolume)
    }
}

pub fn settings_items(in_game: bool, is_client: bool) -> Vec<SettingsItem> {
    use SettingsItem::*;
    let mut items = vec![Theme, Guide, TowerVolume, GameVolume];
    if in_game {
        if !is_client {
            items.push(Restart);
        }
        items.push(MainMenu);
    }
    items.push(Close);
    items
}

pub fn settings_panel(count: usize) -> Rect {
    let h = 96.0 + count as f32 * 54.0;
    Rect::new((SCREEN_W - 460.0) / 2.0, (SCREEN_H - h) / 2.0, 460.0, h)
}

pub fn settings_row(count: usize, i: usize) -> Rect {
    let p = settings_panel(count);
    Rect::new(p.x + 40.0, p.y + 66.0 + i as f32 * 54.0, p.w - 80.0, 44.0)
}

/// The draggable part of a volume slider row.
pub fn slider_track(row: Rect) -> Rect {
    Rect::new(
        row.x + 190.0,
        row.y + row.h / 2.0 - 5.0,
        row.w - 200.0,
        10.0,
    )
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

// Main menu.

pub fn lobby_panel() -> Rect {
    Rect::new((SCREEN_W - 800.0) / 2.0, 128.0, 800.0, 566.0)
}

pub fn lobby_map_card(i: usize) -> Rect {
    let p = lobby_panel();
    Rect::new(p.x + 20.0 + i as f32 * 258.0, p.y + 50.0, 244.0, 172.0)
}

pub fn lobby_mode_button(sandbox: bool) -> Rect {
    let p = lobby_panel();
    let i = if sandbox { 1.0 } else { 0.0 };
    Rect::new(p.x + 20.0 + i * 260.0, p.y + 266.0, 250.0, 42.0)
}

pub fn lobby_play() -> Rect {
    let p = lobby_panel();
    Rect::new(p.x + 20.0, p.y + 326.0, 760.0, 56.0)
}

pub fn lobby_host() -> Rect {
    let p = lobby_panel();
    Rect::new(p.x + 20.0, p.y + 428.0, 250.0, 44.0)
}

pub fn lobby_address() -> Rect {
    let p = lobby_panel();
    Rect::new(p.x + 290.0, p.y + 428.0, 330.0, 44.0)
}

pub fn lobby_join() -> Rect {
    let p = lobby_panel();
    Rect::new(p.x + 630.0, p.y + 428.0, 150.0, 44.0)
}

pub fn lobby_skills() -> Rect {
    let p = lobby_panel();
    Rect::new(p.x + 20.0, p.y + 492.0, 246.0, 40.0)
}

pub fn lobby_settings() -> Rect {
    let p = lobby_panel();
    Rect::new(p.x + 277.0, p.y + 492.0, 246.0, 40.0)
}

pub fn lobby_quit() -> Rect {
    let p = lobby_panel();
    Rect::new(p.x + 534.0, p.y + 492.0, 246.0, 40.0)
}

/// Level badge in the top left of the main menu (also opens the skill tree).
pub fn lobby_profile() -> Rect {
    Rect::new(16.0, 16.0, 220.0, 78.0)
}

// Skill tree.

pub fn skills_panel() -> Rect {
    Rect::new(40.0, 40.0, SCREEN_W - 80.0, SCREEN_H - 80.0)
}

pub fn skills_close() -> Rect {
    let p = skills_panel();
    Rect::new(p.x + p.w - 44.0, p.y + 12.0, 32.0, 32.0)
}

pub fn skills_reset() -> Rect {
    let p = skills_panel();
    Rect::new(p.x + 20.0, p.y + p.h - 56.0, 210.0, 40.0)
}

/// Each branch is a column: its tier 1 skill on top, the two tier 2 skills
/// it unlocks below.
pub fn skill_node(id: usize) -> Rect {
    use crate::profile::SKILLS;
    let p = skills_panel();
    let skill = &SKILLS[id];
    let branch = skill.parent.unwrap_or(id);
    let x = p.x + 20.0 + branch as f32 * 226.0;
    match skill.parent {
        None => Rect::new(x, p.y + 140.0, 212.0, 100.0),
        Some(parent) => {
            let k = SKILLS[parent]
                .children()
                .position(|c| c.id == id)
                .unwrap_or(0);
            Rect::new(x, p.y + 300.0 + k as f32 * 130.0, 212.0, 110.0)
        }
    }
}

/// Main menu state.
pub struct Lobby {
    pub map: usize,
    pub sandbox: bool,
    /// Host address typed into the join box.
    pub address: String,
    pub typing: bool,
    pub status: Option<String>,
    /// Waiting on a connection; buttons are disabled.
    pub busy: bool,
    /// The selected map, drawn behind the menu.
    pub preview: Map,
}

impl Lobby {
    pub fn new() -> Self {
        Self {
            map: 0,
            sandbox: false,
            address: "127.0.0.1".to_string(),
            typing: false,
            status: None,
            busy: false,
            preview: Map::new(0),
        }
    }
}

pub enum LobbyAction {
    Play,
    Host,
    Join(String),
    Quit,
}

pub const PATH_KEYS: [KeyCode; PATHS] = [KeyCode::Comma, KeyCode::Period, KeyCode::Slash];
pub const PATH_KEY_LABELS: [&str; PATHS] = [",", ".", "/"];
pub const TIER_COUNT: usize = TIERS as usize;

fn shift() -> bool {
    is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift)
}

pub fn handle_lobby(
    lobby: &mut Lobby,
    game: &mut Game,
    profile: &mut crate::profile::Profile,
) -> Option<LobbyAction> {
    if game.menu == Some(Menu::Skills) {
        handle_skills(game, profile);
        return None;
    }
    if game.menu.is_some() {
        handle_menu(game, false);
        return None;
    }
    let mouse: Vec2 = mouse_position().into();
    let click = is_mouse_button_pressed(MouseButton::Left);

    if lobby.typing {
        while let Some(c) = get_char_pressed() {
            if (c.is_ascii_graphic() || c == ' ') && lobby.address.len() < 64 {
                lobby.address.push(c);
            }
        }
        if is_key_pressed(KeyCode::Backspace) {
            lobby.address.pop();
        }
        if is_key_pressed(KeyCode::Enter) && !lobby.busy {
            lobby.typing = false;
            return Some(LobbyAction::Join(lobby.address.clone()));
        }
        if is_key_pressed(KeyCode::Escape) {
            lobby.typing = false;
        }
    } else {
        // Drain stray key presses so they don't show up later.
        while get_char_pressed().is_some() {}
        if is_key_pressed(KeyCode::Enter) && !lobby.busy {
            return Some(LobbyAction::Play);
        }
    }

    if !click {
        return None;
    }
    lobby.typing = lobby_address().contains(mouse);
    for i in 0..MAP_COUNT {
        if lobby_map_card(i).contains(mouse) && lobby.map != i {
            lobby.map = i;
            lobby.preview = Map::new(i);
        }
    }
    for sandbox in [false, true] {
        if lobby_mode_button(sandbox).contains(mouse) {
            lobby.sandbox = sandbox;
        }
    }
    if lobby_settings().contains(mouse) {
        game.menu = Some(Menu::Settings);
    }
    if lobby_skills().contains(mouse) || lobby_profile().contains(mouse) {
        game.menu = Some(Menu::Skills);
    }
    if lobby_quit().contains(mouse) {
        return Some(LobbyAction::Quit);
    }
    if lobby.busy {
        return None;
    }
    if lobby_play().contains(mouse) {
        Some(LobbyAction::Play)
    } else if lobby_host().contains(mouse) {
        Some(LobbyAction::Host)
    } else if lobby_join().contains(mouse) {
        Some(LobbyAction::Join(lobby.address.clone()))
    } else {
        None
    }
}

pub fn handle_input(game: &mut Game) {
    let mouse: Vec2 = mouse_position().into();
    let click = is_mouse_button_pressed(MouseButton::Left);

    if game.menu.is_some() {
        handle_menu(game, true);
        return;
    }
    if click && settings_button().contains(mouse) {
        game.menu = Some(Menu::Settings);
        return;
    }
    if (click && stats_button().contains(mouse)) || is_key_pressed(KeyCode::D) {
        game.show_stats = !game.show_stats;
        if click {
            return;
        }
    }

    if game.state != GameState::Playing {
        let retry = game.state == GameState::GameOver
            && (is_key_pressed(KeyCode::R) || (click && retry_button().contains(mouse)));
        let new_game =
            is_key_pressed(KeyCode::Enter) || (click && new_game_button().contains(mouse));
        if retry {
            game.act(Action::RetryWave);
        } else if new_game && !game.is_client {
            game.act(Action::Restart);
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
        KeyCode::Key6,
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
        game.act(Action::StartWave);
    }
    if is_key_pressed(KeyCode::A) {
        game.act(Action::ToggleAuto);
    }
    if is_key_pressed(KeyCode::F) {
        cycle_speed(game);
    }
    if is_key_pressed(KeyCode::P) {
        game.act(Action::SetPaused(!game.paused));
    }
    let selected = game.selected_tower().map(|t| t.id);
    if let Some(id) = selected {
        if is_key_pressed(KeyCode::Tab) {
            game.act(Action::CycleTarget { tower: id });
        }
        for (path, key) in PATH_KEYS.into_iter().enumerate() {
            if is_key_pressed(key) {
                game.act(Action::Upgrade { tower: id, path });
            }
        }
        if is_key_pressed(KeyCode::S) || is_key_pressed(KeyCode::Delete) {
            sell(game, id);
        }
    }

    if !click {
        return;
    }

    if in_map(mouse, 0.0) {
        if let Some(kind) = game.build_choice {
            if game.can_place(mouse) && game.can_afford(game.tower_cost(kind)) {
                game.act(Action::Place { kind, pos: mouse });
                // Hold shift to keep placing the same tower.
                if !shift() {
                    game.build_choice = None;
                }
            }
        } else {
            game.selected = game.tower_at(mouse).map(|t| t.id);
        }
        return;
    }

    if wave_button().contains(mouse) {
        game.act(Action::StartWave);
    } else if auto_button().contains(mouse) {
        game.act(Action::ToggleAuto);
    } else if pause_button().contains(mouse) {
        game.act(Action::SetPaused(!game.paused));
    } else if speed_button().contains(mouse) {
        cycle_speed(game);
    } else if let Some(id) = selected {
        if close_button().contains(mouse) {
            game.selected = None;
        } else if sell_button().contains(mouse) {
            sell(game, id);
        } else if target_button().contains(mouse) {
            game.act(Action::CycleTarget { tower: id });
        }
        for path in 0..PATHS {
            if path_card(path).contains(mouse) {
                game.act(Action::Upgrade { tower: id, path });
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

fn sell(game: &mut Game, id: u32) {
    game.act(Action::Sell { tower: id });
    game.selected = None;
}

fn handle_sandbox_click(game: &mut Game, mouse: Vec2) {
    let count = if shift() { 5 } else { 1 };
    for (i, kind) in SANDBOX_SPAWNS.into_iter().enumerate() {
        if sandbox_spawn_button(i).contains(mouse) {
            game.act(Action::SandboxSpawn { kind, count });
        }
    }
    let step = count as i64;
    let wave = game.wave as i64;
    if sandbox_wave_down().contains(mouse) {
        game.act(Action::SandboxSetWave((wave - step).max(0) as u32));
    } else if sandbox_wave_up().contains(mouse) {
        game.act(Action::SandboxSetWave((wave + step) as u32));
    } else if sandbox_clear_button().contains(mouse) {
        game.act(Action::SandboxClear);
    }
}

/// Settings menu and tower guide input. `in_game` is false on the main menu.
pub fn handle_menu(game: &mut Game, in_game: bool) {
    let mouse: Vec2 = mouse_position().into();
    let click = is_mouse_button_pressed(MouseButton::Left);
    match game.menu {
        Some(Menu::Settings) => {
            let items = settings_items(in_game, game.is_client);
            let n = items.len();
            if is_key_pressed(KeyCode::Escape) {
                game.menu = None;
                return;
            }
            // Volume sliders follow the mouse while held.
            if !is_mouse_button_down(MouseButton::Left) {
                game.dragging = None;
            }
            if click {
                game.dragging =
                    (0..n).find(|&i| items[i].is_slider() && settings_row(n, i).contains(mouse));
            }
            if let Some(i) = game.dragging {
                let track = slider_track(settings_row(n, i));
                let value = ((mouse.x - track.x) / track.w).clamp(0.0, 1.0);
                match items[i] {
                    SettingsItem::TowerVolume => game.tower_volume = value,
                    SettingsItem::GameVolume => game.game_volume = value,
                    _ => {}
                }
                return;
            }
            if !click {
                return;
            }
            let hit = (0..n).find(|&i| settings_row(n, i).contains(mouse));
            match hit.map(|i| items[i]) {
                Some(SettingsItem::Theme) => game.night = !game.night,
                Some(SettingsItem::Guide) => game.menu = Some(Menu::Guide(0)),
                Some(SettingsItem::Restart) => {
                    game.menu = None;
                    game.act(Action::Restart);
                }
                Some(SettingsItem::MainMenu) => {
                    game.menu = None;
                    game.request = Some(Request::MainMenu);
                }
                Some(SettingsItem::Close) => game.menu = None,
                Some(_) => {}
                None if !settings_panel(n).contains(mouse) => game.menu = None,
                None => {}
            }
        }
        Some(Menu::Guide(i)) => {
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
        Some(Menu::Skills) => game.menu = None,
        None => {}
    }
}

fn handle_skills(game: &mut Game, profile: &mut crate::profile::Profile) {
    if is_key_pressed(KeyCode::Escape) {
        game.menu = None;
        return;
    }
    if !is_mouse_button_pressed(MouseButton::Left) {
        return;
    }
    let mouse: Vec2 = mouse_position().into();
    if skills_close().contains(mouse) {
        game.menu = None;
    } else if skills_reset().contains(mouse) {
        profile.reset_skills();
    } else if let Some(skill) = crate::profile::SKILLS
        .iter()
        .find(|s| skill_node(s.id).contains(mouse))
        && profile.unlock(skill.id)
    {
        game.sounds.push(crate::audio::Sfx::Upgrade);
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
    let next = match game.speed {
        1 => 2,
        2 => 3,
        _ => 1,
    };
    game.act(Action::SetSpeed(next));
}
