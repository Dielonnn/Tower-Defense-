use macroquad::prelude::*;

use crate::enemy::EnemyKind;
use crate::game::{EffectShape, Game, GameState, Shot};
use crate::map::*;
use crate::tower::{PATHS, Stats, TIERS, Tower, TowerKind};
use crate::ui;
use crate::wave::MAX_WAVES;

const GRASS: Color = Color::new(0.32, 0.54, 0.27, 1.0);
const PATH_FILL: Color = Color::new(0.80, 0.68, 0.47, 1.0);
const PATH_EDGE: Color = Color::new(0.60, 0.48, 0.31, 1.0);
const PANEL: Color = Color::new(0.13, 0.14, 0.18, 1.0);
const PANEL_LIGHT: Color = Color::new(0.20, 0.22, 0.28, 1.0);
const PANEL_HOVER: Color = Color::new(0.26, 0.29, 0.37, 1.0);
const TEXT: Color = Color::new(0.92, 0.92, 0.95, 1.0);
const TEXT_DIM: Color = Color::new(0.62, 0.64, 0.70, 1.0);
const BAD: Color = Color::new(0.75, 0.32, 0.32, 1.0);

pub fn draw(game: &Game) {
    clear_background(PANEL);
    draw_map(game);
    draw_ranges(game);
    for tower in &game.towers {
        draw_tower(tower, tower.pos, game.time, 1.0);
    }
    draw_enemies(game);
    draw_projectiles(game);
    draw_effects(game);
    draw_placement_preview(game);
    draw_sidebar(game);
    draw_top_bar(game);
    draw_message(game);
    draw_overlay(game);
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

fn button(rect: Rect, label: &str, enabled: bool, highlight: Color) {
    let hover = enabled && rect.contains(mouse());
    let fill = match (enabled, hover) {
        (false, _) => PANEL_LIGHT,
        (true, true) => highlight,
        (true, false) => darken(highlight, 0.8),
    };
    panel_rect(rect, fill, with_alpha(BLACK, 0.4));
    let color = if enabled { WHITE } else { TEXT_DIM };
    text_centered(label, rect.center(), 22, color);
}

fn draw_map(game: &Game) {
    draw_rectangle(MAP_X, MAP_Y, MAP_W, MAP_H, GRASS);
    for &(p, r, shade) in &game.map.decor {
        let c = if shade > 0.5 {
            Color::new(0.36, 0.60, 0.30, 1.0)
        } else {
            Color::new(0.28, 0.48, 0.24, 1.0)
        };
        draw_circle(p.x, p.y, r, c);
    }

    // Thick strokes with round joints so corners and crossings look clean.
    let wps = &game.map.waypoints;
    for (width, color) in [(PATH_WIDTH + 8.0, PATH_EDGE), (PATH_WIDTH, PATH_FILL)] {
        for pair in wps.windows(2) {
            draw_line(pair[0].x, pair[0].y, pair[1].x, pair[1].y, width, color);
        }
        for p in wps {
            draw_circle(p.x, p.y, width / 2.0, color);
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

    let base = game.map.base;
    let stone = Color::from_rgba(90, 90, 110, 255);
    draw_rectangle(base.x - 20.0, base.y - 18.0, 40.0, 36.0, stone);
    for i in 0..3 {
        let x = base.x - 20.0 + i as f32 * 15.0;
        draw_rectangle(x, base.y - 26.0, 10.0, 10.0, stone);
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

fn draw_range(pos: Vec2, stats: &Stats, tint: Color) {
    let range = if stats.range.is_finite() {
        stats.range
    } else {
        stats.bounty_range
    };
    if range > 0.0 {
        draw_circle(pos.x, pos.y, range, with_alpha(tint, 0.10));
        draw_circle_lines(pos.x, pos.y, range, 2.0, with_alpha(tint, 0.55));
    }
}

fn draw_ranges(game: &Game) {
    if let Some(tower) = game.selected.and_then(|i| game.towers.get(i)) {
        draw_range(tower.pos, &tower.stats(), WHITE);
    }
}

fn draw_placement_preview(game: &Game) {
    let m = mouse();
    if !in_map(m, 0.0) {
        return;
    }
    let Some(kind) = game.build_choice else {
        if let Some(t) = game.tower_at(m).map(|i| &game.towers[i]) {
            draw_circle_lines(
                t.pos.x,
                t.pos.y,
                TOWER_RADIUS + 4.0,
                2.0,
                with_alpha(WHITE, 0.7),
            );
        }
        return;
    };
    let ok = game.can_place(m) && game.gold >= kind.cost();
    let tint = if ok { WHITE } else { RED };
    draw_range(m, &kind.stats([0; PATHS]), tint);
    draw_tower(&Tower::new(kind, m), m, game.time, 0.6);
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

fn draw_tower(tower: &Tower, c: Vec2, time: f32, alpha: f32) {
    let color = with_alpha(tower.kind.color(), alpha);
    let dark = with_alpha(darken(tower.kind.color(), 0.45), alpha);
    let stone = Color::new(0.28, 0.28, 0.32, alpha);

    if tower.kind == TowerKind::Farm {
        let soil = Color::new(0.45, 0.30, 0.18, alpha);
        draw_rectangle(c.x - 18.0, c.y - 18.0, 36.0, 36.0, soil);
        for i in 0..3 {
            let y = c.y - 11.0 + i as f32 * 11.0;
            draw_line(c.x - 14.0, y, c.x + 4.0, y, 4.0, color);
        }
        // Little barn.
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
            // Pulse flash right after firing.
            if tower.since_shot < 0.15 {
                let t = tower.since_shot / 0.15;
                draw_circle_lines(
                    c.x,
                    c.y,
                    TOWER_RADIUS + t * 8.0,
                    2.0,
                    with_alpha(SKYBLUE, 1.0 - t),
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
            color = Color::new(0.85, 0.95, 1.0, 1.0);
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
        let t = 1.0 - e.life / e.max_life;
        match e.shape {
            EffectShape::Ring(radius) => {
                let r = radius * (0.4 + 0.6 * t);
                draw_circle(e.pos.x, e.pos.y, r, with_alpha(e.color, 0.25 * (1.0 - t)));
                draw_circle_lines(e.pos.x, e.pos.y, r, 2.0, with_alpha(e.color, 1.0 - t));
            }
            EffectShape::Line(to) => {
                draw_line(
                    e.pos.x,
                    e.pos.y,
                    to.x,
                    to.y,
                    2.0,
                    with_alpha(e.color, 1.0 - t),
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

fn draw_top_bar(game: &Game) {
    draw_rectangle(0.0, 0.0, SCREEN_W, TOP_BAR, PANEL);
    draw_line(0.0, TOP_BAR, SCREEN_W, TOP_BAR, 2.0, BLACK);

    draw_circle(24.0, 24.0, 10.0, GOLD);
    draw_circle_lines(24.0, 24.0, 10.0, 2.0, ORANGE);
    draw_text(game.gold.to_string(), 42.0, 32.0, 28.0, GOLD);

    let heart = vec2(160.0, 24.0);
    draw_circle(heart.x - 5.0, heart.y - 3.0, 6.0, RED);
    draw_circle(heart.x + 5.0, heart.y - 3.0, 6.0, RED);
    draw_triangle(
        vec2(heart.x - 11.0, heart.y - 1.0),
        vec2(heart.x + 11.0, heart.y - 1.0),
        vec2(heart.x, heart.y + 11.0),
        RED,
    );
    draw_text(game.lives.to_string(), 178.0, 32.0, 28.0, TEXT);

    draw_text(
        format!("Wave {}/{}", game.wave, MAX_WAVES),
        250.0,
        32.0,
        28.0,
        TEXT,
    );

    let blue = Color::from_rgba(80, 90, 120, 255);
    if game.wave == 0 {
        button(
            ui::map_button(),
            &format!("Map: {} (M)", game.map.name),
            true,
            blue,
        );
    } else {
        text_right(
            &format!("Map: {}", game.map.name),
            ui::pause_button().x - 16.0,
            30.0,
            20,
            TEXT_DIM,
        );
    }
    let pause_label = if game.paused { "Resume" } else { "Pause" };
    button(ui::pause_button(), pause_label, true, blue);
    button(
        ui::speed_button(),
        &format!("Speed x{}", game.speed),
        true,
        blue,
    );
}

fn draw_sidebar(game: &Game) {
    draw_rectangle(SIDEBAR_X, TOP_BAR, SIDEBAR_W, MAP_H, PANEL);
    draw_line(SIDEBAR_X, TOP_BAR, SIDEBAR_X, SCREEN_H, 2.0, BLACK);

    match game.selected.and_then(|i| game.towers.get(i)) {
        Some(tower) => draw_tower_panel(game, tower),
        None => draw_build_panel(game),
    }

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
        game.can_start_wave(),
        Color::from_rgba(60, 150, 80, 255),
    );
    let (auto_label, auto_color) = if game.auto_play {
        ("Auto ON", Color::from_rgba(60, 150, 80, 255))
    } else {
        ("Auto OFF", Color::from_rgba(90, 95, 110, 255))
    };
    button(ui::auto_button(), auto_label, true, auto_color);
}

fn draw_build_panel(game: &Game) {
    draw_text("TOWERS", SIDEBAR_X + 12.0, TOP_BAR + 26.0, 24.0, TEXT);

    for (i, kind) in TowerKind::ALL.into_iter().enumerate() {
        let r = ui::tower_button(i);
        let affordable = game.gold >= kind.cost();
        let chosen = game.build_choice == Some(kind);
        let fill = if chosen {
            Color::from_rgba(70, 90, 130, 255)
        } else if r.contains(mouse()) {
            PANEL_HOVER
        } else {
            PANEL_LIGHT
        };
        let border = if chosen {
            WHITE
        } else {
            with_alpha(BLACK, 0.5)
        };
        panel_rect(r, fill, border);

        let icon = vec2(r.x + 28.0, r.y + r.h / 2.0);
        draw_circle(icon.x, icon.y, 16.0, kind.color());
        draw_circle_lines(icon.x, icon.y, 16.0, 2.0, BLACK);
        text_centered(&(i + 1).to_string(), icon, 20, BLACK);

        let name_color = if affordable { TEXT } else { TEXT_DIM };
        draw_text(kind.name(), r.x + 54.0, r.y + 24.0, 24.0, name_color);
        let cost_color = if affordable { GOLD } else { BAD };
        text_right(
            &format!("{}g", kind.cost()),
            r.x + r.w - 10.0,
            r.y + 24.0,
            22,
            cost_color,
        );
        draw_text(kind.description(), r.x + 54.0, r.y + 46.0, 16.0, TEXT_DIM);
    }

    let x = SIDEBAR_X + 14.0;
    let mut y = TOP_BAR + 420.0;
    if game.build_choice.is_some() {
        button(
            ui::cancel_button(),
            "Cancel placing (Esc)",
            true,
            Color::from_rgba(170, 70, 60, 255),
        );
    }
    let help = [
        "Click grass to place a tower.",
        "Shift+click places several.",
        "Click a tower to upgrade it.",
        "",
        "1-5  pick tower",
        ", . /  upgrade paths 1-3",
        "S  sell     Space  next wave",
        "A  auto waves   F  speed",
        "P  pause    Esc  cancel",
    ];
    for line in help {
        draw_text(line, x, y, 18.0, TEXT_DIM);
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
            if s.burn_dps > 0.0 {
                lines.push(format!("Burn {:.0}/s for {:.0}s", s.burn_dps, s.burn_time));
            }
            if s.stun_time > 0.0 {
                lines.push(format!("Stun {:.1}s", s.stun_time));
            }
        }
        TowerKind::Frost => {
            lines.push(format!(
                "Slow {:.0}% for {:.1}s",
                (1.0 - s.slow) * 100.0,
                s.slow_time
            ));
            if s.freeze_time > 0.0 || s.brittle > 0.0 {
                lines.push(format!(
                    "Freeze {:.0}s  Brittle +{:.0}%",
                    s.freeze_time,
                    s.brittle * 100.0
                ));
            }
        }
        TowerKind::Sniper => {
            if s.bounces > 0 {
                lines.push(format!("Bounces {}", s.bounces));
            }
            if s.boss_mult > 1.0 {
                lines.push(format!("{:.0}x damage vs bosses", s.boss_mult));
            }
        }
        TowerKind::Farm => {}
    }
    if s.income > 0 {
        lines.push(format!("Income {}g per wave", s.income));
    }
    if s.kill_bounty > 0 {
        lines.push(format!("+{}g per kill nearby", s.kill_bounty));
    }
    if s.lives_per_wave > 0 {
        lines.push(format!("+{} lives per wave", s.lives_per_wave));
    }
    lines
}

fn draw_tower_panel(game: &Game, tower: &Tower) {
    let x = SIDEBAR_X + 14.0;
    draw_text(
        tower.kind.name(),
        x,
        TOP_BAR + 32.0,
        30.0,
        tower.kind.color(),
    );
    button(
        ui::close_button(),
        "x",
        true,
        Color::from_rgba(90, 95, 110, 255),
    );

    let mut y = TOP_BAR + 58.0;
    for line in stat_lines(tower).iter().take(5) {
        draw_text(line, x, y, 18.0, TEXT);
        y += 18.0;
    }

    let names = tower.kind.path_names();
    for (path, name) in names.iter().enumerate() {
        let r = ui::path_card(path);
        let tier = tower.tiers[path];
        let next = tower.next_upgrade(path);
        let open = tower.path_open(path);
        let affordable = tower.upgrade_cost(path).is_some_and(|c| game.gold >= c);
        let hover = r.contains(mouse());
        let fill = if open && affordable && hover {
            PANEL_HOVER
        } else {
            PANEL_LIGHT
        };
        panel_rect(r, fill, with_alpha(BLACK, 0.5));

        draw_text(
            format!("{} ({})", name, ui::PATH_KEY_LABELS[path]),
            r.x + 10.0,
            r.y + 18.0,
            17.0,
            TEXT_DIM,
        );
        for t in 0..TIERS {
            let px = r.x + r.w - 14.0 - (TIERS - 1 - t) as f32 * 14.0;
            let filled = t < tier;
            let c = if filled {
                GREEN
            } else {
                with_alpha(WHITE, 0.15)
            };
            draw_rectangle(px - 5.0, r.y + 8.0, 10.0, 10.0, c);
        }

        match next {
            None => {
                draw_text("MAXED", r.x + 10.0, r.y + 46.0, 24.0, GOLD);
            }
            Some(u) => {
                let name_color = if open { TEXT } else { TEXT_DIM };
                draw_text(u.name, r.x + 10.0, r.y + 44.0, 22.0, name_color);
                draw_text(u.desc, r.x + 10.0, r.y + 64.0, 16.0, TEXT_DIM);
                if open {
                    let c = if affordable { GOLD } else { BAD };
                    text_right(&format!("{}g", u.cost), r.x + r.w - 10.0, r.y + 78.0, 20, c);
                } else {
                    text_right("Locked", r.x + r.w - 10.0, r.y + 78.0, 18, BAD);
                }
            }
        }
    }

    button(
        ui::sell_button(),
        &format!("Sell for {}g (S)", tower.sell_value()),
        true,
        Color::from_rgba(170, 70, 60, 255),
    );

    let mut y = ui::sell_button().y + ui::sell_button().h + 22.0;
    for line in [
        "Only 2 paths per tower, and",
        "only one can go past tier 2.",
    ] {
        draw_text(line, x, y, 17.0, TEXT_DIM);
        y += 18.0;
    }
}

fn draw_message(game: &Game) {
    let Some((text, t)) = &game.message else {
        return;
    };
    let a = t.min(1.0);
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
