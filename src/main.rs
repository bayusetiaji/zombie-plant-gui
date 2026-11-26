mod audio;
mod game;
mod menu;
mod pixel_art;
mod records;
mod render;
mod rng;

use audio::{Audio, Sound};
use game::{Game, GameEvent, PlantKind, Status};
use macroquad::prelude::*;
use menu::{Menu, MenuResult, Screen as MenuScreen, draw_menu};
use pixel_art::Sprites;
use records::{Improvement, Records};
use render::{card_at, cell_at, draw_game, layout};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AppScreen {
    Menu,
    Game,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum GameInput {
    Continue,
    Quit,
    ToMenu,
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Zombie Plant - GUI".to_owned(),
        window_width: 1280,
        window_height: 720,
        window_resizable: true,
        high_dpi: true,
        ..Default::default()
    }
}

fn random_seed() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0x1234_5678)
}

fn handle_menu_input(menu: &mut Menu, records: &Records, audio: &mut Audio) -> (bool, MenuResult) {
    if is_key_pressed(KeyCode::Q) || is_key_pressed(KeyCode::Escape) {
        if menu.screen == MenuScreen::Main {
            return (false, MenuResult::None);
        }
        menu.back();
        audio.play(Sound::Select);
        return (true, MenuResult::None);
    }
    if is_key_pressed(KeyCode::Up)
        || is_key_pressed(KeyCode::W)
        || is_key_pressed(KeyCode::Left)
        || is_key_pressed(KeyCode::A)
    {
        menu.move_selection(-1);
        audio.play(Sound::Select);
    }
    if is_key_pressed(KeyCode::Down)
        || is_key_pressed(KeyCode::S)
        || is_key_pressed(KeyCode::Right)
        || is_key_pressed(KeyCode::D)
    {
        menu.move_selection(1);
        audio.play(Sound::Select);
    }

    let mut result = MenuResult::None;
    if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
        result = menu.activate(records);
        audio.play(Sound::Select);
    }
    if is_mouse_button_pressed(MouseButton::Left) {
        let (x, y) = mouse_position();
        if let Some(clicked) = menu.click(x, y, records) {
            result = clicked;
            audio.play(Sound::Select);
        }
    }
    (true, result)
}

fn handle_game_input(g: &mut Game, audio: &mut Audio) -> GameInput {
    if is_key_pressed(KeyCode::Escape) {
        return GameInput::ToMenu;
    }
    if is_key_pressed(KeyCode::Q) {
        return GameInput::Quit;
    }
    if is_key_pressed(KeyCode::Key1) {
        g.select(PlantKind::Sunflower);
    }
    if is_key_pressed(KeyCode::Key2) {
        g.select(PlantKind::Peashooter);
    }
    if is_key_pressed(KeyCode::Key3) {
        g.select(PlantKind::Wallnut);
    }
    if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) {
        g.cursor.0 = g.cursor.0.saturating_sub(1);
    }
    if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S) {
        g.cursor.0 = (g.cursor.0 + 1).min(game::ROWS - 1);
    }
    if is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::A) {
        g.cursor.1 = g.cursor.1.saturating_sub(1);
    }
    if is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::D) {
        g.cursor.1 = (g.cursor.1 + 1).min(game::COLS - 1);
    }
    if is_key_pressed(KeyCode::Space) {
        let (r, c) = g.cursor;
        g.try_plant(r, c);
    }
    if is_key_pressed(KeyCode::X) {
        let (r, c) = g.cursor;
        g.try_shovel(r, c);
    }
    if is_key_pressed(KeyCode::Enter) {
        if g.status == Status::Won {
            g.next_level();
        } else if g.status == Status::Playing {
            let (r, c) = g.cursor;
            g.try_plant(r, c);
        }
    }
    if is_key_pressed(KeyCode::C) {
        g.collect_nearest();
    }
    if is_key_pressed(KeyCode::M) {
        audio.toggle_mute();
    }
    if is_key_pressed(KeyCode::N) {
        g.restart_campaign();
    }
    if is_key_pressed(KeyCode::R) && g.status != Status::Playing {
        g.restart();
    }

    if is_mouse_button_pressed(MouseButton::Left) {
        let (mx, my) = mouse_position();
        let l = layout();
        if let Some(kind) = card_at(&l, mx, my) {
            g.select(kind);
        } else if let Some((r, c)) = cell_at(&l, mx, my) {
            if !g.collect_at(r, c) {
                g.try_plant(r, c);
            }
        }
    }
    if is_mouse_button_pressed(MouseButton::Right) {
        let (mx, my) = mouse_position();
        if let Some((r, c)) = cell_at(&layout(), mx, my) {
            g.try_shovel(r, c);
        }
    }
    GameInput::Continue
}

#[macroquad::main(window_conf)]
async fn main() {
    // Pixel-style font. Its glyphs are built on an 8px grid, so the text sizes
    // in render.rs and menu.rs are multiples of 8 to keep the pixels aligned.
    match load_ttf_font_from_bytes(include_bytes!("../assets/Silkscreen-Bold.ttf")) {
        Ok(font) => set_default_font(font),
        Err(err) => eprintln!("Could not load bundled pixel font, using fallback: {err:?}"),
    }

    let sprites = Sprites::load();
    let mut audio = Audio::new();
    let mut records = Records::load();
    let mut menu = Menu::new();
    let mut game = Game::new(random_seed());
    let mut last_improvement = Improvement::default();
    let mut screen = AppScreen::Menu;
    let mut last = get_time() as f32;

    loop {
        let now = get_time() as f32;
        let dt = (now - last).min(0.1);
        last = now;

        let mut quit = false;
        match screen {
            AppScreen::Menu => {
                let (cont, result) = handle_menu_input(&mut menu, &records, &mut audio);
                if !cont {
                    quit = true;
                }
                match result {
                    MenuResult::Start(level) => {
                        game = Game::with_level(random_seed(), level);
                        last_improvement = Improvement::default();
                        screen = AppScreen::Game;
                    }
                    MenuResult::Quit => quit = true,
                    MenuResult::None => {}
                }
            }
            AppScreen::Game => match handle_game_input(&mut game, &mut audio) {
                GameInput::Quit => quit = true,
                GameInput::ToMenu => screen = AppScreen::Menu,
                GameInput::Continue => {
                    game.update(dt);
                    let events: Vec<GameEvent> = game.events.drain(..).collect();
                    for event in events {
                        if event == GameEvent::Win {
                            last_improvement = records.submit(
                                game.level,
                                game.elapsed,
                                game.score(),
                                game.stars(),
                            );
                        }
                        let sound = match event {
                            GameEvent::Select => Sound::Select,
                            GameEvent::Plant => Sound::Plant,
                            GameEvent::Shovel => Sound::Shovel,
                            GameEvent::Shoot => Sound::Shoot,
                            GameEvent::Hit => Sound::Hit,
                            GameEvent::ZombieDie => Sound::ZombieDie,
                            GameEvent::Sun => Sound::Sun,
                            GameEvent::WaveWarning => Sound::WaveWarning,
                            GameEvent::Lawnmower => Sound::Lawnmower,
                            GameEvent::Win => Sound::Win,
                            GameEvent::Lose => Sound::Lose,
                        };
                        audio.play(sound);
                    }
                    audio.tick(dt, game.zombies.len() >= 3 || game.elapsed > 25.0);
                }
            },
        }
        if quit {
            break;
        }

        match screen {
            AppScreen::Menu => {
                audio.tick(dt, false);
                draw_menu(&menu, &records, &sprites);
            }
            AppScreen::Game => draw_game(
                &game,
                &sprites,
                &records,
                last_improvement,
                audio.available() && !audio.muted(),
            ),
        }
        next_frame().await;
    }
}
