use macroquad::prelude::*;

use crate::enemy::EnemyKind;
use crate::game::{EffectShape, Game, GameState, Menu, Shot};
use crate::map::*;
use crate::tower::{PATHS, Stats, TIERS, Tower, TowerKind};
use crate::ui;
use crate::wave::MAX_WAVES;

/// Colors for one of the two visual themes.
struct Theme {
    night: bool,
    grass: Color,
    decor_light: Color,
    decor_dark: Color,
    path_fill: Color,
    path_edge: Color,
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

pub fn draw(game: &Game) {
    let t = if game.night { &NIGHT } else { &DAY };
    clear_background(t.panel);
    draw_map(game, t);
    draw_ranges(game);
    for tower in &game.towers {
        draw_tower(tower, tower.pos, game.time, 1.0, t);
    }
    draw_enemies(game);
    draw_projectiles(game);
    draw_effects(game);
    draw_placement_preview(game, t);
    draw_sidebar(game, t);
    draw_top_bar(game, t);
    draw_message(game);
    draw_overlay(game);
    match game.menu {
        Some(Menu::Settings) => draw_settings(game, t),
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

fn draw_map(game: &Game, t: &Theme) {
    draw_rectangle(MAP_X, MAP_Y, MAP_W, MAP_H, t.grass);
    for &(p, r, shade) in &game.map.decor {
        let c = if shade > 0.5 {
            t.decor_light
        } else {
            t.decor_dark
        };
        draw_circle(p.x, p.y, r, c);
    }

    // Thick strokes with square joints keep the straight track crisp.
    let wps = &game.map.waypoints;
    for (width, color) in [(PATH_WIDTH + 8.0, t.path_edge), (PATH_WIDTH, t.path_fill)] {
        for pair in wps.windows(2) {
            draw_line(pair[0].x, pair[0].y, pair[1].x, pair[1].y, width, color);
        }
        for p in wps {
            draw_rectangle(p.x - width / 2.0, p.y - width / 2.0, width, width, color);
        }
    }

    if t.night {
        // Fireflies drifting over the grass.
        for &(p, _, shade) in game.map.decor.iter().filter(|d| d.2 > 0.9) {
            let phase = game.time * 0.8 + shade * 40.0;
            let pos = p + vec2(phase.sin() * 10.0, (phase * 1.3).cos() * 8.0);
            let glow = 0.5 + 0.5 * (game.time * 3.0 + shade * 20.0).sin();
            draw_circle(pos.x, pos.y, 5.0, Color::new(1.0, 0.95, 0.5, 0.12 * glow));
            draw_circle(pos.x, pos.y, 1.6, Color::new(1.0, 0.95, 0.6, 0.8 * glow));
        }
    }

    let portal = game.map.portal;
    let pulse = (game.time * 3.0).sin() * 3.0;
    draw_circle(portal.x, portal.y, 16.0, with_alpha(PURPLE, 0.35));
    draw_circle_lines(
        portal.x,
        portal.y,
        18.0 + pulse,
        3.0,
        with_alpha(PURPLE, 0.9),
    );

    draw_castle(game);
}

fn draw_castle(game: &Game) {
    // Shake and flash red briefly when an enemy gets through.
    let hit = (1.0 - game.since_base_hit / 0.4).max(0.0);
    let shake = vec2((game.time * 60.0).sin(), (game.time * 47.0).cos()) * 3.0 * hit;
    let base = game.map.base + shake;
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
    draw_rectangle(
        base.x - 6.0,
        base.y + 2.0,
        12.0,
        16.0,
        Color::from_rgba(50, 40, 40, 255),
    );
    draw_line(base.x, base.y - 26.0, base.x, base.y - 44.0, 2.0, DARKGRAY);
    draw_triangle(
        vec2(base.x, base.y - 44.0),
        vec2(base.x + 14.0, base.y - 39.0),
        vec2(base.x, base.y - 34.0),
        RED,
    );
}

fn draw_range(pos: Vec2, stats: &Stats, tint: Color) {
    if stats.range > 0.0 && stats.range.is_finite() {
        draw_circle(pos.x, pos.y, stats.range, with_alpha(tint, 0.10));
        draw_circle_lines(pos.x, pos.y, stats.range, 2.0, with_alpha(tint, 0.55));
    }
}

fn draw_ranges(game: &Game) {
    if let Some(tower) = game.selected.and_then(|i| game.towers.get(i)) {
        draw_range(tower.pos, &tower.stats(), WHITE);
    }
}

fn draw_placement_preview(game: &Game, t: &Theme) {
    let m = mouse();
    if game.menu.is_some() || !in_map(m, 0.0) {
        return;
    }
    let Some(kind) = game.build_choice else {
        if let Some(tw) = game.tower_at(m).map(|i| &game.towers[i]) {
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
    draw_range(m, &kind.stats([0; PATHS]), tint);
    draw_tower(&Tower::new(kind, m), m, game.time, 0.6, t);
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

fn draw_tower(tower: &Tower, c: Vec2, time: f32, alpha: f32, t: &Theme) {
    let color = with_alpha(tower.kind.color(), alpha);
    let dark = with_alpha(darken(tower.kind.color(), 0.45), alpha);
    let stone = Color::new(0.28, 0.28, 0.32, alpha);

    if t.night {
        // Lantern glow so towers stand out in the dark.
        draw_circle(
            c.x,
            c.y,
            TOWER_RADIUS + 14.0,
            Color::new(1.0, 0.85, 0.5, 0.07 * alpha),
        );
    }

    if tower.kind == TowerKind::Farm {
        let soil = Color::new(0.45, 0.30, 0.18, alpha);
        draw_rectangle(c.x - 18.0, c.y - 18.0, 36.0, 36.0, soil);
        for i in 0..3 {
            let y = c.y - 11.0 + i as f32 * 11.0;
            draw_line(c.x - 14.0, y, c.x + 4.0, y, 4.0, color);
        }
        let barn = Color::new(0.70, 0.20, 0.18, alpha);
        draw_rectangle(c.x + 6.0, c.y - 4.0, 10.0, 12.0, barn);
        draw_triangle(
            vec2(c.x + 4.0, c.y - 4.0),
            vec2(c.x + 18.0, c.y - 4.0),
            vec2(c.x + 11.0, c.y - 11.0),
            barn,
        );
        draw_rectangle_lines(
            c.x - 18.0,
            c.y - 18.0,
            36.0,
            36.0,
            2.0,
            Color::new(0.25, 0.16, 0.1, alpha),
        );
    } else {
        draw_circle(c.x, c.y, TOWER_RADIUS, stone);
        draw_circle_lines(
            c.x,
            c.y,
            TOWER_RADIUS,
            2.0,
            Color::new(0.15, 0.15, 0.18, alpha),
        );
    }

    let dir = vec2(tower.angle.cos(), tower.angle.sin());
    match tower.kind {
        TowerKind::Farm => {}
        TowerKind::Frost => {
            let pulse = (time * 4.0 + c.x).sin();
            let r = 11.0 + pulse;
            draw_poly(c.x, c.y, 6, r, time * 40.0, color);
            draw_poly_lines(c.x, c.y, 6, r, time * 40.0, 2.0, dark);
            draw_circle(c.x, c.y, 3.5, with_alpha(WHITE, alpha));
            if tower.since_shot < 0.15 {
                let k = tower.since_shot / 0.15;
                draw_circle_lines(
                    c.x,
                    c.y,
                    TOWER_RADIUS + k * 8.0,
                    2.0,
                    with_alpha(SKYBLUE, 1.0 - k),
                );
            }
        }
        _ => {
            let (len, width) = match tower.kind {
                TowerKind::Cannon => (16.0, 9.0),
                TowerKind::Sniper => (26.0, 4.0),
                _ => (17.0, 5.0),
            };
            let recoil = (1.0 - tower.since_shot / 0.12).max(0.0) * 4.0;
            let tip = c + dir * (len - recoil);
            draw_line(c.x, c.y, tip.x, tip.y, width + 2.0, dark);
            draw_line(c.x, c.y, tip.x, tip.y, width, color);
            draw_circle(c.x, c.y, 11.0, color);
            draw_circle_lines(c.x, c.y, 11.0, 2.0, dark);
            if tower.since_shot < 0.06 {
                let flash = c + dir * (len + 4.0);
                draw_circle(flash.x, flash.y, 5.0, with_alpha(YELLOW, 0.8));
            }
        }
    }

    if tower.tiers.iter().any(|&t| t > 0) {
        let label = tower.tier_label();
        let pos = c + vec2(0.0, TOWER_RADIUS + 7.0);
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
            }
        }
        if e.frozen() {
            // Encased in an ice block.
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

    draw_text(
        format!("Wave {}/{}", game.wave, game.max_wave()),
        250.0,
        32.0,
        28.0,
        t.text,
    );
    if game.sandbox {
        let r = Rect::new(420.0, 10.0, 110.0, 28.0);
        panel_rect(r, Color::new(0.85, 0.6, 0.1, 1.0), t.border);
        text_centered("SANDBOX", r.center(), 20, BLACK);
    }

    button(ui::settings_button(), "Settings", true, BLUE, t);
    if game.wave == 0 {
        button(
            ui::map_button(),
            &format!("Map: {} (M)", game.map.name),
            true,
            BLUE,
            t,
        );
    } else {
        text_right(
            &format!("Map: {}", game.map.name),
            ui::pause_button().x - 16.0,
            30.0,
            20,
            t.text_dim,
        );
    }
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

fn draw_sidebar(game: &Game, t: &Theme) {
    draw_rectangle(SIDEBAR_X, TOP_BAR, SIDEBAR_W, MAP_H, t.panel);
    draw_line(SIDEBAR_X, TOP_BAR, SIDEBAR_X, SCREEN_H, 2.0, t.border);

    match game.selected.and_then(|i| game.towers.get(i)) {
        Some(tower) => draw_tower_panel(game, tower, t),
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
    let icon = vec2(r.x + 28.0, r.y + r.h / 2.0);
    draw_circle(icon.x, icon.y, 16.0, kind.color());
    draw_circle_lines(icon.x, icon.y, 16.0, 2.0, BLACK);
    text_centered(&number.to_string(), icon, 20, BLACK);
    let name_color = if affordable { t.text } else { t.text_dim };
    draw_text(kind.name(), r.x + 54.0, r.y + 24.0, 24.0, name_color);
    let cost_color = if affordable { t.gold } else { t.bad };
    text_right(
        &format!("{}g", kind.cost()),
        r.x + r.w - 10.0,
        r.y + 24.0,
        22,
        cost_color,
    );
    draw_text(kind.description(), r.x + 54.0, r.y + 46.0, 16.0, t.text_dim);
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
        draw_text("SANDBOX", x, TOP_BAR + 424.0, 22.0, t.text);
        text_right(
            "shift = x5",
            SIDEBAR_X + SIDEBAR_W - 12.0,
            TOP_BAR + 424.0,
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

    let mut y = TOP_BAR + 420.0;
    let help = [
        "Click grass to place a tower.",
        "Shift+click places several.",
        "Click a tower to upgrade it.",
        "",
        "1-5  pick tower   Tab  target",
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

fn stat_lines(tower: &Tower) -> Vec<String> {
    let s = tower.stats();
    let mut lines = Vec::new();
    if tower.kind != TowerKind::Farm {
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
    match tower.kind {
        TowerKind::Arrow => lines.push(format!(
            "{} arrow(s), pierce {:.0} tiles",
            s.arrows,
            s.pierce / TILE
        )),
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
    if s.lives_per_wave > 0 {
        lines.push(format!("+{} lives per wave", s.lives_per_wave));
    }
    lines
}

fn draw_tower_panel(game: &Game, tower: &Tower, t: &Theme) {
    let x = SIDEBAR_X + 14.0;
    draw_text(
        tower.kind.name(),
        x,
        TOP_BAR + 32.0,
        30.0,
        darken(tower.kind.color(), if t.night { 1.0 } else { 0.75 }),
    );
    button(ui::close_button(), "x", true, GREY_BTN, t);

    let mut y = TOP_BAR + 58.0;
    for line in stat_lines(tower).iter().take(4) {
        draw_text(line, x, y, 18.0, t.text);
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

fn draw_overlay(game: &Game) {
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
        GameState::GameOver => format!(
            "You survived {} waves. Press R to restart",
            game.wave.saturating_sub(1)
        ),
        GameState::Victory => format!("All {MAX_WAVES} waves defeated! Press R to play again"),
    };
    text_centered(&sub, center + vec2(0.0, 30.0), 26, WHITE);
}

fn dim_screen() {
    draw_rectangle(0.0, 0.0, SCREEN_W, SCREEN_H, with_alpha(BLACK, 0.6));
}

fn draw_settings(game: &Game, t: &Theme) {
    dim_screen();
    let p = ui::settings_panel();
    panel_rect(p, t.panel, t.border);
    text_centered("Settings", vec2(p.center().x, p.y + 34.0), 32, t.text);

    let theme = if game.night {
        "Theme: Night"
    } else {
        "Theme: Day"
    };
    let items = [
        (theme, BLUE),
        ("Tower guide", BLUE),
        ("New game", GREEN_BTN),
        ("New sandbox game", Color::new(0.75, 0.5, 0.1, 1.0)),
        ("Resume (Esc)", GREY_BTN),
    ];
    for (i, (label, color)) in items.into_iter().enumerate() {
        button(ui::settings_item(i), label, true, color, t);
    }
    let mode = if game.sandbox { "sandbox" } else { "normal" };
    let note = format!("Playing {mode} on {}. New games restart.", game.map.name);
    text_centered(&note, vec2(p.center().x, p.y + p.h - 24.0), 18, t.text_dim);
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

    // Base tower summary.
    let header = Rect::new(p.x + 20.0, p.y + 104.0, p.w - 40.0, 112.0);
    panel_rect(header, t.panel_light, t.border);
    let sample = Tower::new(kind, Vec2::ZERO);
    let icon = vec2(header.x + 40.0, header.y + 40.0);
    draw_circle(icon.x, icon.y, 22.0, kind.color());
    draw_circle_lines(icon.x, icon.y, 22.0, 2.0, BLACK);
    draw_text(kind.name(), header.x + 76.0, header.y + 34.0, 30.0, t.text);
    draw_text(
        format!("{}g  -  {}", kind.cost(), kind.description()),
        header.x + 76.0,
        header.y + 58.0,
        20.0,
        t.text_dim,
    );
    let mut x = header.x + 76.0;
    for line in stat_lines(&sample) {
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
