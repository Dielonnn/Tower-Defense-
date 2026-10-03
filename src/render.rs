use macroquad::prelude::*;

use crate::enemy::EnemyKind;
use crate::game::{Game, GameState, base_position};
use crate::map::*;
use crate::tower::{MAX_LEVEL, Tower, TowerKind};
use crate::ui;
use crate::wave::MAX_WAVES;

const GRASS_A: Color = Color::new(0.33, 0.55, 0.28, 1.0);
const GRASS_B: Color = Color::new(0.30, 0.51, 0.26, 1.0);
const PATH_FILL: Color = Color::new(0.78, 0.66, 0.45, 1.0);
const PATH_EDGE: Color = Color::new(0.62, 0.50, 0.32, 1.0);
const PANEL: Color = Color::new(0.13, 0.14, 0.18, 1.0);
const PANEL_LIGHT: Color = Color::new(0.20, 0.22, 0.28, 1.0);
const TEXT: Color = Color::new(0.92, 0.92, 0.95, 1.0);
const TEXT_DIM: Color = Color::new(0.62, 0.64, 0.70, 1.0);

pub fn draw(game: &Game) {
    clear_background(PANEL);
    draw_map(game);
    draw_placement_preview(game);
    for tower in &game.towers {
        draw_tower(tower, game.time);
    }
    draw_enemies(game);
    draw_projectiles(game);
    draw_effects(game);
    draw_sidebar(game);
    draw_top_bar(game);
    draw_message(game);
    draw_overlay(game);
}

fn with_alpha(c: Color, a: f32) -> Color {
    Color::new(c.r, c.g, c.b, a)
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

fn button(rect: Rect, label: &str, enabled: bool, highlight: Color) {
    let mouse: Vec2 = mouse_position().into();
    let hover = enabled && rect.contains(mouse);
    let fill = match (enabled, hover) {
        (false, _) => PANEL_LIGHT,
        (true, true) => highlight,
        (true, false) => with_alpha(highlight, 0.75),
    };
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, fill);
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 2.0, with_alpha(BLACK, 0.4));
    let color = if enabled { WHITE } else { TEXT_DIM };
    text_centered(label, rect.center(), 22, color);
}

fn draw_map(game: &Game) {
    for row in 0..ROWS {
        for col in 0..COLS {
            let x = MAP_X + col as f32 * TILE;
            let y = MAP_Y + row as f32 * TILE;
            let color = if (row + col) % 2 == 0 {
                GRASS_A
            } else {
                GRASS_B
            };
            draw_rectangle(x, y, TILE, TILE, color);
        }
    }
    // Draw the path as thick rounded strokes so corners look continuous.
    let wps = &game.map.waypoints;
    for (width, color) in [(TILE, PATH_EDGE), (TILE - 8.0, PATH_FILL)] {
        for pair in wps.windows(2) {
            draw_line(pair[0].x, pair[0].y, pair[1].x, pair[1].y, width, color);
        }
        for p in wps.iter() {
            draw_rectangle(p.x - width / 2.0, p.y - width / 2.0, width, width, color);
        }
    }

    // Entry portal.
    let start = tile_center(0, 2);
    let pulse = (game.time * 3.0).sin() * 3.0;
    draw_circle_lines(start.x, start.y, 18.0 + pulse, 3.0, with_alpha(PURPLE, 0.8));

    // The base the player is defending.
    let base = base_position();
    draw_rectangle(
        base.x - 20.0,
        base.y - 18.0,
        40.0,
        36.0,
        Color::from_rgba(90, 90, 110, 255),
    );
    for i in 0..3 {
        let x = base.x - 20.0 + i as f32 * 15.0;
        draw_rectangle(
            x,
            base.y - 26.0,
            10.0,
            10.0,
            Color::from_rgba(90, 90, 110, 255),
        );
    }
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

fn draw_placement_preview(game: &Game) {
    let mouse: Vec2 = mouse_position().into();
    let Some((col, row)) = tile_at(mouse) else {
        return;
    };
    let center = tile_center(col, row);

    if let Some(kind) = game.build_choice {
        let ok = game.can_build(col, row) && game.gold >= kind.cost();
        let tint = if ok { WHITE } else { RED };
        let range = kind.stats(1).range;
        draw_circle(center.x, center.y, range, with_alpha(tint, 0.12));
        draw_circle_lines(center.x, center.y, range, 2.0, with_alpha(tint, 0.5));
        draw_rectangle(
            center.x - TILE / 2.0,
            center.y - TILE / 2.0,
            TILE,
            TILE,
            with_alpha(tint, 0.25),
        );
        draw_circle(center.x, center.y, 14.0, with_alpha(kind.color(), 0.6));
    } else if game.tower_at(col, row).is_some() {
        draw_rectangle_lines(
            center.x - TILE / 2.0,
            center.y - TILE / 2.0,
            TILE,
            TILE,
            2.0,
            with_alpha(WHITE, 0.6),
        );
    }

    if let Some(tower) = game.selected.and_then(|i| game.towers.get(i)) {
        let c = tower.center();
        let range = tower.stats().range;
        draw_circle(c.x, c.y, range, with_alpha(WHITE, 0.08));
        draw_circle_lines(c.x, c.y, range, 2.0, with_alpha(WHITE, 0.6));
    }
}

fn draw_tower(tower: &Tower, time: f32) {
    let c = tower.center();
    let color = tower.kind.color();
    let dark = Color::new(color.r * 0.45, color.g * 0.45, color.b * 0.45, 1.0);

    draw_rectangle(
        c.x - 19.0,
        c.y - 19.0,
        38.0,
        38.0,
        Color::from_rgba(70, 70, 80, 255),
    );
    draw_rectangle_lines(
        c.x - 19.0,
        c.y - 19.0,
        38.0,
        38.0,
        2.0,
        Color::from_rgba(40, 40, 48, 255),
    );

    let dir = vec2(tower.angle.cos(), tower.angle.sin());
    match tower.kind {
        TowerKind::Frost => {
            let pulse = (time * 4.0 + tower.col as f32).sin() * 2.0;
            draw_poly(c.x, c.y, 6, 13.0 + pulse * 0.5, time * 40.0, color);
            draw_poly_lines(c.x, c.y, 6, 13.0 + pulse * 0.5, time * 40.0, 2.0, dark);
            draw_circle(c.x, c.y, 4.0, WHITE);
        }
        _ => {
            let (len, width) = match tower.kind {
                TowerKind::Cannon => (16.0, 9.0),
                TowerKind::Sniper => (24.0, 4.0),
                _ => (17.0, 5.0),
            };
            // Barrel recoils briefly after each shot.
            let recoil = (1.0 - tower.since_shot / 0.12).max(0.0) * 4.0;
            let tip = c + dir * (len - recoil);
            draw_line(c.x, c.y, tip.x, tip.y, width + 2.0, dark);
            draw_line(c.x, c.y, tip.x, tip.y, width, color);
            draw_circle(c.x, c.y, 12.0, color);
            draw_circle_lines(c.x, c.y, 12.0, 2.0, dark);
            if tower.since_shot < 0.06 {
                let flash = c + dir * (len + 4.0);
                draw_circle(flash.x, flash.y, 5.0, with_alpha(YELLOW, 0.8));
            }
        }
    }

    for i in 0..tower.level {
        let x = c.x - 12.0 + i as f32 * 12.0;
        draw_circle(x, c.y + 16.0, 3.5, GOLD);
        draw_circle_lines(x, c.y + 16.0, 3.5, 1.0, BLACK);
    }
}

fn draw_enemies(game: &Game) {
    for e in &game.enemies {
        let r = e.kind.radius();
        let mut color = e.kind.color();
        if e.slowed() {
            color = Color::new(color.r * 0.5 + 0.2, color.g * 0.6 + 0.3, 1.0, 1.0);
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
        match p.kind {
            TowerKind::Arrow => {
                let dir = (p.pos - p.prev_pos).normalize_or_zero();
                let tail = p.pos - dir * 9.0;
                draw_line(
                    tail.x,
                    tail.y,
                    p.pos.x,
                    p.pos.y,
                    2.5,
                    Color::from_rgba(255, 230, 150, 255),
                );
            }
            TowerKind::Cannon => {
                draw_circle(p.pos.x, p.pos.y, 5.5, Color::from_rgba(40, 40, 40, 255));
            }
            TowerKind::Frost => {
                draw_circle(p.pos.x, p.pos.y, 5.0, with_alpha(SKYBLUE, 0.9));
                draw_circle(p.pos.x, p.pos.y, 2.5, WHITE);
            }
            TowerKind::Sniper => {
                let dir = (p.pos - p.prev_pos).normalize_or_zero();
                let tail = p.pos - dir * 26.0;
                draw_line(
                    tail.x,
                    tail.y,
                    p.pos.x,
                    p.pos.y,
                    2.0,
                    Color::from_rgba(230, 200, 255, 255),
                );
            }
        }
    }
}

fn draw_effects(game: &Game) {
    for e in &game.effects {
        let t = 1.0 - e.life / e.max_life;
        let r = e.radius * (0.4 + 0.6 * t);
        draw_circle(e.pos.x, e.pos.y, r, with_alpha(e.color, 0.25 * (1.0 - t)));
        draw_circle_lines(e.pos.x, e.pos.y, r, 2.0, with_alpha(e.color, 1.0 - t));
    }
    for f in &game.floaters {
        let a = f.life.min(1.0);
        text_centered(&f.text, f.pos + vec2(1.0, 1.0), 20, with_alpha(BLACK, a));
        text_centered(&f.text, f.pos, 20, with_alpha(f.color, a));
    }
}

fn draw_top_bar(game: &Game) {
    draw_rectangle(0.0, 0.0, SCREEN_W, TOP_BAR, PANEL);
    draw_line(0.0, TOP_BAR, SCREEN_W, TOP_BAR, 2.0, BLACK);

    draw_circle(24.0, 24.0, 10.0, GOLD);
    draw_circle_lines(24.0, 24.0, 10.0, 2.0, ORANGE);
    draw_text(game.gold.to_string(), 42.0, 32.0, 28.0, GOLD);

    let heart = vec2(150.0, 24.0);
    draw_circle(heart.x - 5.0, heart.y - 3.0, 6.0, RED);
    draw_circle(heart.x + 5.0, heart.y - 3.0, 6.0, RED);
    draw_triangle(
        vec2(heart.x - 11.0, heart.y - 1.0),
        vec2(heart.x + 11.0, heart.y - 1.0),
        vec2(heart.x, heart.y + 11.0),
        RED,
    );
    draw_text(game.lives.to_string(), 168.0, 32.0, 28.0, TEXT);

    draw_text(
        format!("Wave {}/{}", game.wave, MAX_WAVES),
        240.0,
        32.0,
        28.0,
        TEXT,
    );

    draw_text(
        "1-4 build  U upgrade  S sell  Space wave  F speed  P pause",
        400.0,
        30.0,
        18.0,
        TEXT_DIM,
    );

    let pause_label = if game.paused { "Resume" } else { "Pause" };
    button(
        ui::pause_button(),
        pause_label,
        true,
        Color::from_rgba(80, 90, 120, 255),
    );
    button(
        ui::speed_button(),
        &format!("Speed x{}", game.speed),
        true,
        Color::from_rgba(80, 90, 120, 255),
    );
}

fn draw_sidebar(game: &Game) {
    draw_rectangle(SIDEBAR_X, TOP_BAR, SIDEBAR_W, MAP_H, PANEL);
    draw_line(SIDEBAR_X, TOP_BAR, SIDEBAR_X, SCREEN_H, 2.0, BLACK);
    draw_text("TOWERS", SIDEBAR_X + 12.0, TOP_BAR + 24.0, 24.0, TEXT);

    let mouse: Vec2 = mouse_position().into();
    for (i, kind) in TowerKind::ALL.into_iter().enumerate() {
        let r = ui::tower_button(i);
        let affordable = game.gold >= kind.cost();
        let chosen = game.build_choice == Some(kind);
        let fill = if chosen {
            Color::from_rgba(70, 90, 130, 255)
        } else if r.contains(mouse) {
            Color::from_rgba(45, 50, 64, 255)
        } else {
            PANEL_LIGHT
        };
        draw_rectangle(r.x, r.y, r.w, r.h, fill);
        let border = if chosen {
            WHITE
        } else {
            with_alpha(BLACK, 0.5)
        };
        draw_rectangle_lines(r.x, r.y, r.w, r.h, 2.0, border);

        let icon = vec2(r.x + 30.0, r.y + r.h / 2.0);
        draw_circle(icon.x, icon.y, 16.0, kind.color());
        draw_circle_lines(icon.x, icon.y, 16.0, 2.0, BLACK);
        text_centered(&(i + 1).to_string(), icon, 20, BLACK);

        let name_color = if affordable { TEXT } else { TEXT_DIM };
        draw_text(kind.name(), r.x + 56.0, r.y + 24.0, 24.0, name_color);
        let cost_color = if affordable {
            GOLD
        } else {
            Color::from_rgba(180, 80, 80, 255)
        };
        let cost = format!("{}g", kind.cost());
        let cw = measure_text(&cost, None, 22, 1.0).width;
        draw_text(&cost, r.x + r.w - cw - 10.0, r.y + 24.0, 22.0, cost_color);
        draw_text(kind.description(), r.x + 56.0, r.y + 48.0, 16.0, TEXT_DIM);
    }

    draw_info_panel(game);

    let can_wave = game.can_start_wave();
    let label = if game.wave_active {
        format!("{} enemies left", game.enemies.len())
    } else if game.wave >= MAX_WAVES {
        "All waves done".to_string()
    } else {
        format!("Start wave {}", game.wave + 1)
    };
    button(
        ui::wave_button(),
        &label,
        can_wave,
        Color::from_rgba(60, 150, 80, 255),
    );
}

fn draw_info_panel(game: &Game) {
    let x = SIDEBAR_X + 14.0;
    let mut y = TOP_BAR + 350.0;
    let line = |y: &mut f32, text: &str, color: Color| {
        draw_text(text, x, *y, 19.0, color);
        *y += 19.0;
    };

    let selected = game.selected.and_then(|i| game.towers.get(i));
    let (kind, level) = match (selected, game.build_choice) {
        (Some(t), _) => (t.kind, t.level),
        (None, Some(k)) => (k, 1),
        (None, None) => {
            line(&mut y, "Pick a tower, then click", TEXT_DIM);
            line(&mut y, "on the grass to build it.", TEXT_DIM);
            line(&mut y, "Click a tower to upgrade", TEXT_DIM);
            line(&mut y, "or sell it.", TEXT_DIM);
            line(&mut y, "Shift+click builds many.", TEXT_DIM);
            return;
        }
    };

    let stats = kind.stats(level);
    draw_text(
        format!("{} - Lv {}", kind.name(), level),
        x,
        y,
        24.0,
        kind.color(),
    );
    y += 24.0;
    let next = (level < MAX_LEVEL).then(|| kind.stats(level + 1));
    // Shows "now -> next" for placed towers that can still be upgraded.
    let fmt = |now: f32, next: Option<f32>, prec: usize| match next {
        Some(n) if selected.is_some() => format!("{now:.prec$} -> {n:.prec$}"),
        _ => format!("{now:.prec$}"),
    };
    line(
        &mut y,
        &format!("Damage: {}", fmt(stats.damage, next.map(|s| s.damage), 0)),
        TEXT,
    );
    line(
        &mut y,
        &format!(
            "Range: {}",
            fmt(stats.range / TILE, next.map(|s| s.range / TILE), 1)
        ),
        TEXT,
    );
    line(
        &mut y,
        &format!("Fire rate: {:.1}/s", 1.0 / stats.cooldown),
        TEXT,
    );
    if stats.splash > 0.0 {
        line(&mut y, "Splash damage", ORANGE);
    }
    if stats.slow < 1.0 {
        line(
            &mut y,
            &format!(
                "Slows {:.0}% for {:.1}s",
                (1.0 - stats.slow) * 100.0,
                stats.slow_time
            ),
            SKYBLUE,
        );
    }

    if let Some(tower) = selected {
        let label = match tower.upgrade_cost() {
            Some(cost) => format!("Up {cost}g"),
            None => "Max level".to_string(),
        };
        let can_up = tower.upgrade_cost().is_some_and(|c| game.gold >= c);
        button(
            ui::upgrade_button(),
            &label,
            can_up,
            Color::from_rgba(60, 110, 170, 255),
        );
        button(
            ui::sell_button(),
            &format!("Sell {}g", tower.sell_value()),
            true,
            Color::from_rgba(170, 70, 60, 255),
        );
    }
}

fn draw_message(game: &Game) {
    let Some((text, t)) = &game.message else {
        return;
    };
    let a = t.min(1.0);
    let center = vec2(MAP_W / 2.0, TOP_BAR + 40.0);
    let m = measure_text(text, None, 32, 1.0);
    draw_rectangle(
        center.x - m.width / 2.0 - 16.0,
        center.y - 22.0,
        m.width + 32.0,
        44.0,
        with_alpha(BLACK, 0.55 * a),
    );
    text_centered(text, center, 32, with_alpha(WHITE, a));
}

fn draw_overlay(game: &Game) {
    let (title, color) = match game.state {
        GameState::Playing if game.paused => ("PAUSED", WHITE),
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
    text_centered(&sub, center + vec2(0.0, 30.0), 26, TEXT);
}
