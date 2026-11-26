use crate::rng::Rng;
use std::time::{SystemTime, UNIX_EPOCH};

pub const ROWS: usize = 5;
pub const COLS: usize = 12;
pub const MAX_LEVELS: usize = 3;

const START_SUN: i32 = 200;
const SUN_VALUE: i32 = 40;
const PEA_SPEED: f32 = 7.5;
const PEA_DAMAGE: f32 = 20.0;
const PEA_COOLDOWN: f32 = 1.2;
const ZOMBIE_SPEED: f32 = 0.34;
const ZOMBIE_HP: f32 = 100.0;
const EAT_INTERVAL: f32 = 0.5;
const EAT_DAMAGE: f32 = 18.0;
const SUN_COOLDOWN: f32 = 6.0;
const SKY_SUN_INTERVAL: f32 = 7.0;
const SUN_LIFETIME: f32 = 14.0;

#[derive(Clone, Copy, Debug)]
pub struct LevelConfig {
    pub zombies: usize,
    pub start_sun: i32,
    pub first_spawn: f32,
    pub spawn_base: f32,
    pub spawn_min: f32,
    pub speed_mul: f32,
    pub zombie_hp: f32,
}

pub fn level_config(level: usize) -> LevelConfig {
    match level.clamp(1, MAX_LEVELS) {
        1 => LevelConfig {
            zombies: 10,
            start_sun: START_SUN,
            first_spawn: 6.0,
            spawn_base: 4.4,
            spawn_min: 2.6,
            speed_mul: 1.0,
            zombie_hp: ZOMBIE_HP,
        },
        2 => LevelConfig {
            zombies: 14,
            start_sun: START_SUN,
            first_spawn: 5.0,
            spawn_base: 3.9,
            spawn_min: 2.3,
            speed_mul: 1.15,
            zombie_hp: 110.0,
        },
        _ => LevelConfig {
            zombies: 18,
            start_sun: START_SUN,
            first_spawn: 4.0,
            spawn_base: 3.5,
            spawn_min: 2.0,
            speed_mul: 1.3,
            zombie_hp: 120.0,
        },
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlantKind {
    Sunflower,
    Peashooter,
    Wallnut,
}

impl PlantKind {
    pub const ALL: [PlantKind; 3] = [
        PlantKind::Sunflower,
        PlantKind::Peashooter,
        PlantKind::Wallnut,
    ];

    pub fn cost(self) -> i32 {
        match self {
            PlantKind::Sunflower => 50,
            PlantKind::Peashooter => 100,
            PlantKind::Wallnut => 50,
        }
    }

    #[allow(dead_code)]
    pub fn emoji(self) -> &'static str {
        match self {
            PlantKind::Sunflower => "🌻",
            PlantKind::Peashooter => "🌱",
            PlantKind::Wallnut => "🥜",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            PlantKind::Sunflower => "Sunflower",
            PlantKind::Peashooter => "Peashooter",
            PlantKind::Wallnut => "Wall-nut",
        }
    }

    pub fn max_hp(self) -> f32 {
        match self {
            PlantKind::Wallnut => 350.0,
            _ => 100.0,
        }
    }
}

pub struct Plant {
    pub kind: PlantKind,
    pub row: usize,
    pub col: usize,
    pub hp: f32,
    timer: f32,
}

pub struct Zombie {
    pub x: f32,
    pub row: usize,
    pub hp: f32,
    pub max_hp: f32,
    pub walk_phase: f32,
    pub walking: bool,
    attack: f32,
    speed: f32,
}

pub struct Pea {
    pub x: f32,
    pub row: usize,
}

pub struct Sun {
    pub col: usize,
    pub y: f32,
    target_y: f32,
    life: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Playing,
    Won,
    Lost,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameEvent {
    Select,
    Plant,
    Shoot,
    Hit,
    ZombieDie,
    Sun,
    Win,
    Lose,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EffectKind {
    Shoot,
    Hit,
    Plant,
    Sun,
    Eat,
    Death,
}

/// A short-lived visual burst. `x`/`y` are tile-space centers, so the renderer
/// can place pea puffs, hits, and death clouds exactly where they happened.
#[derive(Clone, Copy, Debug)]
pub struct Effect {
    pub kind: EffectKind,
    pub x: f32,
    pub y: f32,
    pub life: f32,
    pub duration: f32,
}

impl Effect {
    fn new(kind: EffectKind, x: f32, y: f32) -> Self {
        let duration = match kind {
            EffectKind::Shoot => 0.22,
            EffectKind::Hit => 0.30,
            EffectKind::Plant => 0.48,
            EffectKind::Sun => 0.55,
            EffectKind::Eat => 0.36,
            EffectKind::Death => 0.60,
        };
        Self {
            kind,
            x,
            y,
            life: duration,
            duration,
        }
    }
}

fn effect(kind: EffectKind, x: f32, y: f32) -> Effect {
    Effect::new(kind, x, y)
}

pub struct Game {
    pub level: usize,
    pub sun: i32,
    pub selected: PlantKind,
    pub plants: Vec<Plant>,
    pub zombies: Vec<Zombie>,
    pub peas: Vec<Pea>,
    pub suns: Vec<Sun>,
    pub cursor: (usize, usize),
    pub status: Status,
    pub spawned: usize,
    pub killed: usize,
    pub elapsed: f32,
    pub notice: Option<(String, f32)>,
    pub events: Vec<GameEvent>,
    pub effects: Vec<Effect>,
    sky_timer: f32,
    zombie_timer: f32,
    rng: Rng,
}

fn fresh_seed() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0x1234_5678)
}

impl Game {
    pub fn new(seed: u64) -> Self {
        Self::with_level(seed, 1)
    }

    pub fn with_level(seed: u64, level: usize) -> Self {
        let level = level.clamp(1, MAX_LEVELS);
        let cfg = level_config(level);
        Self {
            level,
            sun: cfg.start_sun,
            selected: PlantKind::Peashooter,
            plants: Vec::new(),
            zombies: Vec::new(),
            peas: Vec::new(),
            suns: Vec::new(),
            cursor: (2, 4),
            status: Status::Playing,
            spawned: 0,
            killed: 0,
            elapsed: 0.0,
            notice: Some((
                format!(
                    "Level {} - pick a plant (1/2/3), then plant with Space.",
                    level
                ),
                7.0,
            )),
            events: Vec::new(),
            effects: Vec::new(),
            sky_timer: 3.0,
            zombie_timer: cfg.first_spawn,
            rng: Rng::new(seed),
        }
    }

    /// Restart the current level with a fresh layout.
    pub fn restart(&mut self) {
        let level = self.level;
        *self = Self::with_level(fresh_seed(), level);
    }

    /// Start the campaign again from level 1.
    pub fn restart_campaign(&mut self) {
        *self = Self::with_level(fresh_seed(), 1);
    }

    /// Advance after a win; after the final level, loop back to level 1.
    pub fn next_level(&mut self) {
        if self.status != Status::Won {
            return;
        }
        let next = if self.level < MAX_LEVELS {
            self.level + 1
        } else {
            1
        };
        *self = Self::with_level(fresh_seed(), next);
    }

    /// Score for a completed level: kills, leftover sun, and a time bonus.
    pub fn score(&self) -> i32 {
        let time_bonus = ((180.0 - self.elapsed).max(0.0) * 10.0) as i32;
        self.killed as i32 * 100 + self.sun * 2 + time_bonus
    }

    fn set_notice(&mut self, msg: impl Into<String>) {
        self.notice = Some((msg.into(), 2.5));
    }

    pub fn select(&mut self, kind: PlantKind) {
        self.selected = kind;
        self.set_notice(format!("Selected {} ({} sun)", kind.name(), kind.cost()));
        self.events.push(GameEvent::Select);
    }

    pub fn try_plant(&mut self, row: usize, col: usize) {
        if self.status != Status::Playing || row >= ROWS || col >= COLS {
            return;
        }
        if self.plants.iter().any(|p| p.row == row && p.col == col) {
            self.set_notice("That tile is already occupied");
            return;
        }
        let kind = self.selected;
        if self.sun < kind.cost() {
            self.set_notice(format!("Need {} sun, you have {}", kind.cost(), self.sun));
            return;
        }
        self.sun -= kind.cost();
        self.plants.push(Plant {
            kind,
            row,
            col,
            hp: kind.max_hp(),
            timer: 0.0,
        });
        self.events.push(GameEvent::Plant);
        self.effects.push(effect(
            EffectKind::Plant,
            col as f32 + 0.5,
            row as f32 + 0.5,
        ));
    }

    /// Returns true when a falling sun was collected at this tile.
    pub fn collect_at(&mut self, row: usize, col: usize) -> bool {
        if let Some(i) = self
            .suns
            .iter()
            .position(|s| s.col == col && s.y.floor() as usize == row && s.life > 0.0)
        {
            self.sun += SUN_VALUE;
            self.suns.remove(i);
            self.set_notice(format!("+{} sun", SUN_VALUE));
            self.events.push(GameEvent::Sun);
            self.effects
                .push(effect(EffectKind::Sun, col as f32 + 0.5, row as f32 + 0.5));
            true
        } else {
            false
        }
    }

    /// Keyboard fallback: collect the sun closest to the cursor.
    pub fn collect_nearest(&mut self) {
        if self.suns.is_empty() {
            self.set_notice("No sun to collect yet");
            return;
        }
        let (cr, cc) = self.cursor;
        let mut best: Option<(usize, f32)> = None;
        for (i, s) in self.suns.iter().enumerate() {
            let d = (s.col as f32 - cc as f32).abs() + (s.y - cr as f32).abs();
            if best.map_or(true, |(_, bd)| d < bd) {
                best = Some((i, d));
            }
        }
        if let Some((i, _)) = best {
            let sun = self.suns.remove(i);
            self.sun += SUN_VALUE;
            self.set_notice(format!("+{} sun", SUN_VALUE));
            self.events.push(GameEvent::Sun);
            self.effects
                .push(effect(EffectKind::Sun, sun.col as f32 + 0.5, sun.y + 0.5));
        }
    }

    pub fn update(&mut self, dt: f32) {
        let mut expired = false;
        if let Some((_, t)) = &mut self.notice {
            *t -= dt;
            if *t <= 0.0 {
                expired = true;
            }
        }
        if expired {
            self.notice = None;
        }
        for e in &mut self.effects {
            e.life -= dt;
        }
        self.effects.retain(|e| e.life > 0.0);

        if self.status != Status::Playing {
            return;
        }
        self.elapsed += dt;

        // Suns falling from the sky.
        self.sky_timer -= dt;
        if self.sky_timer <= 0.0 {
            self.sky_timer += SKY_SUN_INTERVAL;
            let col = self.rng.range(COLS);
            let target_y = self.rng.range(ROWS) as f32;
            self.suns.push(Sun {
                col,
                y: 0.0,
                target_y,
                life: SUN_LIFETIME,
            });
        }

        for s in &mut self.suns {
            if s.y < s.target_y {
                s.y = (s.y + 1.4 * dt).min(s.target_y);
            } else {
                s.life -= dt;
            }
        }
        self.suns.retain(|s| s.life > 0.0);

        // Plants: sunflowers generate sun, peashooters fire at zombies in their row.
        for p in &mut self.plants {
            p.timer -= dt;
            if p.timer > 0.0 {
                continue;
            }
            match p.kind {
                PlantKind::Sunflower => {
                    p.timer += SUN_COOLDOWN;
                    self.suns.push(Sun {
                        col: p.col,
                        y: p.row as f32,
                        target_y: p.row as f32,
                        life: SUN_LIFETIME,
                    });
                }
                PlantKind::Peashooter => {
                    let has_target = self
                        .zombies
                        .iter()
                        .any(|z| z.row == p.row && z.x > p.col as f32 + 0.2);
                    if has_target {
                        p.timer += PEA_COOLDOWN;
                        self.peas.push(Pea {
                            x: p.col as f32 + 0.4,
                            row: p.row,
                        });
                        self.events.push(GameEvent::Shoot);
                        self.effects.push(effect(
                            EffectKind::Shoot,
                            p.col as f32 + 0.85,
                            p.row as f32 + 0.5,
                        ));
                    } else {
                        p.timer = 0.0;
                    }
                }
                PlantKind::Wallnut => p.timer = 0.0,
            }
        }

        // Zombies walk left, or eat the plant blocking them. The phase only
        // advances while walking so a chewing zombie never shuffles its feet.
        for z in &mut self.zombies {
            let col = z.x.floor() as isize;
            let blocker = if (0..COLS as isize).contains(&col) {
                self.plants
                    .iter_mut()
                    .find(|p| p.row == z.row && p.col == col as usize)
            } else {
                None
            };
            if let Some(p) = blocker {
                z.walking = false;
                z.attack -= dt;
                if z.attack <= 0.0 {
                    z.attack += EAT_INTERVAL;
                    p.hp -= EAT_DAMAGE;
                    self.effects.push(effect(
                        EffectKind::Eat,
                        p.col as f32 + 0.55,
                        p.row as f32 + 0.45,
                    ));
                }
            } else {
                z.walking = true;
                z.walk_phase += dt * (5.0 + z.speed * 3.0);
                z.x -= z.speed * dt;
            }
        }
        self.plants.retain(|p| p.hp > 0.0);

        // Peas travel right and damage the first zombie they touch.
        let mut peas = std::mem::take(&mut self.peas);
        let mut survivors = Vec::with_capacity(peas.len());
        for mut pea in peas.drain(..) {
            pea.x += PEA_SPEED * dt;
            if pea.x > COLS as f32 + 0.8 {
                continue;
            }
            let mut hit = false;
            for z in &mut self.zombies {
                if z.row == pea.row && (z.x - pea.x).abs() < 0.45 {
                    z.hp -= PEA_DAMAGE;
                    hit = true;
                    self.events.push(GameEvent::Hit);
                    self.effects
                        .push(effect(EffectKind::Hit, z.x + 0.5, z.row as f32 + 0.5));
                    break;
                }
            }
            if !hit {
                survivors.push(pea);
            }
        }
        self.peas = survivors;

        let mut dying = Vec::new();
        self.zombies.retain(|z| {
            if z.hp <= 0.0 {
                dying.push((z.row, z.x + 0.5));
                false
            } else {
                true
            }
        });
        for (row, x) in dying {
            self.killed += 1;
            self.events.push(GameEvent::ZombieDie);
            self.effects
                .push(effect(EffectKind::Death, x, row as f32 + 0.5));
        }

        // Spawn waves: each level is a bit denser and faster than the last.
        let cfg = level_config(self.level);
        if self.spawned < cfg.zombies {
            self.zombie_timer -= dt;
            if self.zombie_timer <= 0.0 {
                let row = self.rng.range(ROWS);
                self.zombies.push(Zombie {
                    x: COLS as f32 - 0.1,
                    row,
                    hp: cfg.zombie_hp,
                    max_hp: cfg.zombie_hp,
                    walk_phase: self.rng.range_f32(0.0, 2.0),
                    walking: true,
                    attack: EAT_INTERVAL,
                    speed: ZOMBIE_SPEED * cfg.speed_mul * self.rng.range_f32(0.85, 1.2),
                });
                self.spawned += 1;
                let gap = (cfg.spawn_base - self.spawned as f32 * 0.16).max(cfg.spawn_min);
                self.zombie_timer += gap;
            }
        }

        if self.zombies.iter().any(|z| z.x < 0.0) {
            self.status = Status::Lost;
            self.set_notice("A zombie reached your house!");
            self.events.push(GameEvent::Lose);
        } else if self.spawned >= cfg.zombies && self.zombies.is_empty() {
            self.status = Status::Won;
            self.set_notice(if self.level == MAX_LEVELS {
                "Campaign cleared!"
            } else {
                "Level cleared!"
            });
            self.events.push(GameEvent::Win);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn planting_costs_sun_and_blocks_occupied_tiles() {
        let mut g = Game::new(1);
        g.select(PlantKind::Peashooter);
        g.try_plant(2, 0);
        assert_eq!(g.sun, 100);
        assert_eq!(g.plants.len(), 1);

        g.try_plant(2, 0);
        assert_eq!(g.sun, 100);
        assert_eq!(g.plants.len(), 1);

        g.select(PlantKind::Sunflower);
        g.try_plant(2, 1);
        assert_eq!(g.sun, 50);
        assert_eq!(g.plants.len(), 2);
    }

    #[test]
    fn peashooter_kills_a_zombie_in_its_row() {
        let mut g = Game::new(2);
        g.try_plant(0, 0);
        // Prevent new waves from spawning during this isolated combat test.
        g.spawned = level_config(g.level).zombies;
        g.zombies.push(Zombie {
            x: 5.0,
            row: 0,
            hp: ZOMBIE_HP,
            max_hp: ZOMBIE_HP,
            walk_phase: 0.0,
            walking: false,
            attack: EAT_INTERVAL,
            speed: 0.0,
        });

        for _ in 0..200 {
            g.update(0.05);
        }

        assert!(g.zombies.is_empty());
        assert_eq!(g.killed, 1);
    }

    #[test]
    fn zombie_reaching_the_left_side_loses_the_game() {
        let mut g = Game::new(3);
        g.zombies.push(Zombie {
            x: 0.05,
            row: 1,
            hp: ZOMBIE_HP,
            max_hp: ZOMBIE_HP,
            walk_phase: 0.0,
            walking: false,
            attack: EAT_INTERVAL,
            speed: 1.0,
        });
        g.update(0.1);
        assert_eq!(g.status, Status::Lost);
    }

    #[test]
    fn sky_suns_arrive_often_enough_to_fund_defense() {
        let mut g = Game::new(7);
        let start = g.sun;
        for _ in 0..360 {
            g.update(0.05);
        }
        assert!(g.suns.len() >= 2);
        while !g.suns.is_empty() {
            g.collect_nearest();
        }
        assert!(g.sun >= start + 2 * SUN_VALUE);
    }

    #[test]
    fn suns_can_be_collected_by_click_or_keyboard() {
        let mut g = Game::new(4);
        let start = g.sun;
        g.suns.push(Sun {
            col: 2,
            y: 3.0,
            target_y: 3.0,
            life: 10.0,
        });

        assert!(g.collect_at(3, 2));
        assert_eq!(g.sun, start + SUN_VALUE);
        assert!(g.suns.is_empty());

        g.suns.push(Sun {
            col: 4,
            y: 2.0,
            target_y: 2.0,
            life: 10.0,
        });
        g.cursor = (2, 4);
        g.collect_nearest();
        assert_eq!(g.sun, start + 2 * SUN_VALUE);
    }

    #[test]
    fn clearing_the_last_zombie_wins_after_all_waves() {
        let mut g = Game::new(5);
        g.spawned = level_config(g.level).zombies;
        g.zombies.push(Zombie {
            x: 1.0,
            row: 0,
            hp: 0.0,
            max_hp: ZOMBIE_HP,
            walk_phase: 0.0,
            walking: false,
            attack: EAT_INTERVAL,
            speed: 0.0,
        });
        g.update(0.016);
        assert_eq!(g.status, Status::Won);
    }

    #[test]
    fn levels_get_harder() {
        let a = level_config(1);
        let b = level_config(2);
        let c = level_config(3);
        assert!(a.zombies < b.zombies && b.zombies < c.zombies);
        assert!(a.speed_mul < b.speed_mul && b.speed_mul < c.speed_mul);
        assert!(a.zombie_hp < b.zombie_hp && b.zombie_hp < c.zombie_hp);
        assert!(a.spawn_base > b.spawn_base && b.spawn_base > c.spawn_base);
    }

    #[test]
    fn winning_advances_to_the_next_level() {
        let mut g = Game::new(9);
        g.status = Status::Won;
        g.spawned = 3;
        g.killed = 3;
        g.next_level();

        assert_eq!(g.level, 2);
        assert_eq!(g.status, Status::Playing);
        assert_eq!(g.spawned, 0);
        assert_eq!(g.killed, 0);
        assert_eq!(g.sun, level_config(2).start_sun);
        assert!(g.plants.is_empty() && g.zombies.is_empty());
    }

    #[test]
    fn clearing_final_level_loops_back_to_level_one() {
        let mut g = Game::with_level(11, MAX_LEVELS);
        g.status = Status::Won;
        g.next_level();
        assert_eq!(g.level, 1);
    }

    #[test]
    fn next_level_is_ignored_until_a_win() {
        let mut g = Game::new(12);
        g.next_level();
        assert_eq!(g.level, 1);
        assert_eq!(g.status, Status::Playing);
    }

    #[test]
    fn zombie_walk_phase_tracks_motion_not_chewing() {
        let mut g = Game::new(13);
        g.spawned = level_config(g.level).zombies;
        g.zombies.push(Zombie {
            x: 5.0,
            row: 1,
            hp: ZOMBIE_HP,
            max_hp: ZOMBIE_HP,
            walk_phase: 0.0,
            walking: false,
            attack: EAT_INTERVAL,
            speed: 0.0,
        });

        g.update(0.1);
        assert!(g.zombies[0].walking);
        assert!(g.zombies[0].walk_phase > 0.0);

        // A plant in the same tile makes the zombie stop and chew instead.
        g.try_plant(1, 5);
        g.update(0.1);
        let phase = g.zombies[0].walk_phase;
        assert!(!g.zombies[0].walking);
        g.update(0.1);
        assert_eq!(g.zombies[0].walk_phase, phase);
    }

    #[test]
    fn particle_effects_have_a_duration_and_expire() {
        let mut g = Game::new(14);
        g.try_plant(0, 0);
        let planted = g.effects.last().copied().unwrap();
        assert_eq!(planted.kind, EffectKind::Plant);
        assert_eq!(planted.life, planted.duration);
        assert!((planted.x - 0.5).abs() < 0.01);
        assert!((planted.y - 0.5).abs() < 0.01);

        for _ in 0..20 {
            g.update(0.05);
        }
        assert!(!g.effects.iter().any(|e| e.kind == EffectKind::Plant));
    }

    #[test]
    fn dying_zombie_emits_a_death_burst() {
        let mut g = Game::new(15);
        g.spawned = level_config(g.level).zombies;
        g.zombies.push(Zombie {
            x: 3.0,
            row: 2,
            hp: 0.0,
            max_hp: ZOMBIE_HP,
            walk_phase: 0.0,
            walking: false,
            attack: EAT_INTERVAL,
            speed: 0.0,
        });
        g.update(0.016);
        assert!(
            g.effects
                .iter()
                .any(|e| e.kind == EffectKind::Death && (e.x - 3.5).abs() < 0.01)
        );
    }
}
