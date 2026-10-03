use macroquad::prelude::*;

use crate::enemy::EnemyKind;
use crate::game::{EffectShape, Game, GameState, Menu, Shot};
use crate::map::*;
use crate::tower::{PATHS, Stats, TIERS, Tower, TowerKind};
use crate::ui::{self, Lobby, SettingsItem};
use crate::wave::MAX_WAVES;

/// Colors for one of the two visual themes.
struct Theme {
    night: bool,
    grass: Color,
    decor_light: Color,
    decor_dark: Color,
    path_fill: Color,
    path_edge: Color,
    floor_a: Color,
    floor_b: Color,
    mortar: Color,
    panel: Color,
    panel_light: Color,
    panel_hover: Color,
    border: Color,
    text: Color,
    text_dim: Color,
    gold: Color,
    bad: Color,
}

const DAY: Theme = Theme {
    night: false,
    grass: Color::new(0.40, 0.65, 0.32, 1.0),
    decor_light: Color::new(0.46, 0.72, 0.37, 1.0),
    decor_dark: Color::new(0.34, 0.57, 0.28, 1.0),
    path_fill: Color::new(0.86, 0.75, 0.54, 1.0),
    path_edge: Color::new(0.66, 0.53, 0.35, 1.0),
    floor_a: Color::new(0.60, 0.58, 0.55, 1.0),
    floor_b: Color::new(0.55, 0.53, 0.50, 1.0),
    mortar: Color::new(0.40, 0.38, 0.36, 1.0),
    panel: Color::new(0.93, 0.91, 0.86, 1.0),
    panel_light: Color::new(0.85, 0.83, 0.78, 1.0),
    panel_hover: Color::new(0.78, 0.81, 0.88, 1.0),
    border: Color::new(0.0, 0.0, 0.0, 0.25),
    text: Color::new(0.12, 0.12, 0.15, 1.0),
    text_dim: Color::new(0.38, 0.38, 0.42, 1.0),
    gold: Color::new(0.68, 0.46, 0.0, 1.0),
    bad: Color::new(0.75, 0.18, 0.18, 1.0),
};

const NIGHT: Theme = Theme {
    night: true,
    grass: Color::new(0.11, 0.22, 0.19, 1.0),
    decor_light: Color::new(0.14, 0.27, 0.23, 1.0),
    decor_dark: Color::new(0.08, 0.17, 0.15, 1.0),
    path_fill: Color::new(0.38, 0.34, 0.31, 1.0),
    path_edge: Color::new(0.23, 0.20, 0.19, 1.0),
    floor_a: Color::new(0.22, 0.21, 0.24, 1.0),
    floor_b: Color::new(0.19, 0.18, 0.21, 1.0),
    mortar: Color::new(0.11, 0.10, 0.12, 1.0),
    panel: Color::new(0.11, 0.12, 0.16, 1.0),
    panel_light: Color::new(0.18, 0.20, 0.26, 1.0),
    panel_hover: Color::new(0.25, 0.28, 0.36, 1.0),
    border: Color::new(0.0, 0.0, 0.0, 0.5),
    text: Color::new(0.92, 0.92, 0.95, 1.0),
    text_dim: Color::new(0.62, 0.64, 0.70, 1.0),
    gold: Color::new(1.0, 0.80, 0.25, 1.0),
    bad: Color::new(0.90, 0.42, 0.42, 1.0),
};

const BLUE: Color = Color::new(0.31, 0.38, 0.55, 1.0);
const GREEN_BTN: Color = Color::new(0.24, 0.59, 0.31, 1.0);
const RED_BTN: Color = Color::new(0.67, 0.27, 0.24, 1.0);
const GREY_BTN: Color = Color::new(0.40, 0.42, 0.48, 1.0);
const ORANGE_BTN: Color = Color::new(0.75, 0.50, 0.10, 1.0);
const RUG: Color = Color::new(0.62, 0.12, 0.14, 1.0);
const RUG_GOLD: Color = Color::new(0.85, 0.66, 0.22, 1.0);

fn theme(game: &Game) -> &'static Theme {
    if game.night { &NIGHT } else { &DAY }
}

pub fn draw(game: &Game) {
    let t = theme(game);
    clear_background(t.panel);
    draw_map(&game.map, game.time, game.since_base_hit, t);
    draw_ranges(game);
    for (i, tower) in game.towers.iter().enumerate() {
        draw_tower(tower, tower.pos, game.time, 1.0, game.is_buffed(i), t);
    }
    draw_enemies(game);
    draw_projectiles(game);
    draw_effects(game);
    draw_placement_preview(game, t);
    draw_party_badge(game);
    draw_sidebar(game, t);
    draw_top_bar(game, t);
    draw_message(game);
    draw_overlay(game, t);
    draw_menus(game, true, t);
}

fn draw_menus(game: &Game, in_game: bool, t: &Theme) {
    match game.menu {
        Some(Menu::Settings) => draw_settings(game, in_game, t),
        Some(Menu::Guide(i)) => draw_guide(i, t),
        None => {}
    }
}

fn with_alpha(c: Color, a: f32) -> Color {
    Color::new(c.r, c.g, c.b, a)
}

fn darken(c: Color, f: f32) -> Color {
    Color::new(c.r * f, c.g * f, c.b * f, c.a)
}

fn mouse() -> Vec2 {
    mouse_position().into()
}

/// Cheap hash for per-position variety.
fn hash(x: f32, y: f32) -> f32 {
    let v = (x * 12.9898 + y * 78.233).sin() * 43_758.547;
    v - v.floor()
}

fn text_centered(text: &str, center: Vec2, size: u16, color: Color) {
    let m = measure_text(text, None, size, 1.0);
    draw_text(
        text,
        center.x - m.width / 2.0,
        center.y + m.offset_y / 2.0,
        size as f32,
        color,
    );
}

fn text_right(text: &str, right: f32, y: f32, size: u16, color: Color) {
    let w = measure_text(text, None, size, 1.0).width;
    draw_text(text, right - w, y, size as f32, color);
}

fn panel_rect(r: Rect, fill: Color, border: Color) {
    draw_rectangle(r.x, r.y, r.w, r.h, fill);
    draw_rectangle_lines(r.x, r.y, r.w, r.h, 2.0, border);
}

fn button(rect: Rect, label: &str, enabled: bool, highlight: Color, t: &Theme) {
    let hover = enabled && rect.contains(mouse());
    let fill = match (enabled, hover) {
        (false, _) => t.panel_light,
        (true, true) => highlight,
        (true, false) => darken(highlight, 0.85),
    };
    panel_rect(rect, fill, t.border);
    let color = if enabled { WHITE } else { t.text_dim };
    text_centered(label, rect.center(), 22, color);
}

// ---------------------------------------------------------------- maps

fn draw_map(map: &Map, time: f32, since_base_hit: f32, t: &Theme) {
    match map.theme() {
        MapTheme::Meadow => draw_meadow(map, time, t),
        MapTheme::Castle => draw_castle_hall(map, time, t),
    }
    let portal = map.portal;
    let pulse = (time * 3.0).sin() * 3.0;
    draw_circle(portal.x, portal.y, 16.0, with_alpha(PURPLE, 0.35));
    draw_circle_lines(
        portal.x,
        portal.y,
        18.0 + pulse,
        3.0,
        with_alpha(PURPLE, 0.9),
    );

    for prop in &map.props {
        draw_prop(prop, time, t);
    }
    match map.theme() {
        MapTheme::Meadow => draw_castle_base(map.base, time, since_base_hit),
        MapTheme::Castle => draw_throne(map.base, time, since_base_hit),
    }
}

fn draw_meadow(map: &Map, time: f32, t: &Theme) {
    draw_rectangle(MAP_X, MAP_Y, MAP_W, MAP_H, t.grass);
    for &(p, r, shade) in &map.decor {
        let c = if shade > 0.5 {
            t.decor_light
        } else {
            t.decor_dark
        };
        draw_circle(p.x, p.y, r, c);
        // Some tufts get a little flower.
        if shade > 0.86 {
            let petal = match ((shade * 100.0) as u32) % 3 {
                0 => Color::new(1.0, 1.0, 1.0, 0.9),
                1 => Color::new(1.0, 0.85, 0.3, 0.9),
                _ => Color::new(0.95, 0.55, 0.75, 0.9),
            };
            for k in 0..5 {
                let a = k as f32 / 5.0 * std::f32::consts::TAU;
                draw_circle(p.x + a.cos() * 2.5, p.y + a.sin() * 2.5, 1.8, petal);
            }
            draw_circle(p.x, p.y, 1.3, Color::new(0.9, 0.7, 0.1, 1.0));
        }
    }

    // Dirt track with square joints and scattered pebbles.
    let wps = &map.waypoints;
    for (width, color) in [(PATH_WIDTH + 8.0, t.path_edge), (PATH_WIDTH, t.path_fill)] {
        stroke_path(wps, width, color);
    }
    for w in wps.windows(2) {
        let (a, b) = (w[0], w[1]);
        let len = a.distance(b);
        let dir = (b - a) / len;
        let side = vec2(-dir.y, dir.x);
        let mut d = 10.0;
        while d < len {
            let p = a + dir * d;
            let h = hash(p.x, p.y);
            let q = p + side * (h - 0.5) * (PATH_WIDTH - 10.0);
            draw_circle(q.x, q.y, 1.5 + h * 1.5, with_alpha(t.path_edge, 0.7));
            d += 17.0 + h * 14.0;
        }
    }

    if t.night {
        // Fireflies drifting over the grass.
        for &(p, _, shade) in map.decor.iter().filter(|d| d.2 > 0.9) {
            let phase = time * 0.8 + shade * 40.0;
            let pos = p + vec2(phase.sin() * 10.0, (phase * 1.3).cos() * 8.0);
            let glow = 0.5 + 0.5 * (time * 3.0 + shade * 20.0).sin();
            draw_circle(pos.x, pos.y, 5.0, Color::new(1.0, 0.95, 0.5, 0.12 * glow));
            draw_circle(pos.x, pos.y, 1.6, Color::new(1.0, 0.95, 0.6, 0.8 * glow));
        }
    }
}

fn stroke_path(wps: &[Vec2], width: f32, color: Color) {
    for pair in wps.windows(2) {
        draw_line(pair[0].x, pair[0].y, pair[1].x, pair[1].y, width, color);
    }
    for p in wps {
        draw_rectangle(p.x - width / 2.0, p.y - width / 2.0, width, width, color);
    }
}

fn draw_castle_hall(map: &Map, time: f32, t: &Theme) {
    // Flagstone floor.
    let tile = 48.0;
    let mut y = MAP_Y;
    while y < MAP_Y + MAP_H {
        let mut x = MAP_X;
        while x < MAP_X + MAP_W {
            let h = hash(x, y);
            let base = if h > 0.5 { t.floor_a } else { t.floor_b };
            let c = darken(base, 0.94 + h * 0.08);
            draw_rectangle(x, y, tile, tile, c);
            draw_rectangle_lines(x, y, tile, tile, 2.0, t.mortar);
            if h > 0.85 {
                // A crack.
                draw_line(x + 10.0, y + 14.0, x + 22.0, y + 26.0, 1.0, t.mortar);
                draw_line(x + 22.0, y + 26.0, x + 20.0, y + 36.0, 1.0, t.mortar);
            }
            x += tile;
        }
        y += tile;
    }

    // Back wall with hanging banners.
    draw_rectangle(MAP_X, MAP_Y, MAP_W, 34.0, darken(t.floor_b, 0.7));
    for i in 0..((MAP_W / 48.0) as usize) {
        let x = MAP_X + i as f32 * 48.0 + (i % 2) as f32 * 24.0;
        draw_line(x, MAP_Y, x, MAP_Y + 34.0, 1.5, t.mortar);
    }
    draw_line(
        MAP_X,
        MAP_Y + 17.0,
        MAP_X + MAP_W,
        MAP_Y + 17.0,
        1.5,
        t.mortar,
    );
    draw_line(
        MAP_X,
        MAP_Y + 34.0,
        MAP_X + MAP_W,
        MAP_Y + 34.0,
        3.0,
        t.mortar,
    );
    let mut x = MAP_X + 120.0;
    while x < MAP_X + MAP_W - 40.0 {
        let sway = (time * 1.2 + x).sin() * 1.5;
        draw_rectangle(x - 12.0, MAP_Y + 4.0, 24.0, 44.0, RUG);
        draw_triangle(
            vec2(x - 12.0, MAP_Y + 48.0),
            vec2(x + 12.0, MAP_Y + 48.0),
            vec2(x + sway, MAP_Y + 58.0),
            RUG,
        );
        draw_rectangle_lines(x - 12.0, MAP_Y + 4.0, 24.0, 44.0, 2.0, RUG_GOLD);
        draw_circle(x, MAP_Y + 22.0, 6.0, RUG_GOLD);
        draw_circle(x, MAP_Y + 22.0, 3.0, RUG);
        x += 240.0;
    }

    // The track is a royal rug with gold trim and a diamond pattern.
    let wps = &map.waypoints;
    stroke_path(wps, PATH_WIDTH + 6.0, RUG_GOLD);
    stroke_path(wps, PATH_WIDTH - 2.0, RUG);
    stroke_path(wps, PATH_WIDTH - 12.0, RUG_GOLD);
    stroke_path(wps, PATH_WIDTH - 16.0, darken(RUG, 0.92));
    for w in wps.windows(2) {
        let (a, b) = (w[0], w[1]);
        let len = a.distance(b);
        let dir = (b - a) / len;
        let mut d = 16.0;
        while d < len - 8.0 {
            let p = a + dir * d;
            draw_poly(p.x, p.y, 4, 4.5, 0.0, RUG_GOLD);
            d += 32.0;
        }
    }

    if t.night {
        // Darken the hall away from the torches.
        draw_rectangle(MAP_X, MAP_Y, MAP_W, MAP_H, Color::new(0.0, 0.0, 0.05, 0.25));
    }
}

fn draw_prop(prop: &Prop, time: f32, t: &Theme) {
    let p = prop.pos;
    match prop.kind {
        PropKind::Tree => {
            let s = 0.85 + prop.seed * 0.35;
            draw_circle(p.x + 4.0, p.y + 6.0, 20.0 * s, with_alpha(BLACK, 0.22));
            draw_rectangle(
                p.x - 3.0,
                p.y + 4.0,
                6.0,
                12.0 * s,
                Color::new(0.42, 0.28, 0.16, 1.0),
            );
            let (dark, mid, light) = if t.night {
                (
                    Color::new(0.06, 0.18, 0.10, 1.0),
                    Color::new(0.09, 0.24, 0.13, 1.0),
                    Color::new(0.13, 0.30, 0.17, 1.0),
                )
            } else {
                (
                    Color::new(0.16, 0.42, 0.20, 1.0),
                    Color::new(0.22, 0.52, 0.25, 1.0),
                    Color::new(0.32, 0.63, 0.30, 1.0),
                )
            };
            draw_circle(p.x, p.y, 18.0 * s, dark);
            draw_circle(p.x - 5.0 * s, p.y - 4.0 * s, 12.0 * s, mid);
            draw_circle(p.x + 6.0 * s, p.y - 2.0 * s, 10.0 * s, mid);
            draw_circle(p.x - 4.0 * s, p.y - 8.0 * s, 6.0 * s, light);
        }
        PropKind::Rock => {
            let rot = prop.seed * 90.0;
            draw_poly(p.x + 2.0, p.y + 3.0, 6, 10.0, rot, with_alpha(BLACK, 0.2));
            draw_poly(p.x, p.y, 6, 10.0, rot, Color::new(0.52, 0.52, 0.55, 1.0));
            draw_poly(
                p.x - 2.0,
                p.y - 3.0,
                5,
                5.0,
                rot,
                Color::new(0.68, 0.68, 0.70, 1.0),
            );
        }
        PropKind::Pillar => {
            draw_circle(p.x + 4.0, p.y + 5.0, 16.0, with_alpha(BLACK, 0.3));
            draw_circle(p.x, p.y, 16.0, darken(t.floor_a, 0.85));
            draw_circle(p.x, p.y, 13.0, darken(t.floor_a, 1.15));
            draw_circle_lines(p.x, p.y, 16.0, 2.0, t.mortar);
            draw_circle_lines(p.x, p.y, 8.0, 1.0, with_alpha(t.mortar, 0.6));
        }
        PropKind::Torch => {
            let flicker = (time * 13.0 + prop.seed * 50.0).sin() * 0.5
                + (time * 7.3 + prop.seed * 20.0).sin() * 0.5;
            let glow = if t.night { 0.22 } else { 0.10 };
            draw_circle(
                p.x,
                p.y - 4.0,
                46.0 + flicker * 3.0,
                Color::new(1.0, 0.6, 0.2, glow * 0.6),
            );
            draw_circle(
                p.x,
                p.y - 4.0,
                26.0 + flicker * 2.0,
                Color::new(1.0, 0.7, 0.3, glow),
            );
            // Stand and bowl.
            draw_rectangle(
                p.x - 2.0,
                p.y - 2.0,
                4.0,
                12.0,
                Color::new(0.25, 0.2, 0.18, 1.0),
            );
            draw_rectangle(
                p.x - 6.0,
                p.y - 4.0,
                12.0,
                5.0,
                Color::new(0.35, 0.3, 0.26, 1.0),
            );
            // Flame.
            let h = 10.0 + flicker * 2.5;
            draw_triangle(
                vec2(p.x - 5.0, p.y - 4.0),
                vec2(p.x + 5.0, p.y - 4.0),
                vec2(p.x + flicker * 1.5, p.y - 4.0 - h),
                Color::new(1.0, 0.45, 0.1, 1.0),
            );
            draw_triangle(
                vec2(p.x - 2.5, p.y - 4.0),
                vec2(p.x + 2.5, p.y - 4.0),
                vec2(p.x + flicker, p.y - 4.0 - h * 0.6),
                Color::new(1.0, 0.85, 0.3, 1.0),
            );
        }
    }
}

/// Shake and flash red briefly when an enemy gets through.
fn base_hit_shake(time: f32, since_base_hit: f32) -> (Vec2, f32) {
    let hit = (1.0 - since_base_hit / 0.4).max(0.0);
    let shake = vec2((time * 60.0).sin(), (time * 47.0).cos()) * 3.0 * hit;
    (shake, hit)
}

fn draw_castle_base(base: Vec2, time: f32, since_base_hit: f32) {
    let (shake, hit) = base_hit_shake(time, since_base_hit);
    let base = base + shake;
    let stone = Color::new(0.35 + 0.5 * hit, 0.35 - 0.15 * hit, 0.43 - 0.2 * hit, 1.0);
    draw_rectangle(base.x - 20.0, base.y - 18.0, 40.0, 36.0, stone);
    for i in 0..3 {
        let x = base.x - 20.0 + i as f32 * 15.0;
        draw_rectangle(x, base.y - 26.0, 10.0, 10.0, stone);
    }
    draw_rectangle_lines(
        base.x - 20.0,
        base.y - 18.0,
        40.0,
        36.0,
        2.0,
        with_alpha(BLACK, 0.4),
    );
    for row in 0..3 {
        let y = base.y - 8.0 + row as f32 * 9.0;
        draw_line(
            base.x - 20.0,
            y,
            base.x + 20.0,
            y,
            1.0,
            with_alpha(BLACK, 0.25),
        );
    }
    draw_rectangle(
        base.x - 6.0,
        base.y + 2.0,
        12.0,
        16.0,
        Color::from_rgba(50, 40, 40, 255),
    );
    draw_circle(base.x, base.y + 2.0, 6.0, Color::from_rgba(50, 40, 40, 255));
    draw_line(base.x, base.y - 26.0, base.x, base.y - 44.0, 2.0, DARKGRAY);
    let wave = (time * 4.0).sin() * 2.0;
    draw_triangle(
        vec2(base.x, base.y - 44.0),
        vec2(base.x + 14.0, base.y - 39.0 + wave),
        vec2(base.x, base.y - 34.0),
        RED,
    );
}

fn draw_throne(base: Vec2, time: f32, since_base_hit: f32) {
    let (shake, hit) = base_hit_shake(time, since_base_hit);
    let p = base + shake;
    let gold = Color::new(0.85 + 0.15 * hit, 0.66 - 0.3 * hit, 0.22 - 0.1 * hit, 1.0);
    // Dais.
    draw_rectangle(
        p.x - 26.0,
        p.y + 10.0,
        52.0,
        12.0,
        Color::new(0.45, 0.42, 0.40, 1.0),
    );
    draw_rectangle_lines(
        p.x - 26.0,
        p.y + 10.0,
        52.0,
        12.0,
        2.0,
        with_alpha(BLACK, 0.4),
    );
    // Back, arms, seat.
    draw_rectangle(p.x - 14.0, p.y - 30.0, 28.0, 40.0, gold);
    draw_triangle(
        vec2(p.x - 14.0, p.y - 30.0),
        vec2(p.x + 14.0, p.y - 30.0),
        vec2(p.x, p.y - 42.0),
        gold,
    );
    draw_rectangle(p.x - 10.0, p.y - 24.0, 20.0, 30.0, RUG);
    draw_rectangle(p.x - 20.0, p.y - 4.0, 8.0, 16.0, gold);
    draw_rectangle(p.x + 12.0, p.y - 4.0, 8.0, 16.0, gold);
    draw_rectangle(p.x - 12.0, p.y + 2.0, 24.0, 8.0, darken(RUG, 0.8));
    draw_circle(p.x, p.y - 36.0, 3.0, Color::new(0.3, 0.6, 1.0, 1.0));
    draw_circle(p.x - 8.0, p.y - 28.0, 2.0, Color::new(0.9, 0.2, 0.3, 1.0));
    draw_circle(p.x + 8.0, p.y - 28.0, 2.0, Color::new(0.9, 0.2, 0.3, 1.0));
    draw_rectangle_lines(
        p.x - 14.0,
        p.y - 30.0,
        28.0,
        40.0,
        2.0,
        with_alpha(BLACK, 0.35),
    );
}

// ---------------------------------------------------------------- towers

fn draw_range_circle(pos: Vec2, radius: f32, tint: Color) {
    if radius > 0.0 && radius.is_finite() {
        draw_circle(pos.x, pos.y, radius, with_alpha(tint, 0.10));
        draw_circle_lines(pos.x, pos.y, radius, 2.0, with_alpha(tint, 0.55));
    }
}

fn draw_tower_ranges(pos: Vec2, stats: &Stats) {
    draw_range_circle(pos, stats.range, WHITE);
    draw_range_circle(pos, stats.buff_radius, GREEN);
}

fn draw_ranges(game: &Game) {
    let Some(id) = game.selected else {
        return;
    };
    if let Some(i) = game.tower_index(id) {
        draw_tower_ranges(game.towers[i].pos, &game.effective_stats(i));
    }
}

fn draw_placement_preview(game: &Game, t: &Theme) {
    let m = mouse();
    if game.menu.is_some() || !in_map(m, 0.0) {
        return;
    }
    let Some(kind) = game.build_choice else {
        if let Some(tw) = game.tower_at(m) {
            draw_circle_lines(
                tw.pos.x,
                tw.pos.y,
                TOWER_RADIUS + 4.0,
                2.0,
                with_alpha(WHITE, 0.7),
            );
        }
        return;
    };
    let ok = game.can_place(m) && game.can_afford(kind.cost());
    let tint = if ok { WHITE } else { RED };
    draw_range_circle(m, kind.stats([0; PATHS]).range, tint);
    draw_tower(&Tower::new(0, kind, m), m, game.time, 0.6, false, t);
    if !ok {
        draw_circle_lines(m.x, m.y, TOWER_RADIUS + 2.0, 3.0, RED);
    }
    let hint = if kind == TowerKind::Sniper {
        "Global range  -  Right-click / Esc to cancel"
    } else {
        "Right-click / Esc to cancel"
    };
    let w = measure_text(hint, None, 16, 1.0).width;
    let x = (m.x - w / 2.0).clamp(MAP_X + 4.0, MAP_X + MAP_W - w - 4.0);
    let y = (m.y + TOWER_RADIUS + 22.0).min(MAP_Y + MAP_H - 6.0);
    draw_rectangle(x - 4.0, y - 13.0, w + 8.0, 18.0, with_alpha(BLACK, 0.55));
    draw_text(hint, x, y, 16.0, WHITE);
}

/// Ring color showing a tower's highest upgrade tier.
fn tier_color(tier: u8, time: f32) -> Option<Color> {
    match tier {
        1 => Some(Color::new(0.80, 0.50, 0.25, 1.0)),
        2 => Some(Color::new(0.80, 0.82, 0.86, 1.0)),
        3 => Some(Color::new(1.00, 0.80, 0.20, 1.0)),
        4 => {
            let pulse = 0.75 + 0.25 * (time * 4.0).sin();
            Some(Color::new(0.75 * pulse, 0.45 * pulse, 1.0, 1.0))
        }
        _ => None,
    }
}

fn draw_tower(tower: &Tower, c: Vec2, time: f32, alpha: f32, buffed: bool, t: &Theme) {
    let a = |col: Color| with_alpha(col, col.a * alpha);
    let color = a(tower.kind.color());
    let dark = a(darken(tower.kind.color(), 0.45));
    let dir = vec2(tower.angle.cos(), tower.angle.sin());
    let side = vec2(-dir.y, dir.x);
    let recoil = (1.0 - tower.since_shot / 0.12).max(0.0) * 4.0;

    if t.night {
        // Lantern glow so towers stand out in the dark.
        draw_circle(
            c.x,
            c.y,
            TOWER_RADIUS + 14.0,
            Color::new(1.0, 0.85, 0.5, 0.07 * alpha),
        );
    }
    draw_circle(
        c.x + 3.0,
        c.y + 4.0,
        TOWER_RADIUS,
        a(Color::new(0.0, 0.0, 0.0, 0.25)),
    );

    match tower.kind {
        TowerKind::Farm => draw_farm(tower, c, time, alpha),
        TowerKind::MinuteMan => draw_minuteman(tower, c, alpha),
        TowerKind::Arrow => {
            // Wooden platform with a crossbow on top.
            draw_poly(
                c.x,
                c.y,
                8,
                TOWER_RADIUS,
                22.5,
                a(Color::new(0.50, 0.34, 0.20, 1.0)),
            );
            draw_poly_lines(
                c.x,
                c.y,
                8,
                TOWER_RADIUS,
                22.5,
                2.0,
                a(Color::new(0.30, 0.20, 0.12, 1.0)),
            );
            for k in [-1.0, 0.0, 1.0] {
                let y = c.y + k * 7.0;
                draw_line(
                    c.x - 14.0,
                    y,
                    c.x + 14.0,
                    y,
                    1.0,
                    a(Color::new(0.35, 0.23, 0.14, 1.0)),
                );
            }
            let back = c - dir * (7.0 - recoil * 0.5);
            let front = c + dir * (14.0 - recoil);
            draw_line(back.x, back.y, front.x, front.y, 5.0, dark);
            let limb_root = c + dir * 9.0;
            let tip_l = c + dir * 3.0 + side * 13.0;
            let tip_r = c + dir * 3.0 - side * 13.0;
            draw_line(limb_root.x, limb_root.y, tip_l.x, tip_l.y, 3.0, color);
            draw_line(limb_root.x, limb_root.y, tip_r.x, tip_r.y, 3.0, color);
            let nock = c - dir * (3.0 - recoil);
            let string = a(Color::new(0.95, 0.92, 0.85, 1.0));
            draw_line(tip_l.x, tip_l.y, nock.x, nock.y, 1.0, string);
            draw_line(tip_r.x, tip_r.y, nock.x, nock.y, 1.0, string);
            if tower.since_shot > 0.15 {
                let bolt = c + dir * 17.0;
                draw_line(nock.x, nock.y, bolt.x, bolt.y, 2.0, string);
            }
            draw_circle(c.x, c.y, 4.0, color);
        }
        TowerKind::Cannon => {
            draw_circle(c.x, c.y, TOWER_RADIUS, a(Color::new(0.30, 0.30, 0.34, 1.0)));
            draw_circle_lines(
                c.x,
                c.y,
                TOWER_RADIUS,
                2.0,
                a(Color::new(0.15, 0.15, 0.18, 1.0)),
            );
            for k in 0..6 {
                let ang = k as f32 / 6.0 * std::f32::consts::TAU;
                draw_circle(
                    c.x + ang.cos() * 14.5,
                    c.y + ang.sin() * 14.5,
                    1.6,
                    a(Color::new(0.6, 0.6, 0.65, 1.0)),
                );
            }
            let tip = c + dir * (18.0 - recoil);
            let barrel = a(Color::new(0.20, 0.20, 0.22, 1.0));
            draw_line(c.x, c.y, tip.x, tip.y, 11.0, barrel);
            draw_circle(tip.x, tip.y, 6.5, barrel);
            draw_circle(tip.x, tip.y, 3.5, a(BLACK));
            let band = c + dir * (9.0 - recoil * 0.5);
            draw_line(
                band.x - side.x * 6.0,
                band.y - side.y * 6.0,
                band.x + side.x * 6.0,
                band.y + side.y * 6.0,
                2.0,
                a(Color::new(0.65, 0.55, 0.3, 1.0)),
            );
            draw_circle(c.x, c.y, 10.0, color);
            draw_circle_lines(c.x, c.y, 10.0, 2.0, dark);
            if tower.since_shot < 0.08 {
                let flash = c + dir * 26.0;
                draw_circle(flash.x, flash.y, 7.0, with_alpha(ORANGE, 0.8 * alpha));
                draw_circle(flash.x, flash.y, 4.0, with_alpha(YELLOW, 0.9 * alpha));
            }
        }
        TowerKind::Frost => {
            draw_circle(c.x, c.y, TOWER_RADIUS, a(Color::new(0.80, 0.90, 0.97, 1.0)));
            draw_circle_lines(
                c.x,
                c.y,
                TOWER_RADIUS,
                2.0,
                a(Color::new(0.45, 0.65, 0.80, 1.0)),
            );
            let pulse = (time * 4.0 + c.x).sin();
            let r = 10.0 + pulse;
            draw_poly(c.x, c.y, 6, r, time * 40.0, color);
            draw_poly_lines(c.x, c.y, 6, r, time * 40.0, 2.0, dark);
            draw_circle(c.x, c.y, 3.5, a(WHITE));
            // Shards orbiting the crystal.
            for k in 0..3 {
                let ang = time * 1.5 + k as f32 / 3.0 * std::f32::consts::TAU;
                let s = c + vec2(ang.cos(), ang.sin()) * 14.0;
                draw_poly(
                    s.x,
                    s.y,
                    4,
                    3.0,
                    ang.to_degrees(),
                    a(Color::new(0.85, 0.97, 1.0, 1.0)),
                );
            }
            if tower.since_shot < 0.15 {
                let k = tower.since_shot / 0.15;
                draw_circle_lines(
                    c.x,
                    c.y,
                    TOWER_RADIUS + k * 8.0,
                    2.0,
                    with_alpha(SKYBLUE, (1.0 - k) * alpha),
                );
            }
        }
        TowerKind::Sniper => {
            draw_poly(
                c.x,
                c.y,
                6,
                TOWER_RADIUS,
                0.0,
                a(Color::new(0.35, 0.33, 0.40, 1.0)),
            );
            draw_poly_lines(
                c.x,
                c.y,
                6,
                TOWER_RADIUS,
                0.0,
                2.0,
                a(Color::new(0.18, 0.17, 0.22, 1.0)),
            );
            let tip = c + dir * (28.0 - recoil);
            draw_line(c.x, c.y, tip.x, tip.y, 6.0, dark);
            draw_line(
                c.x,
                c.y,
                tip.x,
                tip.y,
                3.5,
                a(Color::new(0.25, 0.25, 0.28, 1.0)),
            );
            draw_line(
                tip.x - side.x * 3.0,
                tip.y - side.y * 3.0,
                tip.x + side.x * 3.0,
                tip.y + side.y * 3.0,
                3.0,
                a(Color::new(0.15, 0.15, 0.17, 1.0)),
            );
            // Scope.
            let s0 = c + dir * 3.0 + side * 5.0;
            let s1 = c + dir * 13.0 + side * 5.0;
            draw_line(
                s0.x,
                s0.y,
                s1.x,
                s1.y,
                3.5,
                a(Color::new(0.1, 0.1, 0.12, 1.0)),
            );
            draw_circle(s1.x, s1.y, 1.8, a(Color::new(0.5, 0.8, 1.0, 1.0)));
            draw_circle(c.x, c.y, 9.0, color);
            draw_circle_lines(c.x, c.y, 9.0, 2.0, dark);
            if tower.since_shot < 0.06 {
                let flash = c + dir * 32.0;
                draw_circle(flash.x, flash.y, 5.0, with_alpha(YELLOW, 0.8 * alpha));
            }
        }
    }

    let top = tower.tiers.iter().copied().max().unwrap_or(0);
    if let Some(ring) = tier_color(top, time) {
        draw_circle_lines(c.x, c.y, TOWER_RADIUS + 2.0, 2.5, with_alpha(ring, alpha));
    }
    if buffed {
        let b = c + vec2(14.0, -14.0);
        draw_circle(b.x, b.y, 7.0, a(Color::new(0.15, 0.6, 0.25, 1.0)));
        draw_circle_lines(b.x, b.y, 7.0, 1.5, a(WHITE));
        draw_triangle(
            vec2(b.x, b.y - 4.5),
            vec2(b.x - 4.0, b.y + 1.0),
            vec2(b.x + 4.0, b.y + 1.0),
            a(WHITE),
        );
        draw_rectangle(b.x - 1.5, b.y, 3.0, 4.0, a(WHITE));
    }
    if top > 0 {
        let label = tower.tier_label();
        let pos = c + vec2(0.0, TOWER_RADIUS + 8.0);
        let w = measure_text(&label, None, 14, 1.0).width;
        draw_rectangle(
            pos.x - w / 2.0 - 3.0,
            pos.y - 7.0,
            w + 6.0,
            14.0,
            with_alpha(BLACK, 0.6 * alpha),
        );
        text_centered(&label, pos, 14, with_alpha(GOLD, alpha));
    }
}

/// A colonial rifleman seen from above. His gun changes with the upgrade
/// path he's furthest along: musket/rifle, dual pistols or a light MG.
fn draw_minuteman(tower: &Tower, c: Vec2, alpha: f32) {
    let a = |col: Color| with_alpha(col, col.a * alpha);
    let dir = vec2(tower.angle.cos(), tower.angle.sin());
    let side = vec2(-dir.y, dir.x);
    let recoil = (1.0 - tower.since_shot / 0.1).max(0.0) * 3.0;
    let line = |p: Vec2, q: Vec2, w: f32, col: Color| draw_line(p.x, p.y, q.x, q.y, w, a(col));

    // Trampled dirt patch.
    draw_circle(c.x, c.y, TOWER_RADIUS, a(Color::new(0.55, 0.45, 0.32, 1.0)));
    draw_circle_lines(
        c.x,
        c.y,
        TOWER_RADIUS,
        2.0,
        a(Color::new(0.38, 0.30, 0.20, 1.0)),
    );

    let [rifle, pistols, gunner] = tower.tiers;
    let build = if rifle + pistols + gunner == 0 {
        0
    } else if gunner >= rifle && gunner >= pistols {
        2
    } else if pistols >= rifle {
        1
    } else {
        0
    };
    let coat = tower.kind.color();
    let metal = Color::new(0.20, 0.20, 0.22, 1.0);
    let wood = Color::new(0.45, 0.28, 0.14, 1.0);
    let skin = Color::new(0.93, 0.78, 0.62, 1.0);

    // Shoulders and coat.
    let back = c - dir * 2.0;
    draw_circle(back.x, back.y, 8.0, a(coat));
    for s in [-1.0, 1.0] {
        let sh = back + side * s * 6.5;
        draw_circle(sh.x, sh.y, 4.5, a(darken(coat, 0.85)));
    }

    let mut muzzles = Vec::new();
    match build {
        1 => {
            // Dual-wielded pistols.
            for s in [-1.0, 1.0] {
                let hand = c + side * s * 8.0 + dir * 5.0;
                let tip = hand + dir * (9.0 - recoil);
                line(back + side * s * 6.5, hand, 3.0, darken(coat, 0.85));
                line(hand - dir * 2.0, tip, 3.5, metal);
                draw_circle(hand.x, hand.y, 2.2, a(skin));
                muzzles.push(tip);
            }
        }
        2 => {
            // Light machine gun with an ammo box and bipod.
            let base = c + side * 3.0 - dir * 4.0;
            let tip = base + dir * (28.0 - recoil);
            line(base, base + dir * 10.0, 7.0, metal);
            line(base, tip, 4.0, metal);
            let box_c = c - side * 7.0 + dir * 4.0;
            draw_rectangle(
                box_c.x - 4.0,
                box_c.y - 4.0,
                8.0,
                8.0,
                a(Color::new(0.35, 0.40, 0.22, 1.0)),
            );
            let bipod = base + dir * 20.0;
            line(bipod, bipod + side * 5.0 - dir * 3.0, 1.5, metal);
            line(bipod, bipod - side * 5.0 - dir * 3.0, 1.5, metal);
            muzzles.push(tip);
        }
        _ => {
            // Musket, becoming a scoped rifle on the rifle path.
            let base = c + side * 4.0 - dir * 6.0;
            let tip = base + dir * (28.0 - recoil);
            line(base, base + dir * 12.0, 4.5, wood);
            line(base + dir * 8.0, tip, 2.5, metal);
            if rifle >= 2 {
                let s0 = base + dir * 9.0 + side * 3.0;
                line(s0, s0 + dir * 8.0, 3.0, Color::new(0.08, 0.08, 0.1, 1.0));
            }
            muzzles.push(tip);
        }
    }
    // Arms reaching to the weapon (pistols draw their own).
    if build != 1 {
        let grip = c + side * 4.0 + dir * 6.0;
        line(back + side * 6.5, grip, 3.0, darken(coat, 0.85));
        line(
            back - side * 6.5,
            grip - side * 2.0,
            3.0,
            darken(coat, 0.85),
        );
        draw_circle(grip.x, grip.y, 2.2, a(skin));
    }

    // Head with a tricorn hat.
    let head = c - dir * 1.0;
    draw_circle(head.x, head.y, 5.5, a(skin));
    draw_triangle(
        head + dir * 6.5,
        head - dir * 4.0 + side * 7.0,
        head - dir * 4.0 - side * 7.0,
        a(Color::new(0.12, 0.10, 0.10, 1.0)),
    );
    draw_circle(head.x, head.y, 2.5, a(Color::new(0.22, 0.18, 0.16, 1.0)));

    if tower.since_shot < 0.05 {
        for m in muzzles {
            let f = m + dir * 3.0;
            draw_circle(f.x, f.y, 4.0, with_alpha(YELLOW, 0.85 * alpha));
        }
    }
}

fn draw_farm(tower: &Tower, c: Vec2, time: f32, alpha: f32) {
    let a = |col: Color| with_alpha(col, col.a * alpha);
    let soil = a(Color::new(0.45, 0.30, 0.18, 1.0));
    draw_rectangle(c.x - 18.0, c.y - 18.0, 36.0, 36.0, soil);
    // Wheat rows swaying in the wind.
    for row in 0..3 {
        let y = c.y - 11.0 + row as f32 * 10.0;
        for col in 0..5 {
            let x = c.x - 14.0 + col as f32 * 4.5;
            let sway = (time * 2.0 + x * 0.3).sin() * 1.2;
            draw_line(
                x,
                y + 3.0,
                x + sway,
                y - 3.0,
                2.0,
                a(Color::new(0.85, 0.75, 0.30, 1.0)),
            );
        }
    }
    // Fence.
    let fence = a(Color::new(0.70, 0.55, 0.35, 1.0));
    draw_rectangle_lines(c.x - 18.0, c.y - 18.0, 36.0, 36.0, 2.0, fence);
    for k in 0..4 {
        let x = c.x - 18.0 + k as f32 * 12.0;
        draw_rectangle(x - 1.0, c.y + 15.0, 2.5, 5.0, fence);
    }
    // Barn.
    let barn = a(Color::new(0.70, 0.20, 0.18, 1.0));
    draw_rectangle(c.x + 6.0, c.y - 2.0, 11.0, 12.0, barn);
    draw_triangle(
        vec2(c.x + 4.0, c.y - 2.0),
        vec2(c.x + 19.0, c.y - 2.0),
        vec2(c.x + 11.5, c.y - 10.0),
        barn,
    );
    draw_rectangle(
        c.x + 9.5,
        c.y + 3.0,
        4.0,
        7.0,
        a(Color::new(0.95, 0.9, 0.85, 1.0)),
    );
    // Bank path: a stack of coins. Support path: a banner.
    if tower.tiers[1] > 0 {
        for k in 0..tower.tiers[1] {
            draw_circle(c.x - 12.0, c.y + 10.0 - k as f32 * 3.0, 4.0, a(GOLD));
            draw_circle_lines(c.x - 12.0, c.y + 10.0 - k as f32 * 3.0, 4.0, 1.0, a(ORANGE));
        }
    }
    if tower.tiers[2] > 0 {
        draw_line(
            c.x - 16.0,
            c.y - 16.0,
            c.x - 16.0,
            c.y - 34.0,
            2.0,
            a(DARKGRAY),
        );
        let wave = (time * 4.0).sin() * 2.0;
        draw_triangle(
            vec2(c.x - 16.0, c.y - 34.0),
            vec2(c.x - 2.0, c.y - 30.0 + wave),
            vec2(c.x - 16.0, c.y - 25.0),
            a(Color::new(0.15, 0.6, 0.25, 1.0)),
        );
    }
}

// ---------------------------------------------------------------- units

fn draw_enemies(game: &Game) {
    for e in &game.enemies {
        let r = e.kind.radius();
        let mut color = e.kind.color();
        if e.slowed() {
            color = Color::new(color.r * 0.5 + 0.2, color.g * 0.6 + 0.3, 1.0, 1.0);
        }
        if e.stunned() {
            color = Color::new(0.85, 0.85, 0.85, 1.0);
        }
        if e.since_hit < 0.06 {
            color = WHITE;
        }
        draw_circle(e.pos.x + 2.0, e.pos.y + 3.0, r, with_alpha(BLACK, 0.25));
        match e.kind {
            EnemyKind::Tank => {
                draw_rectangle(e.pos.x - r, e.pos.y - r, r * 2.0, r * 2.0, color);
                draw_rectangle_lines(e.pos.x - r, e.pos.y - r, r * 2.0, r * 2.0, 2.0, BLACK);
                draw_rectangle(
                    e.pos.x - r * 0.5,
                    e.pos.y - r * 0.5,
                    r,
                    r,
                    darken(color, 0.8),
                );
            }
            EnemyKind::Runner => {
                draw_poly(e.pos.x, e.pos.y, 3, r + 2.0, game.time * 360.0, color);
            }
            EnemyKind::Boss => {
                draw_poly(e.pos.x, e.pos.y, 8, r, game.time * 30.0, color);
                draw_poly_lines(e.pos.x, e.pos.y, 8, r, game.time * 30.0, 3.0, BLACK);
                draw_circle(e.pos.x - 6.0, e.pos.y - 4.0, 3.0, YELLOW);
                draw_circle(e.pos.x + 6.0, e.pos.y - 4.0, 3.0, YELLOW);
            }
            EnemyKind::Grunt => {
                draw_circle(e.pos.x, e.pos.y, r, color);
                draw_circle_lines(e.pos.x, e.pos.y, r, 2.0, with_alpha(BLACK, 0.6));
                draw_circle(e.pos.x - 3.5, e.pos.y - 2.0, 1.8, BLACK);
                draw_circle(e.pos.x + 3.5, e.pos.y - 2.0, 1.8, BLACK);
            }
        }
        if e.frozen() {
            let s = r + 4.0;
            draw_rectangle(
                e.pos.x - s,
                e.pos.y - s,
                s * 2.0,
                s * 2.0,
                Color::new(0.7, 0.9, 1.0, 0.55),
            );
            draw_rectangle_lines(
                e.pos.x - s,
                e.pos.y - s,
                s * 2.0,
                s * 2.0,
                2.0,
                Color::new(0.9, 0.97, 1.0, 0.9),
            );
        } else if e.freeze_immune() {
            // Thawing out: can't be refrozen yet.
            draw_circle_lines(
                e.pos.x,
                e.pos.y,
                r + 4.0,
                1.5,
                Color::new(0.7, 0.9, 1.0, 0.6),
            );
        }
        if e.burning() {
            let flicker = (game.time * 20.0 + e.id as f32).sin() * 2.0;
            draw_circle(
                e.pos.x,
                e.pos.y - r - 2.0 + flicker,
                4.0,
                with_alpha(ORANGE, 0.9),
            );
        }

        if e.hp < e.max_hp {
            let w = r * 2.4;
            let x = e.pos.x - w / 2.0;
            let y = e.pos.y - r - 9.0;
            let frac = (e.hp / e.max_hp).clamp(0.0, 1.0);
            let bar = if frac > 0.5 {
                GREEN
            } else if frac > 0.25 {
                ORANGE
            } else {
                RED
            };
            draw_rectangle(x - 1.0, y - 1.0, w + 2.0, 6.0, BLACK);
            draw_rectangle(x, y, w * frac, 4.0, bar);
        }
    }
}

fn draw_projectiles(game: &Game) {
    for p in &game.projectiles {
        match &p.shot {
            Shot::Arrow { dir, .. } => {
                let tail = p.pos - *dir * 12.0;
                draw_line(
                    tail.x,
                    tail.y,
                    p.pos.x,
                    p.pos.y,
                    2.5,
                    Color::from_rgba(255, 230, 150, 255),
                );
                draw_circle(p.pos.x, p.pos.y, 2.0, WHITE);
            }
            Shot::Shell { .. } => {
                let fill = if p.stats.burn_dps > 0.0 {
                    Color::from_rgba(200, 70, 30, 255)
                } else {
                    Color::from_rgba(40, 40, 40, 255)
                };
                draw_circle(p.pos.x, p.pos.y, 5.5, fill);
            }
        }
    }
}

fn draw_effects(game: &Game) {
    for e in &game.effects {
        let k = 1.0 - e.life / e.max_life;
        match e.shape {
            EffectShape::Ring(radius) => {
                let r = radius * (0.4 + 0.6 * k);
                draw_circle(e.pos.x, e.pos.y, r, with_alpha(e.color, 0.25 * (1.0 - k)));
                draw_circle_lines(e.pos.x, e.pos.y, r, 2.0, with_alpha(e.color, 1.0 - k));
            }
            EffectShape::Line(to) => {
                draw_line(
                    e.pos.x,
                    e.pos.y,
                    to.x,
                    to.y,
                    2.0,
                    with_alpha(e.color, 1.0 - k),
                );
            }
        }
    }
    for f in &game.floaters {
        let a = f.life.min(1.0);
        text_centered(&f.text, f.pos + vec2(1.0, 1.0), 20, with_alpha(BLACK, a));
        text_centered(&f.text, f.pos, 20, with_alpha(f.color, a));
    }
}

// ---------------------------------------------------------------- HUD

fn draw_party_badge(game: &Game) {
    let Some(text) = &game.party else {
        return;
    };
    let w = measure_text(text, None, 18, 1.0).width;
    let r = Rect::new(MAP_X + 8.0, MAP_Y + MAP_H - 30.0, w + 16.0, 22.0);
    draw_rectangle(r.x, r.y, r.w, r.h, with_alpha(BLACK, 0.6));
    draw_text(text, r.x + 8.0, r.y + 16.0, 18.0, WHITE);
}

fn draw_top_bar(game: &Game, t: &Theme) {
    draw_rectangle(0.0, 0.0, SCREEN_W, TOP_BAR, t.panel);
    draw_line(0.0, TOP_BAR, SCREEN_W, TOP_BAR, 2.0, t.border);

    draw_circle(24.0, 24.0, 10.0, GOLD);
    draw_circle_lines(24.0, 24.0, 10.0, 2.0, ORANGE);
    draw_text(game.gold.to_string(), 42.0, 32.0, 28.0, t.gold);

    let heart = vec2(160.0, 24.0);
    draw_circle(heart.x - 5.0, heart.y - 3.0, 6.0, RED);
    draw_circle(heart.x + 5.0, heart.y - 3.0, 6.0, RED);
    draw_triangle(
        vec2(heart.x - 11.0, heart.y - 1.0),
        vec2(heart.x + 11.0, heart.y - 1.0),
        vec2(heart.x, heart.y + 11.0),
        RED,
    );
    draw_text(game.lives.to_string(), 178.0, 32.0, 28.0, t.text);

    if game.sandbox {
        let label = format!("Wave {} / ", game.wave);
        let w = measure_text(&label, None, 28, 1.0).width;
        draw_text(&label, 250.0, 32.0, 28.0, t.text);
        draw_infinity(vec2(250.0 + w + 16.0, 23.0), 14.0, t.text);
    } else {
        draw_text(
            format!("Wave {}/{}", game.wave, MAX_WAVES),
            250.0,
            32.0,
            28.0,
            t.text,
        );
    }
    if game.sandbox {
        let r = Rect::new(520.0, 10.0, 110.0, 28.0);
        panel_rect(r, Color::new(0.85, 0.6, 0.1, 1.0), t.border);
        text_centered("SANDBOX", r.center(), 20, BLACK);
    }

    text_right(
        &format!("Map: {}", game.map.name),
        ui::settings_button().x - 16.0,
        30.0,
        20,
        t.text_dim,
    );
    button(ui::settings_button(), "Settings", true, BLUE, t);
    let pause_label = if game.paused { "Resume" } else { "Pause" };
    button(ui::pause_button(), pause_label, true, BLUE, t);
    button(
        ui::speed_button(),
        &format!("Speed x{}", game.speed),
        true,
        BLUE,
        t,
    );
}

/// The default font has no infinity glyph, so draw a lemniscate instead.
fn draw_infinity(center: Vec2, size: f32, color: Color) {
    let point = |k: f32| {
        let s = k.sin();
        let d = 1.0 + s * s;
        center + vec2(k.cos() / d, s * k.cos() / d) * size
    };
    let steps = 48;
    for i in 0..steps {
        let k0 = i as f32 / steps as f32 * std::f32::consts::TAU;
        let k1 = (i + 1) as f32 / steps as f32 * std::f32::consts::TAU;
        let (p, q) = (point(k0), point(k1));
        draw_line(p.x, p.y, q.x, q.y, 3.0, color);
    }
}

fn draw_sidebar(game: &Game, t: &Theme) {
    draw_rectangle(SIDEBAR_X, TOP_BAR, SIDEBAR_W, MAP_H, t.panel);
    draw_line(SIDEBAR_X, TOP_BAR, SIDEBAR_X, SCREEN_H, 2.0, t.border);

    match game.selected.and_then(|id| game.tower_index(id)) {
        Some(i) => draw_tower_panel(game, i, t),
        None => draw_build_panel(game, t),
    }

    let label = if game.wave_active {
        format!("{} enemies left", game.enemies.len())
    } else if game.wave >= game.max_wave() {
        "All waves done".to_string()
    } else {
        format!("Start wave {}", game.wave + 1)
    };
    button(
        ui::wave_button(),
        &label,
        game.can_start_wave(),
        GREEN_BTN,
        t,
    );
    let (auto_label, auto_color) = if game.auto_play {
        ("Auto ON", GREEN_BTN)
    } else {
        ("Auto OFF", GREY_BTN)
    };
    button(ui::auto_button(), auto_label, true, auto_color, t);
}

fn tower_card(r: Rect, kind: TowerKind, number: usize, affordable: bool, t: &Theme) {
    let icon = vec2(r.x + 26.0, r.y + r.h / 2.0);
    draw_circle(icon.x, icon.y, 15.0, kind.color());
    draw_circle_lines(icon.x, icon.y, 15.0, 2.0, BLACK);
    text_centered(&number.to_string(), icon, 20, BLACK);
    let name_color = if affordable { t.text } else { t.text_dim };
    draw_text(kind.name(), r.x + 50.0, r.y + 22.0, 23.0, name_color);
    if kind.is_starter() {
        let w = measure_text(kind.name(), None, 23, 1.0).width;
        let tag = Rect::new(r.x + 56.0 + w, r.y + 8.0, 58.0, 16.0);
        draw_rectangle(
            tag.x,
            tag.y,
            tag.w,
            tag.h,
            Color::new(0.24, 0.59, 0.31, 1.0),
        );
        text_centered("STARTER", tag.center(), 14, WHITE);
    }
    let cost_color = if affordable { t.gold } else { t.bad };
    text_right(
        &format!("{}g", kind.cost()),
        r.x + r.w - 10.0,
        r.y + 22.0,
        22,
        cost_color,
    );
    draw_text(kind.description(), r.x + 50.0, r.y + 42.0, 16.0, t.text_dim);
}

fn draw_build_panel(game: &Game, t: &Theme) {
    draw_text("TOWERS", SIDEBAR_X + 12.0, TOP_BAR + 26.0, 24.0, t.text);

    for (i, kind) in TowerKind::ALL.into_iter().enumerate() {
        let r = ui::tower_button(i);
        let chosen = game.build_choice == Some(kind);
        let fill = if chosen {
            Color::new(0.45, 0.60, 0.85, 1.0)
        } else if r.contains(mouse()) {
            t.panel_hover
        } else {
            t.panel_light
        };
        let border = if chosen { t.text } else { t.border };
        panel_rect(r, fill, border);
        tower_card(r, kind, i + 1, game.can_afford(kind.cost()), t);
    }

    if game.build_choice.is_some() {
        button(
            ui::cancel_button(),
            "Cancel placing (Esc)",
            true,
            RED_BTN,
            t,
        );
    }

    let x = SIDEBAR_X + 14.0;
    if game.sandbox {
        draw_text("SANDBOX", x, TOP_BAR + 434.0, 22.0, t.text);
        text_right(
            "shift = x5",
            SIDEBAR_X + SIDEBAR_W - 12.0,
            TOP_BAR + 434.0,
            16,
            t.text_dim,
        );
        for (i, kind) in ui::SANDBOX_SPAWNS.into_iter().enumerate() {
            let name = format!("+ {kind:?}");
            button(
                ui::sandbox_spawn_button(i),
                &name,
                true,
                darken(kind.color(), 0.7),
                t,
            );
        }
        button(ui::sandbox_wave_down(), "-", !game.wave_active, BLUE, t);
        button(ui::sandbox_wave_up(), "+", !game.wave_active, BLUE, t);
        let label = format!("Next wave: {}", game.wave + 1);
        let mid = (ui::sandbox_wave_down().center() + ui::sandbox_wave_up().center()) / 2.0;
        text_centered(&label, mid, 20, t.text);
        button(
            ui::sandbox_clear_button(),
            "Clear enemies",
            true,
            RED_BTN,
            t,
        );
        return;
    }

    let mut y = TOP_BAR + 432.0;
    let help = [
        "Click grass to place a tower.",
        "Shift+click places several.",
        "Click a tower to upgrade it.",
        "",
        "1-6  pick tower   Tab  target",
        ", . /  upgrade paths 1-3",
        "S  sell     Space  next wave",
        "A  auto waves   F  speed",
        "P  pause    Esc  menu",
    ];
    for line in help {
        draw_text(line, x, y, 18.0, t.text_dim);
        y += 19.0;
    }
}

fn stat_lines(kind: TowerKind, s: &Stats) -> Vec<String> {
    let mut lines = Vec::new();
    if kind != TowerKind::Farm {
        lines.push(format!(
            "Damage {:.0}   Rate {:.2}/s",
            s.damage,
            1.0 / s.cooldown
        ));
        if s.range.is_finite() {
            lines.push(format!("Range {:.1} tiles", s.range / TILE));
        } else {
            lines.push("Range: global".to_string());
        }
    }
    match kind {
        TowerKind::Arrow => lines.push(format!(
            "{} arrow(s), pierce {:.0} tiles",
            s.arrows,
            s.pierce / TILE
        )),
        TowerKind::MinuteMan => {
            let mut extra = Vec::new();
            if s.targets > 1 {
                extra.push(format!("{} targets", s.targets));
            }
            if s.penetrate > 0 {
                extra.push(format!("Pierce {}", s.penetrate));
            }
            if s.slow < 1.0 {
                extra.push(format!("Slow {:.0}%", (1.0 - s.slow) * 100.0));
            }
            if s.boss_mult > 1.0 {
                extra.push(format!("{:.0}x bosses", s.boss_mult));
            }
            if !extra.is_empty() {
                lines.push(extra.join("  "));
            }
        }
        TowerKind::Cannon => {
            lines.push(format!("Splash {:.1} tiles", s.splash / TILE));
            let mut extra = Vec::new();
            if s.burn_dps > 0.0 {
                extra.push(format!("Burn {:.0}/s {:.0}s", s.burn_dps, s.burn_time));
            }
            if s.stun_time > 0.0 {
                extra.push(format!("Stun {:.1}s", s.stun_time));
            }
            if !extra.is_empty() {
                lines.push(extra.join("  "));
            }
        }
        TowerKind::Frost => {
            lines.push(format!("Freeze {:.1}s", s.freeze_time));
            let mut extra = Vec::new();
            if s.slow < 1.0 {
                extra.push(format!(
                    "Slow {:.0}% {:.0}s",
                    (1.0 - s.slow) * 100.0,
                    s.slow_time
                ));
            }
            if s.brittle > 0.0 {
                extra.push(format!("Brittle +{:.0}%", s.brittle * 100.0));
            }
            if !extra.is_empty() {
                lines.push(extra.join("  "));
            }
        }
        TowerKind::Sniper => {
            let mut extra = Vec::new();
            if s.bounces > 0 {
                extra.push(format!("Bounces {}", s.bounces));
            }
            if s.boss_mult > 1.0 {
                extra.push(format!("{:.0}x vs bosses", s.boss_mult));
            }
            if !extra.is_empty() {
                lines.push(extra.join("  "));
            }
        }
        TowerKind::Farm => {}
    }
    if s.income > 0 {
        lines.push(format!("Income {}g per wave", s.income));
    }
    if s.interest > 0.0 {
        lines.push(format!(
            "Interest {:.0}% (max {}g)",
            s.interest * 100.0,
            s.interest_cap
        ));
    }
    if s.buff_radius > 0.0 {
        lines.push(format!(
            "Buff +{:.0}% spd +{:.0}% rng +{:.0}% dmg",
            s.buff_speed * 100.0,
            s.buff_range * 100.0,
            s.buff_damage * 100.0
        ));
    }
    lines
}

fn draw_tower_panel(game: &Game, index: usize, t: &Theme) {
    let tower = &game.towers[index];
    let x = SIDEBAR_X + 14.0;
    let name_color = darken(tower.kind.color(), if t.night { 1.0 } else { 0.75 });
    draw_text(tower.kind.name(), x, TOP_BAR + 32.0, 30.0, name_color);
    if game.is_buffed(index) {
        let w = measure_text(tower.kind.name(), None, 30, 1.0).width;
        draw_text(
            "BUFFED",
            x + w + 10.0,
            TOP_BAR + 30.0,
            18.0,
            Color::new(0.15, 0.6, 0.25, 1.0),
        );
    }
    button(ui::close_button(), "x", true, GREY_BTN, t);

    let stats = game.effective_stats(index);
    let mut y = TOP_BAR + 58.0;
    for line in stat_lines(tower.kind, &stats).iter().take(4) {
        draw_text(line, x, y, 17.0, t.text);
        y += 18.0;
    }

    if tower.kind.has_targeting() {
        let label = format!("Target: {} (Tab)", tower.targeting.name());
        button(ui::target_button(), &label, true, BLUE, t);
    } else if tower.kind == TowerKind::Frost {
        draw_text(
            "Hits everything in range",
            x,
            TOP_BAR + 140.0,
            17.0,
            t.text_dim,
        );
    }

    let names = tower.kind.path_names();
    for (path, name) in names.iter().enumerate() {
        let r = ui::path_card(path);
        let tier = tower.tiers[path];
        let next = tower.next_upgrade(path);
        let open = tower.path_open(path);
        let affordable = tower.upgrade_cost(path).is_some_and(|c| game.can_afford(c));
        let hover = r.contains(mouse());
        let fill = if open && affordable && hover {
            t.panel_hover
        } else {
            t.panel_light
        };
        panel_rect(r, fill, t.border);
        draw_text(
            format!("{name} ({})", ui::PATH_KEY_LABELS[path]),
            r.x + 10.0,
            r.y + 18.0,
            17.0,
            t.text_dim,
        );
        draw_tier_pips(r, tier, t);

        match next {
            None => {
                draw_text("MAXED", r.x + 10.0, r.y + 46.0, 24.0, t.gold);
            }
            Some(u) => {
                let name_color = if open { t.text } else { t.text_dim };
                draw_text(u.name, r.x + 10.0, r.y + 44.0, 22.0, name_color);
                draw_text(u.desc, r.x + 10.0, r.y + 64.0, 16.0, t.text_dim);
                if open {
                    let c = if affordable { t.gold } else { t.bad };
                    text_right(&format!("{}g", u.cost), r.x + r.w - 10.0, r.y + 78.0, 20, c);
                } else {
                    text_right("Locked", r.x + r.w - 10.0, r.y + 78.0, 18, t.bad);
                }
            }
        }
    }

    button(
        ui::sell_button(),
        &format!("Sell for {}g (S)", tower.sell_value()),
        true,
        RED_BTN,
        t,
    );

    let mut y = ui::sell_button().y + ui::sell_button().h + 22.0;
    for line in [
        "Only 2 paths per tower, and",
        "only one can go past tier 2.",
    ] {
        draw_text(line, x, y, 17.0, t.text_dim);
        y += 18.0;
    }
}

fn draw_tier_pips(r: Rect, tier: u8, t: &Theme) {
    for i in 0..TIERS {
        let px = r.x + r.w - 14.0 - (TIERS - 1 - i) as f32 * 14.0;
        let c = if i < tier {
            GREEN
        } else {
            with_alpha(t.text, 0.15)
        };
        draw_rectangle(px - 5.0, r.y + 8.0, 10.0, 10.0, c);
    }
}

fn draw_message(game: &Game) {
    let Some((text, life)) = &game.message else {
        return;
    };
    let a = life.min(1.0);
    let center = vec2(MAP_W / 2.0, TOP_BAR + 30.0);
    let m = measure_text(text, None, 30, 1.0);
    draw_rectangle(
        center.x - m.width / 2.0 - 16.0,
        center.y - 20.0,
        m.width + 32.0,
        40.0,
        with_alpha(BLACK, 0.55 * a),
    );
    text_centered(text, center, 30, with_alpha(WHITE, a));
}

fn draw_overlay(game: &Game, t: &Theme) {
    let (title, color) = match game.state {
        GameState::Playing if game.paused && game.menu.is_none() => ("PAUSED", WHITE),
        GameState::Playing => return,
        GameState::GameOver => ("GAME OVER", RED),
        GameState::Victory => ("VICTORY!", GOLD),
    };
    draw_rectangle(0.0, TOP_BAR, MAP_W, MAP_H, with_alpha(BLACK, 0.55));
    let center = vec2(MAP_W / 2.0, TOP_BAR + MAP_H / 2.0);
    text_centered(title, center - vec2(0.0, 30.0), 72, color);
    let sub = match game.state {
        GameState::Playing => "Press P to resume".to_string(),
        GameState::GameOver => format!("You were overrun on wave {}.", game.wave),
        GameState::Victory => format!("All {MAX_WAVES} waves defeated!"),
    };
    text_centered(&sub, center + vec2(0.0, 30.0), 26, WHITE);
    if game.state == GameState::Playing {
        return;
    }
    if game.state == GameState::GameOver {
        button(
            ui::retry_button(),
            &format!("Retry wave {} (R)", game.wave),
            true,
            GREEN_BTN,
            t,
        );
    }
    let label = if game.is_client {
        "Host restarts"
    } else {
        "New game (Enter)"
    };
    button(ui::new_game_button(), label, !game.is_client, BLUE, t);
}

// ---------------------------------------------------------------- menus

fn dim_screen() {
    draw_rectangle(0.0, 0.0, SCREEN_W, SCREEN_H, with_alpha(BLACK, 0.6));
}

fn draw_settings(game: &Game, in_game: bool, t: &Theme) {
    dim_screen();
    let items = ui::settings_items(in_game, game.is_client);
    let n = items.len();
    let p = ui::settings_panel(n);
    panel_rect(p, t.panel, t.border);
    text_centered("Settings", vec2(p.center().x, p.y + 34.0), 32, t.text);
    text_right(
        &crate::version_label(),
        p.x + p.w - 12.0,
        p.y + 22.0,
        18,
        t.text_dim,
    );

    for (i, item) in items.iter().enumerate() {
        let r = ui::settings_row(n, i);
        let (label, color) = match item {
            SettingsItem::Theme => (
                if game.night {
                    "Theme: Night"
                } else {
                    "Theme: Day"
                },
                BLUE,
            ),
            SettingsItem::Guide => ("Tower guide", BLUE),
            SettingsItem::Restart => ("Restart", GREEN_BTN),
            SettingsItem::MainMenu => {
                if game.online {
                    ("Leave party", ORANGE_BTN)
                } else {
                    ("Main menu", ORANGE_BTN)
                }
            }
            SettingsItem::Close => (
                if in_game {
                    "Resume (Esc)"
                } else {
                    "Close (Esc)"
                },
                GREY_BTN,
            ),
            SettingsItem::TowerVolume => {
                draw_slider(r, "Tower sounds", game.tower_volume, t);
                continue;
            }
            SettingsItem::GameVolume => {
                draw_slider(r, "Game sounds", game.game_volume, t);
                continue;
            }
        };
        button(r, label, true, color, t);
    }
    if game.online {
        text_centered(
            "Menus don't pause the game in a party.",
            vec2(p.center().x, p.y + p.h - 14.0),
            16,
            t.text_dim,
        );
    }
}

fn draw_slider(row: Rect, label: &str, value: f32, t: &Theme) {
    panel_rect(row, t.panel_light, t.border);
    draw_text(
        format!("{label}  {:.0}%", value * 100.0),
        row.x + 12.0,
        row.y + 28.0,
        20.0,
        t.text,
    );
    let track = ui::slider_track(row);
    draw_rectangle(track.x, track.y, track.w, track.h, with_alpha(t.text, 0.2));
    draw_rectangle(track.x, track.y, track.w * value, track.h, BLUE);
    let knob = vec2(track.x + track.w * value, track.y + track.h / 2.0);
    draw_circle(knob.x, knob.y, 10.0, WHITE);
    draw_circle_lines(knob.x, knob.y, 10.0, 2.0, BLUE);
}

fn draw_guide(index: usize, t: &Theme) {
    dim_screen();
    let p = ui::guide_panel();
    panel_rect(p, t.panel, t.border);
    draw_text("Tower guide", p.x + 20.0, p.y + 38.0, 32.0, t.text);
    text_right(
        "Left/Right to browse, Esc to go back",
        ui::guide_close().x - 16.0,
        p.y + 34.0,
        18,
        t.text_dim,
    );
    button(ui::guide_close(), "x", true, GREY_BTN, t);

    let kind = TowerKind::ALL[index];
    for (i, k) in TowerKind::ALL.into_iter().enumerate() {
        let r = ui::guide_tab(i);
        let fill = if i == index {
            darken(k.color(), 0.8)
        } else if r.contains(mouse()) {
            t.panel_hover
        } else {
            t.panel_light
        };
        panel_rect(r, fill, t.border);
        let c = if i == index { WHITE } else { t.text };
        text_centered(k.name(), r.center(), 22, c);
    }

    let header = Rect::new(p.x + 20.0, p.y + 104.0, p.w - 40.0, 112.0);
    panel_rect(header, t.panel_light, t.border);
    let sample = Tower::new(0, kind, vec2(header.x + 40.0, header.y + 44.0));
    sample_tower(&sample);
    draw_text(kind.name(), header.x + 76.0, header.y + 34.0, 30.0, t.text);
    draw_text(
        format!("{}g  -  {}", kind.cost(), kind.description()),
        header.x + 76.0,
        header.y + 58.0,
        20.0,
        t.text_dim,
    );
    let mut x = header.x + 76.0;
    for line in stat_lines(kind, &sample.stats()) {
        let w = measure_text(&line, None, 18, 1.0).width;
        draw_text(&line, x, header.y + 92.0, 18.0, t.text);
        x += w + 28.0;
    }

    let names = kind.path_names();
    for (path, upgrades) in kind.upgrades().iter().enumerate() {
        let first = ui::guide_card(path, 0);
        draw_text(
            format!(
                "Path {}: {}  ({})",
                path + 1,
                names[path],
                ui::PATH_KEY_LABELS[path]
            ),
            first.x,
            first.y - 8.0,
            20.0,
            t.text,
        );
        for (tier, u) in upgrades.iter().enumerate().take(ui::TIER_COUNT) {
            let r = ui::guide_card(path, tier);
            panel_rect(r, t.panel_light, t.border);
            draw_text(
                format!("Tier {}", tier + 1),
                r.x + 12.0,
                r.y + 22.0,
                17.0,
                t.text_dim,
            );
            text_right(
                &format!("{}g", u.cost),
                r.x + r.w - 12.0,
                r.y + 22.0,
                20,
                t.gold,
            );
            draw_text(u.name, r.x + 12.0, r.y + 48.0, 24.0, t.text);
            draw_text(u.desc, r.x + 12.0, r.y + 70.0, 18.0, t.text_dim);
        }
    }
}

/// Draws a tower icon for the guide.
fn sample_tower(tower: &Tower) {
    draw_tower(tower, tower.pos, get_time() as f32, 1.0, false, &DAY);
}

pub fn draw_lobby(lobby: &Lobby, game: &Game) {
    let t = theme(game);
    let time = get_time() as f32;
    clear_background(t.panel);
    draw_map(&lobby.preview, time, 10.0, t);
    draw_rectangle(0.0, 0.0, SCREEN_W, SCREEN_H, with_alpha(BLACK, 0.45));

    let title = "RUSTY TOWER DEFENSE";
    text_centered(
        title,
        vec2(SCREEN_W / 2.0 + 3.0, 73.0),
        64,
        with_alpha(BLACK, 0.6),
    );
    text_centered(
        title,
        vec2(SCREEN_W / 2.0, 70.0),
        64,
        Color::new(1.0, 0.85, 0.3, 1.0),
    );
    text_centered(
        &crate::version_label(),
        vec2(SCREEN_W / 2.0, 112.0),
        22,
        WHITE,
    );

    let p = ui::lobby_panel();
    panel_rect(p, t.panel, t.border);

    draw_text("Map", p.x + 20.0, p.y + 30.0, 22.0, t.text);
    for i in 0..MAP_COUNT {
        let r = ui::lobby_map_card(i);
        let map = Map::new(i);
        let chosen = lobby.map == i;
        let fill = if chosen {
            Color::new(0.45, 0.60, 0.85, 1.0)
        } else if r.contains(mouse()) {
            t.panel_hover
        } else {
            t.panel_light
        };
        panel_rect(r, fill, if chosen { t.text } else { t.border });
        let c = if chosen { WHITE } else { t.text };
        draw_text(map.name, r.x + 12.0, r.y + 30.0, 28.0, c);
        let d = if chosen { WHITE } else { t.text_dim };
        draw_text(map.description, r.x + 12.0, r.y + 56.0, 16.0, d);
    }

    draw_text("Mode", p.x + 20.0, p.y + 142.0, 22.0, t.text);
    for sandbox in [false, true] {
        let r = ui::lobby_mode_button(sandbox);
        let chosen = lobby.sandbox == sandbox;
        let color = if chosen { BLUE } else { GREY_BTN };
        let label = if sandbox { "Sandbox" } else { "Normal" };
        panel_rect(r, if chosen { color } else { t.panel_light }, t.border);
        text_centered(label, r.center(), 22, if chosen { WHITE } else { t.text });
    }

    button(ui::lobby_play(), "Play", !lobby.busy, GREEN_BTN, t);

    draw_text("Multiplayer (co-op)", p.x + 20.0, p.y + 296.0, 22.0, t.text);
    button(ui::lobby_host(), "Host party", !lobby.busy, BLUE, t);
    let field = ui::lobby_address();
    let fill = if lobby.typing { WHITE } else { t.panel_light };
    panel_rect(field, fill, if lobby.typing { BLUE } else { t.border });
    let shown = if lobby.address.is_empty() && !lobby.typing {
        "Host address".to_string()
    } else {
        lobby.address.clone()
    };
    let caret = if lobby.typing && (time * 2.0) as i32 % 2 == 0 {
        "|"
    } else {
        ""
    };
    let color = if lobby.typing { BLACK } else { t.text };
    draw_text(
        format!("{shown}{caret}"),
        field.x + 12.0,
        field.y + 29.0,
        22.0,
        color,
    );
    button(ui::lobby_join(), "Join party", !lobby.busy, BLUE, t);

    button(ui::lobby_settings(), "Settings", true, GREY_BTN, t);
    button(ui::lobby_quit(), "Quit", true, RED_BTN, t);

    let status = lobby
        .status
        .as_deref()
        .unwrap_or("Enter starts a game. Friends join with your IP.");
    text_centered(status, vec2(p.center().x, p.y + p.h - 26.0), 18, t.text_dim);

    draw_menus(game, false, t);
}
