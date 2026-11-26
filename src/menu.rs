//! Start menu, level select, and help screen.

use crate::game::MAX_LEVELS;
use crate::pixel_art::{Sprite, Sprites};
use crate::records::Records;
use crate::render::fmt_time;
use macroquad::prelude::*;

const TITLE_SIZE: f32 = 48.0;
const HEAD_SIZE: f32 = 32.0;
const BODY_SIZE: f32 = 16.0;
const ROW_SIZE: f32 = 24.0;

fn col(r: u8, g: u8, b: u8) -> Color {
    Color::from_rgba(r, g, b, 255)
}

const YELLOW: Color = Color::new(1.0, 0.84, 0.26, 1.0);
const GREEN: Color = Color::new(0.29, 0.66, 0.30, 1.0);
const RED: Color = Color::new(0.95, 0.30, 0.28, 1.0);
const WHITE: Color = Color::new(0.92, 0.94, 0.90, 1.0);
const GREY: Color = Color::new(0.55, 0.60, 0.58, 1.0);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Screen {
    Main,
    LevelSelect,
    Help,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuResult {
    Start(usize),
    Quit,
    None,
}

pub struct Menu {
    pub screen: Screen,
    pub index: usize,
    pub level: usize,
    pub notice: Option<String>,
}

impl Menu {
    pub fn new() -> Self {
        Self {
            screen: Screen::Main,
            index: 0,
            level: 1,
            notice: None,
        }
    }

    pub fn back(&mut self) {
        self.screen = Screen::Main;
        self.index = 0;
        self.notice = None;
    }

    /// Move the highlighted item, wrapping around.
    pub fn move_selection(&mut self, delta: isize) {
        self.notice = None;
        match self.screen {
            Screen::Main => {
                let n = 4isize;
                self.index = ((self.index as isize + delta).rem_euclid(n)) as usize;
            }
            Screen::LevelSelect => {
                let n = MAX_LEVELS as isize;
                let next = (self.level as isize - 1 + delta).rem_euclid(n);
                self.level = (next + 1) as usize;
            }
            Screen::Help => {}
        }
    }

    pub fn is_locked(level: usize, records: &Records) -> bool {
        level > 1 && records.get(level - 1).best_time.is_none()
    }

    pub fn activate(&mut self, records: &Records) -> MenuResult {
        self.notice = None;
        match self.screen {
            Screen::Main => match self.index {
                0 => MenuResult::Start(1),
                1 => {
                    self.screen = Screen::LevelSelect;
                    self.level = 1;
                    MenuResult::None
                }
                2 => {
                    self.screen = Screen::Help;
                    MenuResult::None
                }
                _ => MenuResult::Quit,
            },
            Screen::LevelSelect => {
                if Self::is_locked(self.level, records) {
                    self.notice = Some(format!("Clear level {} first", self.level - 1));
                    MenuResult::None
                } else {
                    MenuResult::Start(self.level)
                }
            }
            Screen::Help => {
                self.back();
                MenuResult::None
            }
        }
    }

    pub fn click(&mut self, x: f32, y: f32, records: &Records) -> Option<MenuResult> {
        match self.screen {
            Screen::Main => {
                for i in 0..4 {
                    if main_item_rect(i).contains(vec2(x, y)) {
                        self.index = i;
                        return Some(self.activate(records));
                    }
                }
            }
            Screen::LevelSelect => {
                for level in 1..=MAX_LEVELS {
                    if level_rect(level).contains(vec2(x, y)) {
                        self.level = level;
                        return Some(self.activate(records));
                    }
                }
                if back_rect().contains(vec2(x, y)) {
                    self.back();
                }
            }
            Screen::Help => {
                if back_rect().contains(vec2(x, y)) {
                    self.back();
                }
            }
        }
        None
    }
}

fn text_centered(s: &str, cx: f32, y: f32, size: f32, color: Color) {
    let width = measure_text(s, None, size as u16, 1.0).width;
    draw_text(s, cx - width / 2.0, y, size, color);
}

/// `draw_text` takes the baseline as y, and Silkscreen glyphs sit roughly
/// `size` pixels above it. Center that glyph box inside the rectangle.
fn centered_baseline(r: Rect, size: f32) -> f32 {
    r.y + (r.h + size) / 2.0
}

fn main_item_rect_at(screen_w: f32, i: usize) -> Rect {
    let w = 460.0;
    Rect::new((screen_w - w) / 2.0, 256.0 + i as f32 * 56.0, w, 44.0)
}

fn main_item_rect(i: usize) -> Rect {
    main_item_rect_at(screen_width(), i)
}

fn level_rect_at(screen_w: f32, level: usize) -> Rect {
    let card_w = 300.0;
    let gap = 24.0;
    let total = card_w * MAX_LEVELS as f32 + gap * (MAX_LEVELS - 1) as f32;
    let x = (screen_w - total) / 2.0 + (level - 1) as f32 * (card_w + gap);
    Rect::new(x, 252.0, card_w, 196.0)
}

fn level_rect(level: usize) -> Rect {
    level_rect_at(screen_width(), level)
}

fn back_rect_at(screen_w: f32, screen_h: f32) -> Rect {
    let w = 240.0;
    Rect::new((screen_w - w) / 2.0, screen_h - 74.0, w, 40.0)
}

fn back_rect() -> Rect {
    back_rect_at(screen_width(), screen_height())
}

fn draw_background() {
    let w = screen_width();
    let h = screen_height();
    draw_rectangle(0.0, 0.0, w, h, col(14, 20, 28));
    draw_rectangle(0.0, h * 0.72, w, h, col(18, 38, 30));
    draw_rectangle(0.0, h * 0.86, w, h, col(24, 32, 28));
}

fn draw_title(sprites: &Sprites) {
    let w = screen_width();
    // draw_text uses the baseline as y, so leave room for the glyphs above it.
    let y = 64.0;
    let t = get_time() as f32;
    let step = (t * 2.5).floor() % 2.0 == 1.0;
    let zombie_frame = if step {
        Sprite::ZombieStep
    } else {
        Sprite::Zombie
    };
    let bob = (t * 5.0).sin() * 4.0;
    sprites.draw_centered(Sprite::Sunflower, w / 2.0 - 400.0, y + 34.0, 80.0);
    sprites.draw_centered(zombie_frame, w / 2.0 + 400.0, y + 34.0 + bob, 80.0);
    text_centered("PLANTS vs ZOMBIES", w / 2.0, y, TITLE_SIZE, YELLOW);
    text_centered("PIXEL EDITION", w / 2.0, y + 58.0, BODY_SIZE, GREY);
}

fn draw_main(menu: &Menu, records: &Records) {
    let w = screen_width();
    let labels = ["START GAME", "LEVEL SELECT", "HOW TO PLAY", "QUIT"];
    for (i, label) in labels.iter().enumerate() {
        let r = main_item_rect(i);
        let active = i == menu.index;
        draw_rectangle(r.x, r.y, r.w, r.h, col(28, 40, 38));
        draw_rectangle_lines(r.x, r.y, r.w, r.h, 2.0, if active { YELLOW } else { GREY });
        let shown = if active {
            format!("> {label} <")
        } else {
            (*label).to_string()
        };
        text_centered(
            &shown,
            r.x + r.w / 2.0,
            centered_baseline(r, ROW_SIZE),
            ROW_SIZE,
            if active { YELLOW } else { WHITE },
        );
    }

    // Best records per level, so the menu doubles as a progress screen.
    let card_w = 300.0;
    let gap = 24.0;
    let total = card_w * MAX_LEVELS as f32 + gap * (MAX_LEVELS - 1) as f32;
    let start = (w - total) / 2.0;
    for level in 1..=MAX_LEVELS {
        let best = records.get(level);
        let text = match best.best_time {
            Some(t) => format!("LV{level}  {} / {} pts", fmt_time(t), best.best_score),
            None => format!("LV{level}  --:-- / 0 pts"),
        };
        text_centered(
            &text,
            start + (level - 1) as f32 * (card_w + gap) + card_w / 2.0,
            512.0,
            BODY_SIZE,
            if best.best_time.is_some() {
                GREEN
            } else {
                GREY
            },
        );
    }
    text_centered(
        "Arrows move  |  Enter select  |  Q quit",
        w / 2.0,
        screen_height() - 40.0,
        BODY_SIZE,
        GREY,
    );
}

fn draw_level_select(menu: &Menu, records: &Records, sprites: &Sprites) {
    let w = screen_width();
    for level in 1..=MAX_LEVELS {
        let r = level_rect(level);
        let locked = Menu::is_locked(level, records);
        let active = level == menu.level;
        let accent = if locked {
            RED
        } else if active {
            YELLOW
        } else {
            GREY
        };
        draw_rectangle(r.x, r.y, r.w, r.h, col(26, 38, 36));
        draw_rectangle_lines(r.x, r.y, r.w, r.h, 3.0, accent);
        text_centered(
            &format!("LEVEL {level}"),
            r.x + r.w / 2.0,
            r.y + 40.0,
            HEAD_SIZE,
            accent,
        );
        let icon = if level == 1 {
            Sprite::Sunflower
        } else if level == 2 {
            Sprite::Peashooter
        } else {
            Sprite::Wallnut
        };
        sprites.draw_centered(icon, r.x + r.w / 2.0, r.y + 98.0, 64.0);
        let best = records.get(level);
        let detail = match best.best_time {
            Some(t) => format!("BEST {} / {} pts", fmt_time(t), best.best_score),
            None => "NO RECORD YET".to_string(),
        };
        text_centered(&detail, r.x + r.w / 2.0, r.y + 150.0, BODY_SIZE, WHITE);
        text_centered(
            if locked { "LOCKED" } else { "READY" },
            r.x + r.w / 2.0,
            r.y + 178.0,
            BODY_SIZE,
            accent,
        );
    }
    text_centered(
        "Enter start  |  Arrows switch level  |  Esc back",
        w / 2.0,
        screen_height() - 110.0,
        BODY_SIZE,
        GREY,
    );
    let r = back_rect();
    draw_rectangle(r.x, r.y, r.w, r.h, col(28, 40, 38));
    draw_rectangle_lines(r.x, r.y, r.w, r.h, 2.0, GREY);
    text_centered(
        "BACK",
        r.x + r.w / 2.0,
        centered_baseline(r, ROW_SIZE),
        ROW_SIZE,
        WHITE,
    );
}

fn draw_help() {
    let w = screen_width();
    text_centered("HOW TO PLAY", w / 2.0, 210.0, HEAD_SIZE, YELLOW);
    let lines = [
        "1 / 2 / 3  -  pick Sunflower, Peashooter, Wallnut",
        "Arrows / WASD  -  move the planting cursor",
        "Space / Enter / click  -  plant on the tile",
        "C / click a sun  -  collect sun",
        "M sound on/off   N restart campaign",
        "Survive all 3 levels before a zombie gets home",
        "Esc menu   Q quit",
    ];
    for (i, line) in lines.iter().enumerate() {
        text_centered(line, w / 2.0, 276.0 + i as f32 * 38.0, BODY_SIZE, WHITE);
    }
    let r = back_rect();
    draw_rectangle(r.x, r.y, r.w, r.h, col(28, 40, 38));
    draw_rectangle_lines(r.x, r.y, r.w, r.h, 2.0, GREY);
    text_centered(
        "BACK",
        r.x + r.w / 2.0,
        centered_baseline(r, ROW_SIZE),
        ROW_SIZE,
        WHITE,
    );
}

pub fn draw_menu(menu: &Menu, records: &Records, sprites: &Sprites) {
    draw_background();
    draw_title(sprites);
    match menu.screen {
        Screen::Main => draw_main(menu, records),
        Screen::LevelSelect => draw_level_select(menu, records, sprites),
        Screen::Help => draw_help(),
    }
    if let Some(msg) = &menu.notice {
        let notice_y = if menu.screen == Screen::Main {
            screen_height() - 64.0
        } else {
            screen_height() - 150.0
        };
        text_centered(msg, screen_width() / 2.0, notice_y, BODY_SIZE, YELLOW);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn temp_path(name: &str) -> PathBuf {
        let path = std::env::temp_dir().join(name);
        let _ = fs::remove_file(&path);
        path
    }

    fn empty_records() -> Records {
        Records::from_path(temp_path("zpg-menu-empty.txt"))
    }

    #[test]
    fn level_two_starts_locked_and_unlocks_after_level_one() {
        let path = temp_path("zpg-menu-unlock.txt");
        let mut records = Records::from_path(path);
        assert!(Menu::is_locked(2, &records));

        records.submit(1, 42.0, 1234);
        assert!(!Menu::is_locked(2, &records));
        assert!(Menu::is_locked(3, &records));
    }

    #[test]
    fn menu_navigation_wraps() {
        let mut menu = Menu::new();
        assert_eq!(menu.index, 0);
        menu.move_selection(-1);
        assert_eq!(menu.index, 3);
        menu.move_selection(1);
        assert_eq!(menu.index, 0);
    }

    #[test]
    fn locked_level_cannot_be_started() {
        let records = empty_records();
        let mut menu = Menu::new();
        menu.screen = Screen::LevelSelect;
        menu.level = 2;
        assert_eq!(menu.activate(&records), MenuResult::None);
        assert!(menu.notice.is_some());
    }

    #[test]
    fn menu_labels_fit_vertically_inside_their_boxes() {
        let row = main_item_rect_at(1280.0, 0);
        let baseline = centered_baseline(row, ROW_SIZE);
        assert!(baseline - ROW_SIZE >= row.y);
        assert!(baseline <= row.y + row.h);

        let back = back_rect_at(1280.0, 720.0);
        let baseline = centered_baseline(back, ROW_SIZE);
        assert!(baseline - ROW_SIZE >= back.y);
        assert!(baseline <= back.y + back.h);

        let card = level_rect_at(1280.0, 1);
        assert!(40.0 - HEAD_SIZE >= 0.0);
        assert!(card.y + 178.0 <= card.y + card.h);
    }
}
