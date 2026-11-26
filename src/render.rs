use crate::game::{
    COLS, Effect, EffectKind, FloatKind, Game, MAX_LEVELS, Plant, PlantKind, ROWS, Status, Zombie,
    level_config,
};
use crate::pixel_art::{Sprite, Sprites};
use crate::records::{Improvement, Records};
use macroquad::prelude::*;
use std::f32::consts::{PI, TAU};

const TOP_PAD: f32 = 168.0;
const SCREEN_PAD: f32 = 48.0;
const BOTTOM_RESERVE: f32 = 310.0;
const MAX_CELL: f32 = 100.0;
const SELECTED_LABEL_Y: f32 = 99.0;
const NOTICE_BOTTOM_OFFSET: f32 = 38.0;
const HINT_BOTTOM_OFFSET: f32 = 20.0;

// Silkscreen is drawn on an 8px grid, so keeping sizes at multiples of 8
// avoids uneven, blurry pixels. The HUD at 16px still fits a 1280px window.
const FONT_SMALL: f32 = 16.0;
const FONT_BODY: f32 = 24.0;
const FONT_TITLE: f32 = 32.0;

#[derive(Clone, Copy, Debug)]
pub struct Layout {
    pub cell: f32,
    pub board_x: f32,
    pub board_y: f32,
    pub board_w: f32,
    pub board_h: f32,
    pub card_w: f32,
    pub card_h: f32,
    pub card_gap: f32,
    pub cards_x: f32,
    pub cards_y: f32,
}

pub fn layout_for(w: f32, h: f32) -> Layout {
    let cell = ((w - SCREEN_PAD * 2.0) / COLS as f32)
        .min((h - BOTTOM_RESERVE) / ROWS as f32)
        .min(MAX_CELL)
        .max(24.0);
    let board_w = cell * COLS as f32;
    let board_h = cell * ROWS as f32;
    let board_x = (w - board_w) / 2.0;
    let board_y = TOP_PAD;
    let card_gap = 14.0;
    let card_w = (board_w - card_gap * 2.0) / 3.0;
    let cards_w = card_w * 3.0 + card_gap * 2.0;
    Layout {
        cell,
        board_x,
        board_y,
        board_w,
        board_h,
        card_w,
        card_h: 40.0,
        card_gap,
        cards_x: (w - cards_w) / 2.0,
        cards_y: 116.0,
    }
}

pub fn layout() -> Layout {
    layout_for(screen_width(), screen_height())
}

pub fn cell_at(l: &Layout, x: f32, y: f32) -> Option<(usize, usize)> {
    if x < l.board_x || y < l.board_y || x >= l.board_x + l.board_w || y >= l.board_y + l.board_h {
        return None;
    }
    // Small epsilon keeps exact tile boundaries from rounding down a cell.
    let c = ((x - l.board_x + 0.01) / l.cell) as usize;
    let r = ((y - l.board_y + 0.01) / l.cell) as usize;
    if c < COLS && r < ROWS {
        Some((r, c))
    } else {
        None
    }
}

pub fn card_at(l: &Layout, x: f32, y: f32) -> Option<PlantKind> {
    if x < l.cards_x || y < l.cards_y || y >= l.cards_y + l.card_h {
        return None;
    }
    let step = l.card_w + l.card_gap;
    let i = ((x - l.cards_x) / step) as usize;
    let inside = x - l.cards_x - i as f32 * step <= l.card_w;
    if inside {
        PlantKind::ALL.get(i).copied()
    } else {
        None
    }
}

fn col(r: u8, g: u8, b: u8) -> Color {
    Color::from_rgba(r, g, b, 255)
}

const GREEN: Color = Color::new(0.29, 0.66, 0.30, 1.0);
const YELLOW: Color = Color::new(1.0, 0.84, 0.26, 1.0);
const RED: Color = Color::new(0.95, 0.30, 0.28, 1.0);
const WHITE: Color = Color::new(0.92, 0.94, 0.90, 1.0);
const GREY: Color = Color::new(0.55, 0.60, 0.58, 1.0);
const BROWN: Color = Color::new(0.55, 0.36, 0.18, 1.0);

fn text_centered(s: &str, cx: f32, y: f32, size: f32, color: Color) {
    let width = measure_text(s, None, size as u16, 1.0).width;
    draw_text(s, cx - width / 2.0, y, size, color);
}

pub(crate) fn fmt_time(seconds: f32) -> String {
    let s = seconds.max(0.0) as u32;
    format!("{:02}:{:02}", s / 60, s % 60)
}

fn health_bar(x: f32, y: f32, w: f32, ratio: f32) {
    if ratio >= 0.999 {
        return;
    }
    let h = 5.0;
    draw_rectangle(x, y, w, h, col(30, 30, 30));
    let fill = (w * ratio.clamp(0.0, 1.0)).max(2.0);
    let c = if ratio > 0.6 {
        GREEN
    } else if ratio > 0.3 {
        YELLOW
    } else {
        RED
    };
    draw_rectangle(x, y, fill, h, c);
}

const SHOOT_COLORS: [Color; 2] = [GREEN, YELLOW];
const HIT_COLORS: [Color; 3] = [RED, YELLOW, WHITE];
const PLANT_COLORS: [Color; 2] = [GREEN, BROWN];
const SUN_COLORS: [Color; 2] = [YELLOW, WHITE];
const EAT_COLORS: [Color; 2] = [GREEN, BROWN];
const DEATH_COLORS: [Color; 3] = [GREY, GREEN, RED];

struct ParticleStyle {
    count: usize,
    angle: f32,
    spread: f32,
    speed: f32,
    gravity: f32,
    size: f32,
    colors: &'static [Color],
}

fn particle_style(kind: EffectKind) -> ParticleStyle {
    match kind {
        // Pea muzzle flash: a tight puff toward the right.
        EffectKind::Shoot => ParticleStyle {
            count: 5,
            angle: 0.0,
            spread: 0.9,
            speed: 3.2,
            gravity: 0.0,
            size: 0.10,
            colors: &SHOOT_COLORS,
        },
        // Pea impact: sparks fly outward and fall a little.
        EffectKind::Hit => ParticleStyle {
            count: 8,
            angle: 0.0,
            spread: TAU,
            speed: 2.0,
            gravity: 1.5,
            size: 0.12,
            colors: &HIT_COLORS,
        },
        EffectKind::Plant => ParticleStyle {
            count: 10,
            angle: -PI / 2.0,
            spread: 2.4,
            speed: 1.6,
            gravity: 3.0,
            size: 0.12,
            colors: &PLANT_COLORS,
        },
        EffectKind::Sun => ParticleStyle {
            count: 9,
            angle: -PI / 2.0,
            spread: 2.0,
            speed: 1.3,
            gravity: -0.5,
            size: 0.10,
            colors: &SUN_COLORS,
        },
        // Chewing throws little leaf crumbs up and to the left.
        EffectKind::Eat => ParticleStyle {
            count: 6,
            angle: -2.2,
            spread: 1.4,
            speed: 1.1,
            gravity: 2.5,
            size: 0.09,
            colors: &EAT_COLORS,
        },
        EffectKind::Death => ParticleStyle {
            count: 12,
            angle: 0.0,
            spread: TAU,
            speed: 1.8,
            gravity: 1.2,
            size: 0.14,
            colors: &DEATH_COLORS,
        },
    }
}

fn hash01(n: u32) -> f32 {
    let mut x = n;
    x ^= x >> 16;
    x = x.wrapping_mul(0x7feb_352d);
    x ^= x >> 15;
    x = x.wrapping_mul(0x846c_a68b);
    x ^= x >> 16;
    x as f32 / u32::MAX as f32
}

fn effect_seed(e: &Effect) -> u32 {
    let sx = ((e.x * 131.0) as i32).unsigned_abs();
    let sy = ((e.y * 197.0) as i32).unsigned_abs();
    sx ^ sy.rotate_left(7) ^ (e.kind as u32).wrapping_mul(0x9e37_79b9)
}

/// Deterministic particle drift in tile units. No mutable particle list is
/// needed: age alone decides where each dot is, which keeps the effect cheap.
fn particle_offset(e: &Effect, index: usize) -> (f32, f32) {
    let style = particle_style(e.kind);
    let age = (e.duration - e.life).max(0.0);
    let progress = (age / e.duration).clamp(0.0, 1.0);
    let seed = effect_seed(e);
    let jitter = hash01(seed.wrapping_add((index as u32).wrapping_mul(0x9e37_79b9))) - 0.5;
    let variation = 0.7 + hash01(seed.wrapping_add(index as u32 + 991)) * 0.6;
    let angle = style.angle + jitter * style.spread;
    let distance = style.speed * variation * age * (1.0 - progress * 0.35);
    (
        angle.cos() * distance,
        angle.sin() * distance + style.gravity * age * age,
    )
}

fn with_alpha(color: Color, alpha: f32) -> Color {
    Color::new(color.r, color.g, color.b, alpha.clamp(0.0, 1.0))
}

fn draw_particles(e: &Effect, l: &Layout) {
    let style = particle_style(e.kind);
    let age = (e.duration - e.life).max(0.0);
    let progress = (age / e.duration).clamp(0.0, 1.0);
    let alpha = (1.0 - progress).powf(1.4) * 0.95;

    for i in 0..style.count {
        let (dx, dy) = particle_offset(e, i);
        let px = l.board_x + (e.x + dx) * l.cell;
        let py = l.board_y + (e.y + dy) * l.cell;
        let size = (style.size * (1.0 - progress * 0.4) * l.cell)
            .max(2.0)
            .round();
        let color = style.colors[i % style.colors.len()];
        draw_rectangle(
            px - size / 2.0,
            py - size / 2.0,
            size,
            size,
            with_alpha(color, alpha),
        );
    }

    // Bright core for the first third of the burst makes hits feel snappy.
    if progress < 0.35 {
        let core_alpha = (1.0 - progress / 0.35) * 0.7;
        let core = (l.cell * 0.16 * (1.0 - progress / 0.35)).max(2.0).round();
        draw_rectangle(
            l.board_x + e.x * l.cell - core / 2.0,
            l.board_y + e.y * l.cell - core / 2.0,
            core,
            core,
            with_alpha(WHITE, core_alpha),
        );
    }
}

fn plant_sprite(kind: PlantKind) -> Sprite {
    match kind {
        PlantKind::Sunflower => Sprite::Sunflower,
        PlantKind::Peashooter => Sprite::Peashooter,
        PlantKind::Wallnut => Sprite::Wallnut,
    }
}

fn draw_plant(sprites: &Sprites, p: &Plant, x: f32, y: f32, cell: f32, t: f32) {
    let bob = (t * 2.0 + x * 0.01).sin() * cell * 0.015;
    let pop = (p.flash / 0.32).clamp(0.0, 1.0);
    let (dx, dy, scale) = match p.kind {
        PlantKind::Sunflower => (0.0, -pop * cell * 0.06, 1.0 + pop * 0.14),
        PlantKind::Peashooter => (-pop * cell * 0.10, 0.0, 1.0),
        PlantKind::Wallnut => (0.0, 0.0, 1.0),
    };
    sprites.draw_centered(
        plant_sprite(p.kind),
        x + cell / 2.0 + dx,
        y + cell / 2.0 + dy + bob,
        cell * scale,
    );

    // Wallnut cracks as it takes damage.
    let ratio = p.hp / p.kind.max_hp();
    if ratio < 0.6 && p.kind == PlantKind::Wallnut {
        draw_line(
            x + cell * 0.38,
            y + cell * 0.30,
            x + cell * 0.55,
            y + cell * 0.52,
            2.0,
            col(45, 28, 16),
        );
    }
    if ratio < 0.3 && p.kind == PlantKind::Wallnut {
        draw_line(
            x + cell * 0.58,
            y + cell * 0.28,
            x + cell * 0.42,
            y + cell * 0.62,
            2.0,
            col(45, 28, 16),
        );
    }
    health_bar(x + 5.0, y + 4.0, cell - 10.0, ratio);
}

fn draw_zombie(sprites: &Sprites, z: &Zombie, x: f32, y: f32, cell: f32, t: f32) {
    let stride = if z.walking {
        (z.walk_phase * PI).sin().abs()
    } else {
        0.0
    };
    let frame = if z.walking && z.walk_phase.floor() % 2.0 == 1.0 {
        Sprite::ZombieStep
    } else {
        Sprite::Zombie
    };
    let bob = if z.walking {
        -stride * cell * 0.05
    } else {
        (t * 10.0 + x * 0.02).sin() * cell * 0.012
    };

    // A small shadow keeps the hopping sprite grounded.
    draw_rectangle(
        x + cell * 0.25,
        y + cell * 0.90,
        cell * 0.50,
        cell * 0.06,
        Color::new(0.0, 0.0, 0.0, 0.25),
    );
    sprites.draw_box(frame, x, y + bob, cell);
    health_bar(x + 5.0, y + 4.0, cell - 10.0, z.hp / z.max_hp);
}

fn draw_sun(sprites: &Sprites, cx: f32, cy: f32, cell: f32, t: f32) {
    let pulse = 1.0 + (t * 4.0).sin() * 0.06;
    sprites.draw_centered(Sprite::Sun, cx, cy, cell * 0.9 * pulse);
}

/// Show what would be planted where. Mouse wins when it is over the board;
/// otherwise the keyboard cursor is used.
fn draw_ghost(sprites: &Sprites, g: &Game, l: &Layout, hover: Option<(usize, usize)>) {
    if g.status != Status::Playing {
        return;
    }
    let (r, c) = hover.unwrap_or(g.cursor);
    if r >= ROWS || c >= COLS {
        return;
    }
    let x = l.board_x + c as f32 * l.cell;
    let y = l.board_y + r as f32 * l.cell;
    let occupied = g.plants.iter().any(|p| p.row == r && p.col == c);
    let affordable = g.sun >= g.selected.cost();
    let ok = !occupied && affordable;
    let tint = if ok { GREEN } else { RED };

    draw_rectangle(x, y, l.cell, l.cell, with_alpha(tint, 0.16));
    draw_rectangle_lines(x, y, l.cell, l.cell, 2.0, with_alpha(tint, 0.70));
    sprites.draw_ghost(
        plant_sprite(g.selected),
        x + l.cell / 2.0,
        y + l.cell / 2.0,
        l.cell * 0.88,
        if ok { 0.55 } else { 0.28 },
    );
}

fn draw_lawnmowers(sprites: &Sprites, g: &Game, l: &Layout) {
    for (row, ready) in g.lawnmowers.iter().enumerate() {
        if *ready {
            sprites.draw_box(
                Sprite::Lawnmower,
                l.board_x + l.cell * 0.06,
                l.board_y + row as f32 * l.cell,
                l.cell * 0.46,
            );
        }
    }
    for run in &g.mower_runs {
        if run.row >= ROWS {
            continue;
        }
        sprites.draw_box(
            Sprite::Lawnmower,
            l.board_x + run.x * l.cell,
            l.board_y + run.row as f32 * l.cell,
            l.cell * 0.85,
        );
    }
}

fn draw_floaters(g: &Game, l: &Layout) {
    for f in &g.floaters {
        let age = (f.duration - f.life).max(0.0);
        let progress = (age / f.duration).clamp(0.0, 1.0);
        let text = format!("{:+}", f.value);
        let x = l.board_x + f.x * l.cell;
        let y = l.board_y + (f.y - progress * 0.9) * l.cell;
        let color = match f.kind {
            FloatKind::Sun => YELLOW,
            FloatKind::Spend => RED,
            FloatKind::Kill => GREEN,
        };
        text_centered(&text, x, y, FONT_SMALL, with_alpha(color, 1.0 - progress));
    }
}

fn draw_wave_warning(g: &Game, l: &Layout, t: f32) {
    let Some((msg, life)) = &g.wave_warning else {
        return;
    };
    let alpha = (life / 1.2).clamp(0.25, 1.0);
    let pulse = 1.0 + (t * 12.0).sin() * 0.02;
    let text_w = measure_text(msg, None, FONT_BODY as u16, 1.0).width;
    let w = (text_w + 56.0) * pulse;
    let h = 46.0;
    let cx = screen_width() / 2.0;
    let cy = l.board_y + l.board_h * 0.18;
    draw_rectangle(
        cx - w / 2.0,
        cy - h / 2.0,
        w,
        h,
        with_alpha(col(18, 18, 26), 0.82),
    );
    draw_rectangle_lines(
        cx - w / 2.0,
        cy - h / 2.0,
        w,
        h,
        3.0,
        with_alpha(YELLOW, alpha),
    );
    text_centered(
        msg,
        cx,
        cy + FONT_BODY * 0.35,
        FONT_BODY,
        with_alpha(YELLOW, alpha),
    );
}

fn draw_background(l: &Layout) {
    let w = screen_width();
    let h = screen_height();
    draw_rectangle(0.0, 0.0, w, h, col(16, 22, 30));
    draw_rectangle(0.0, l.board_y + l.board_h * 0.55, w, h, col(20, 40, 32));
    draw_rectangle(0.0, l.board_y + l.board_h, w, h, col(26, 34, 30));
}

fn draw_cards(sprites: &Sprites, g: &Game, l: &Layout) {
    for (i, kind) in PlantKind::ALL.iter().enumerate() {
        let x = l.cards_x + i as f32 * (l.card_w + l.card_gap);
        let selected = *kind == g.selected;
        let bg = if selected {
            col(70, 76, 38)
        } else {
            col(32, 42, 40)
        };
        draw_rectangle(x, l.cards_y, l.card_w, l.card_h, bg);
        draw_rectangle_lines(
            x,
            l.cards_y,
            l.card_w,
            l.card_h,
            2.0,
            if selected { YELLOW } else { GREY },
        );
        let icon_x = x + l.card_h * 0.45;
        let icon_y = l.cards_y + l.card_h * 0.47;
        let icon = l.card_h * 0.7;
        sprites.draw_centered(plant_sprite(*kind), icon_x, icon_y, icon);
        let label = format!("{} {}", kind.name(), kind.cost());
        let tx = x + l.card_h * 0.85 + (l.card_w - l.card_h * 0.85) / 2.0;
        draw_text(
            &label,
            tx - measure_text(&label, None, FONT_SMALL as u16, 1.0).width / 2.0,
            l.cards_y + l.card_h * 0.60,
            FONT_SMALL,
            if selected { YELLOW } else { WHITE },
        );
    }
}

fn draw_hud(g: &Game, l: &Layout, records: &Records, audio_on: bool) {
    let w = screen_width();
    let cfg = level_config(g.level);
    let best = records.get(g.level);
    let best_text = match best.best_time {
        Some(t) => format!("{} / {} / *{}", fmt_time(t), best.best_score, best.stars),
        None => "--:-- / 0 / *0".to_string(),
    };
    // draw_text takes the baseline as y, so keep the title's glyphs on screen.
    text_centered("PLANTS vs ZOMBIES", w / 2.0, 48.0, FONT_TITLE, YELLOW);
    let hud = format!(
        "Level {}/{}   Sun {}   Zombie {}/{}   Killed {}   {}   Best {}   M:{}",
        g.level,
        MAX_LEVELS,
        g.sun,
        g.spawned,
        cfg.zombies,
        g.killed,
        fmt_time(g.elapsed),
        best_text,
        if audio_on { "on" } else { "off" }
    );
    text_centered(&hud, w / 2.0, 70.0, FONT_SMALL, WHITE);

    let seg_w = 14.0;
    let seg_gap = 3.0;
    let total_w = 20.0 * (seg_w + seg_gap) - seg_gap;
    let start_x = w / 2.0 - total_w / 2.0;
    let filled = g.spawned * 20 / cfg.zombies;
    for i in 0..20 {
        let x = start_x + i as f32 * (seg_w + seg_gap);
        draw_rectangle(
            x,
            96.0,
            seg_w,
            9.0,
            if i < filled { GREEN } else { col(45, 55, 52) },
        );
    }
    let label = format!("Selected: {}", g.selected.name());
    draw_text(
        &label,
        start_x + total_w + 16.0,
        SELECTED_LABEL_Y,
        FONT_SMALL,
        GREY,
    );

    let _ = l;
}

fn draw_result(g: &Game, l: &Layout, records: &Records, improvement: Improvement) {
    let won = g.status == Status::Won;
    let accent = if won { GREEN } else { RED };
    let h = screen_height();
    let box_y = l.board_y + l.board_h + 12.0;
    let box_h = (h - box_y - 12.0).min(112.0);
    if box_h < 112.0 {
        return;
    }
    let cx = screen_width() / 2.0;
    draw_rectangle(l.board_x, box_y, l.board_w, box_h, col(20, 20, 30));
    draw_rectangle_lines(l.board_x, box_y, l.board_w, box_h, 3.0, accent);
    let title = if !won {
        "GAME OVER"
    } else if g.level == MAX_LEVELS {
        "ALL LEVELS CLEARED!"
    } else {
        "LEVEL CLEARED!"
    };
    text_centered(title, cx, box_y + 34.0, FONT_BODY, accent);
    let cfg = level_config(g.level);
    let detail = format!(
        "Level {}/{}   Kills {}/{}   Time {}   Score {}",
        g.level,
        MAX_LEVELS,
        g.killed,
        cfg.zombies,
        fmt_time(g.elapsed),
        g.score()
    );
    text_centered(&detail, cx, box_y + 58.0, FONT_SMALL, accent);

    let best = records.get(g.level);
    let record = match best.best_time {
        Some(t) => format!("Best {} / {} pts", fmt_time(t), best.best_score),
        None => "No record yet".to_string(),
    };
    let record = format!("{record}   Stars {}/3", g.stars());
    let record = if won && (improvement.time || improvement.score || improvement.stars) {
        let what = if improvement.time && improvement.score {
            "NEW RECORD!"
        } else if improvement.stars {
            "NEW STARS!"
        } else if improvement.time {
            "NEW BEST TIME!"
        } else {
            "NEW HIGH SCORE!"
        };
        format!("{what}   {record}")
    } else {
        record
    };
    text_centered(&record, cx, box_y + 80.0, FONT_SMALL, accent);

    let footer = if !won {
        "R retry  |  N new game  |  Esc menu  |  Q quit"
    } else if g.level == MAX_LEVELS {
        "Enter play again  |  R replay  |  Esc menu  |  Q quit"
    } else {
        "Enter next level  |  R replay  |  Esc menu  |  Q quit"
    };
    text_centered(footer, cx, box_y + 102.0, FONT_SMALL, accent);
}

pub fn draw_game(
    g: &Game,
    sprites: &Sprites,
    records: &Records,
    improvement: Improvement,
    audio_on: bool,
) {
    let l = layout();
    let t = get_time() as f32;
    let (mx, my) = mouse_position();
    let hover = cell_at(&l, mx, my);
    draw_background(&l);
    draw_hud(g, &l, records, audio_on);
    draw_cards(sprites, g, &l);

    for r in 0..ROWS {
        for c in 0..COLS {
            let x = l.board_x + c as f32 * l.cell;
            let y = l.board_y + r as f32 * l.cell;
            let tile = if (r + c) % 2 == 0 {
                col(28, 54, 36)
            } else {
                col(34, 62, 40)
            };
            draw_rectangle(x, y, l.cell, l.cell, tile);
        }
    }
    if g.status == Status::Playing {
        let (cr, cc) = g.cursor;
        if cr < ROWS && cc < COLS {
            let x = l.board_x + cc as f32 * l.cell;
            let y = l.board_y + cr as f32 * l.cell;
            draw_rectangle(x, y, l.cell, l.cell, Color::from_rgba(90, 100, 50, 90));
            draw_rectangle_lines(x, y, l.cell, l.cell, 2.0, YELLOW);
        }
        draw_ghost(sprites, g, &l, hover);
    }

    draw_lawnmowers(sprites, g, &l);
    for p in &g.plants {
        let x = l.board_x + p.col as f32 * l.cell;
        let y = l.board_y + p.row as f32 * l.cell;
        draw_plant(sprites, p, x, y, l.cell, t);
    }
    for pea in &g.peas {
        let x = l.board_x + (pea.x + 0.5) * l.cell;
        let y = l.board_y + pea.row as f32 * l.cell + l.cell / 2.0;
        sprites.draw_centered(Sprite::Pea, x, y, l.cell * 0.42);
    }
    for z in &g.zombies {
        let x = l.board_x + z.x * l.cell;
        let y = l.board_y + z.row as f32 * l.cell;
        draw_zombie(sprites, z, x, y, l.cell, t);
    }
    for s in &g.suns {
        let x = l.board_x + (s.col as f32 + 0.5) * l.cell;
        let y = l.board_y + (s.y + 0.5) * l.cell;
        draw_sun(sprites, x, y, l.cell, t);
    }
    for e in &g.effects {
        draw_particles(e, &l);
    }
    draw_floaters(g, &l);
    draw_wave_warning(g, &l, t);

    if g.status == Status::Playing {
        let hints = "1/2/3 pick  |  Arrows move  |  Space plant  |  X shovel  |  C sun  |  M sound  |  Esc menu";
        text_centered(
            hints,
            screen_width() / 2.0,
            screen_height() - HINT_BOTTOM_OFFSET,
            FONT_SMALL,
            GREY,
        );
        // Notices used to sit at y=144, right on top of the plant cards.
        // Keep them in the free strip above the hint line instead.
        if let Some((msg, _)) = &g.notice {
            text_centered(
                msg,
                screen_width() / 2.0,
                screen_height() - NOTICE_BOTTOM_OFFSET,
                FONT_SMALL,
                YELLOW,
            );
        }
    } else {
        // The result box already says what happened, so no popup here.
        draw_result(g, &l, records, improvement);
    }
    let _ = audio_on;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mouse_maps_to_cells_and_cards() {
        let l = layout_for(1280.0, 720.0);
        assert_eq!(cell_at(&l, l.board_x, l.board_y), Some((0, 0)));
        assert_eq!(
            cell_at(&l, l.board_x + l.cell * 2.0, l.board_y + l.cell * 4.0),
            Some((4, 2))
        );
        assert_eq!(cell_at(&l, l.board_x - 1.0, l.board_y), None);
        assert_eq!(
            cell_at(&l, l.board_x + l.cell * COLS as f32, l.board_y),
            None
        );
        assert_eq!(
            card_at(&l, l.cards_x, l.cards_y + 1.0),
            Some(PlantKind::Sunflower)
        );
        assert_eq!(
            card_at(&l, l.cards_x + l.card_w + l.card_gap, l.cards_y + 1.0),
            Some(PlantKind::Peashooter)
        );
        assert_eq!(card_at(&l, l.cards_x, l.cards_y - 1.0), None);
    }

    #[test]
    fn board_is_centered_with_room_for_result_box() {
        let l = layout_for(1280.0, 720.0);
        assert!((l.board_x - (1280.0 - l.board_w) / 2.0).abs() < 0.01);
        assert!(l.board_y + l.board_h + 124.0 <= 720.0);
    }

    #[test]
    fn popups_do_not_overlap_cards_or_hints() {
        let l = layout_for(1280.0, 720.0);
        let h = 720.0;
        let notice_y = h - NOTICE_BOTTOM_OFFSET;
        let hint_y = h - HINT_BOTTOM_OFFSET;
        assert!(notice_y > l.cards_y + l.card_h);
        assert!(notice_y + FONT_SMALL < hint_y);
        assert!(hint_y + FONT_SMALL <= h);
        assert!(SELECTED_LABEL_Y + FONT_SMALL <= l.cards_y);
    }

    #[test]
    fn pixel_font_sizes_are_grid_aligned() {
        for size in [FONT_SMALL, FONT_BODY, FONT_TITLE] {
            assert_eq!(size % 8.0, 0.0, "font size {size} breaks the 8px grid");
        }
    }

    #[test]
    fn particles_move_in_the_direction_of_their_effect() {
        let shoot = Effect {
            kind: EffectKind::Shoot,
            x: 2.5,
            y: 1.5,
            life: 0.11,
            duration: 0.22,
        };
        let (dx, dy) = particle_offset(&shoot, 0);
        assert!(dx > 0.0, "pea puffs fly right");
        assert!(dx.abs() > dy.abs(), "shoot burst stays mostly horizontal");

        let sun = Effect {
            kind: EffectKind::Sun,
            x: 3.5,
            y: 2.5,
            life: 0.275,
            duration: 0.55,
        };
        let (_, dy) = particle_offset(&sun, 0);
        assert!(dy < 0.0, "sun sparkles drift upward");
    }
}
