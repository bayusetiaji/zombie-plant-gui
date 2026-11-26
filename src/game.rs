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
const SHOVEL_REFUND: i32 = 25;
pub const BIG_WAVE_EVERY: usize = 4;
const MOWER_SPEED: f32 = 14.0;
const FIRST_STAR_WAVE: usize = 10;
const SECOND_STAR_WAVE: usize = 20;

#[derive(Clone, Copy, Debug)]
pub struct LevelConfig {
    pub start_sun: i32,
    pub first_spawn: f32,
    pub spawn_base: f32,
    pub spawn_min: f32,
    pub speed_mul: f32,
    pub zombie_hp: f32,
    pub hp_growth: f32,
    pub speed_growth: f32,
}

pub fn level_config(level: usize) -> LevelConfig {
    match level.clamp(1, MAX_LEVELS) {
        1 => LevelConfig {
            start_sun: START_SUN,
            first_spawn: 6.0,
            spawn_base: 4.4,
            spawn_min: 2.6,
            speed_mul: 1.0,
            zombie_hp: ZOMBIE_HP,
            hp_growth: 4.0,
            speed_growth: 0.008,
        },
        2 => LevelConfig {
            start_sun: START_SUN,
            first_spawn: 5.0,
            spawn_base: 3.9,
            spawn_min: 2.3,
            speed_mul: 1.15,
            zombie_hp: 110.0,
            hp_growth: 5.0,
            speed_growth: 0.010,
        },
        _ => LevelConfig {
            start_sun: START_SUN,
            first_spawn: 4.0,
            spawn_base: 3.5,
            spawn_min: 2.0,
            speed_mul: 1.3,
            zombie_hp: 120.0,
            hp_growth: 6.0,
            speed_growth: 0.012,
        },
    }
}

/// Friendly names for the three endless difficulties.
pub fn level_name(level: usize) -> &'static str {
    match level.clamp(1, MAX_LEVELS) {
        1 => "EASY",
        2 => "MEDIUM",
        _ => "HARD",
    }
}

/// Wave a player must reach on the previous difficulty to unlock this one.
pub fn unlock_wave(level: usize) -> usize {
    match level.clamp(1, MAX_LEVELS) {
        1 => 0,
        2 => FIRST_STAR_WAVE,
        _ => SECOND_STAR_WAVE - 5,
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
    pub flash: f32,
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
    Lost,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameEvent {
    Select,
    Plant,
    Shovel,
    Shoot,
    Hit,
    ZombieDie,
    Sun,
    WaveWarning,
    Lawnmower,
    Milestone,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FloatKind {
    Sun,
    Spend,
    Kill,
}

/// Small rising number, e.g. `+40` when a sun is collected.
#[derive(Clone, Copy, Debug)]
pub struct FloatingText {
    pub kind: FloatKind,
    pub value: i32,
    pub x: f32,
    pub y: f32,
    pub life: f32,
    pub duration: f32,
}

/// A lawnmower sweeping across one row after a zombie escaped.
#[derive(Clone, Copy, Debug)]
pub struct MowerRun {
    pub row: usize,
    pub x: f32,
    pub life: f32,
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
    pub paused: bool,
    pub wave: usize,
    pub spawned: usize,
    pub killed: usize,
    pub elapsed: f32,
    pub notice: Option<(String, f32)>,
    pub events: Vec<GameEvent>,
    pub effects: Vec<Effect>,
    pub floaters: Vec<FloatingText>,
    pub lawnmowers: [bool; ROWS],
    pub mower_runs: Vec<MowerRun>,
    pub wave_warning: Option<(String, f32)>,
    sky_timer: f32,
    zombie_timer: f32,
    pending_big_wave: bool,
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
            paused: false,
            wave: 1,
            spawned: 0,
            killed: 0,
            elapsed: 0.0,
            notice: Some((
                format!(
                    "{} - endless survival. Pick a plant (1/2/3), then Space.",
                    level_name(level)
                ),
                7.0,
            )),
            events: Vec::new(),
            effects: Vec::new(),
            floaters: Vec::new(),
            lawnmowers: [true; ROWS],
            mower_runs: Vec::new(),
            wave_warning: None,
            sky_timer: 3.0,
            zombie_timer: cfg.first_spawn,
            pending_big_wave: false,
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

    /// Toggle the frozen state. Returns the new paused value when the toggle
    /// was accepted (ignored once the run is over).
    pub fn toggle_pause(&mut self) -> bool {
        if self.status != Status::Playing {
            return false;
        }
        self.paused = !self.paused;
        self.paused
    }

    /// Score rewards kills, leftover sun, wave progress, and survival time.
    pub fn score(&self) -> i32 {
        self.killed as i32 * 100 + self.sun * 2 + self.wave as i32 * 250 + self.elapsed as i32 * 5
    }

    fn set_notice(&mut self, msg: impl Into<String>) {
        self.notice = Some((msg.into(), 2.5));
    }

    fn float(&mut self, kind: FloatKind, value: i32, x: f32, y: f32) {
        let duration = 0.8;
        self.floaters.push(FloatingText {
            kind,
            value,
            x,
            y,
            life: duration,
            duration,
        });
    }

    /// Stars are awarded when the run ends: wave milestones + a clean defense.
    pub fn stars(&self) -> u8 {
        if self.status != Status::Lost {
            return 0;
        }
        let mut stars = 0;
        if self.wave >= FIRST_STAR_WAVE {
            stars += 1;
        }
        if self.wave >= SECOND_STAR_WAVE {
            stars += 1;
        }
        if self.lawnmowers.iter().all(|&ready| ready) {
            stars += 1;
        }
        stars
    }

    /// Remove a plant with a small sun refund. Returns true when one was removed.
    pub fn try_shovel(&mut self, row: usize, col: usize) -> bool {
        if self.paused || self.status != Status::Playing || row >= ROWS || col >= COLS {
            return false;
        }
        let Some(i) = self
            .plants
            .iter()
            .position(|p| p.row == row && p.col == col)
        else {
            return false;
        };
        self.plants.remove(i);
        self.sun += SHOVEL_REFUND;
        self.set_notice(format!("Shoveled plant (+{SHOVEL_REFUND} sun)"));
        self.events.push(GameEvent::Shovel);
        self.effects.push(effect(
            EffectKind::Plant,
            col as f32 + 0.5,
            row as f32 + 0.5,
        ));
        self.float(
            FloatKind::Sun,
            SHOVEL_REFUND,
            col as f32 + 0.5,
            row as f32 + 0.35,
        );
        true
    }

    pub fn select(&mut self, kind: PlantKind) {
        if self.paused || self.status != Status::Playing {
            return;
        }
        self.selected = kind;
        self.set_notice(format!("Selected {} ({} sun)", kind.name(), kind.cost()));
        self.events.push(GameEvent::Select);
    }

    pub fn try_plant(&mut self, row: usize, col: usize) {
        if self.paused || self.status != Status::Playing || row >= ROWS || col >= COLS {
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
            flash: 0.28,
            timer: 0.0,
        });
        self.events.push(GameEvent::Plant);
        self.effects.push(effect(
            EffectKind::Plant,
            col as f32 + 0.5,
            row as f32 + 0.5,
        ));
        self.float(
            FloatKind::Spend,
            -kind.cost(),
            col as f32 + 0.5,
            row as f32 + 0.35,
        );
    }

    /// Returns true when a falling sun was collected at this tile.
    pub fn collect_at(&mut self, row: usize, col: usize) -> bool {
        if self.paused || self.status != Status::Playing {
            return false;
        }
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
            self.float(
                FloatKind::Sun,
                SUN_VALUE,
                col as f32 + 0.5,
                row as f32 + 0.35,
            );
            true
        } else {
            false
        }
    }

    /// Keyboard fallback: collect the sun closest to the cursor.
    pub fn collect_nearest(&mut self) {
        if self.paused || self.status != Status::Playing {
            return;
        }
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
            self.float(
                FloatKind::Sun,
                SUN_VALUE,
                sun.col as f32 + 0.5,
                sun.y + 0.35,
            );
        }
    }

    pub fn update(&mut self, dt: f32) {
        if self.paused || self.status != Status::Playing {
            return;
        }
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
        for f in &mut self.floaters {
            f.life -= dt;
        }
        self.floaters.retain(|f| f.life > 0.0);

        let mut warning_expired = false;
        if let Some((_, t)) = &mut self.wave_warning {
            *t -= dt;
            if *t <= 0.0 {
                warning_expired = true;
            }
        }
        if warning_expired {
            self.wave_warning = None;
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
            p.flash = (p.flash - dt).max(0.0);
            p.timer -= dt;
            if p.timer > 0.0 {
                continue;
            }
            match p.kind {
                PlantKind::Sunflower => {
                    p.timer += SUN_COOLDOWN;
                    p.flash = 0.32;
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
                        p.flash = 0.20;
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
                    p.flash = 0.25;
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
            self.float(FloatKind::Kill, 100, x, row as f32 + 0.3);
        }

        // Endless spawning: every fourth spawn is a surge that starts the
        // next wave. Each wave is a little faster, tougher, and more frequent.
        let cfg = level_config(self.level);
        self.zombie_timer -= dt;
        let surge_next = (self.spawned + 1) % BIG_WAVE_EVERY == 0;
        if !self.pending_big_wave && surge_next && self.zombie_timer <= 1.2 {
            self.pending_big_wave = true;
            self.wave_warning = Some((format!("WAVE {} INCOMING!", self.wave + 1), 1.2));
            self.events.push(GameEvent::WaveWarning);
        }
        if self.zombie_timer <= 0.0 {
            let count = if self.pending_big_wave { 2 } else { 1 };
            for _ in 0..count {
                let row = self.rng.range(ROWS);
                let hp = cfg.zombie_hp + self.wave as f32 * cfg.hp_growth;
                let speed = (ZOMBIE_SPEED * cfg.speed_mul + self.wave as f32 * cfg.speed_growth)
                    * self.rng.range_f32(0.85, 1.2);
                self.zombies.push(Zombie {
                    x: COLS as f32 - 0.1,
                    row,
                    hp,
                    max_hp: hp,
                    walk_phase: self.rng.range_f32(0.0, 2.0),
                    walking: true,
                    attack: EAT_INTERVAL,
                    speed,
                });
                self.spawned += 1;
            }
            if self.pending_big_wave {
                self.wave += 1;
                if self.wave == FIRST_STAR_WAVE || self.wave == SECOND_STAR_WAVE {
                    self.events.push(GameEvent::Milestone);
                    self.set_notice(format!("Wave {} reached!", self.wave));
                }
            }
            self.pending_big_wave = false;
            self.wave_warning = None;
            let floor = (cfg.spawn_min * 0.6).max(0.9);
            let gap = (cfg.spawn_base - self.wave as f32 * 0.18).max(floor);
            self.zombie_timer += gap;
        }

        // A zombie crossing the left edge triggers that row's one-shot
        // lawnmower. It only catches the zombie that actually reached the
        // edge; other zombies still marching in that row survive. A second
        // escape in the same row loses the game, so mowers are a safety net
        // rather than a free way to clear every wave.
        let mut escaped_rows: Vec<usize> = self
            .zombies
            .iter()
            .filter(|z| z.x < 0.0)
            .map(|z| z.row)
            .collect();
        escaped_rows.sort_unstable();
        escaped_rows.dedup();
        let mut lost = false;
        for row in escaped_rows {
            if !self.lawnmowers[row] {
                lost = true;
                continue;
            }
            let Some(i) = self.zombies.iter().position(|z| z.row == row && z.x < 0.0) else {
                continue;
            };
            let x = self.zombies[i].x + 0.5;
            self.zombies.remove(i);
            self.lawnmowers[row] = false;
            self.killed += 1;
            self.events.push(GameEvent::ZombieDie);
            self.effects
                .push(effect(EffectKind::Death, x, row as f32 + 0.5));
            self.float(FloatKind::Kill, 100, x, row as f32 + 0.3);
            self.mower_runs.push(MowerRun {
                row,
                x: -0.6,
                life: 1.0,
            });
            self.events.push(GameEvent::Lawnmower);
            self.set_notice(format!("Lawnmower caught a zombie in row {}!", row + 1));
        }
        for run in &mut self.mower_runs {
            run.x += MOWER_SPEED * dt;
            run.life -= dt;
        }
        self.mower_runs
            .retain(|run| run.life > 0.0 && run.x < COLS as f32 + 1.0);

        if lost {
            self.status = Status::Lost;
            self.set_notice(format!(
                "{} run ended at wave {}!",
                level_name(self.level),
                self.wave
            ));
            self.events.push(GameEvent::Lose);
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
        g.zombie_timer = 999.0;
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
        // The row's lawnmower is already spent, so this escape is fatal.
        g.lawnmowers[1] = false;
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
    fn killing_every_visible_zombie_does_not_end_an_endless_run() {
        let mut g = Game::new(5);
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
        assert_eq!(g.status, Status::Playing);
        assert!(g.zombies.is_empty());
    }

    #[test]
    fn difficulties_get_harder() {
        let a = level_config(1);
        let b = level_config(2);
        let c = level_config(3);
        assert!(a.speed_mul < b.speed_mul && b.speed_mul < c.speed_mul);
        assert!(a.zombie_hp < b.zombie_hp && b.zombie_hp < c.zombie_hp);
        assert!(a.hp_growth < b.hp_growth && b.hp_growth < c.hp_growth);
        assert!(a.speed_growth < b.speed_growth && b.speed_growth < c.speed_growth);
        assert!(a.spawn_base > b.spawn_base && b.spawn_base > c.spawn_base);
    }

    #[test]
    fn difficulty_names_and_unlock_waves_are_stable() {
        assert_eq!(level_name(1), "EASY");
        assert_eq!(level_name(2), "MEDIUM");
        assert_eq!(level_name(3), "HARD");
        assert_eq!(unlock_wave(1), 0);
        assert_eq!(unlock_wave(2), 10);
        assert_eq!(unlock_wave(3), 15);
    }

    #[test]
    fn restart_keeps_the_same_difficulty() {
        let mut g = Game::with_level(11, MAX_LEVELS);
        g.wave = 7;
        g.killed = 12;
        g.restart();
        assert_eq!(g.level, MAX_LEVELS);
        assert_eq!(g.wave, 1);
        assert_eq!(g.killed, 0);
        assert_eq!(g.status, Status::Playing);
    }

    #[test]
    fn zombie_walk_phase_tracks_motion_not_chewing() {
        let mut g = Game::new(13);
        g.zombie_timer = 999.0;
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
        g.zombie_timer = 999.0;
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

    #[test]
    fn shovel_removes_a_plant_and_refunds_sun() {
        let mut g = Game::new(16);
        g.try_plant(2, 3);
        let after_plant = g.sun;

        assert!(g.try_shovel(2, 3));
        assert_eq!(g.sun, after_plant + SHOVEL_REFUND);
        assert!(g.plants.is_empty());
        assert!(!g.try_shovel(2, 3));
        assert!(g.events.contains(&GameEvent::Shovel));
        assert!(
            g.floaters
                .iter()
                .any(|f| f.kind == FloatKind::Sun && f.value == SHOVEL_REFUND)
        );
    }

    #[test]
    fn stars_come_from_wave_milestones_and_a_clean_defense() {
        let mut g = Game::new(17);
        g.status = Status::Lost;
        g.wave = 1;
        assert_eq!(g.stars(), 1);

        g.wave = FIRST_STAR_WAVE;
        assert_eq!(g.stars(), 2);

        g.lawnmowers[0] = false;
        assert_eq!(g.stars(), 1);

        g.wave = SECOND_STAR_WAVE;
        assert_eq!(g.stars(), 2);

        g.lawnmowers = [true; ROWS];
        assert_eq!(g.stars(), 3);

        g.status = Status::Playing;
        assert_eq!(g.stars(), 0);
    }

    #[test]
    fn big_wave_warns_then_starts_the_next_wave() {
        let mut g = Game::new(18);
        g.spawned = BIG_WAVE_EVERY - 1;
        g.zombie_timer = 0.5;

        g.update(0.1);
        assert!(g.wave_warning.is_some());
        assert_eq!(g.wave_warning.as_ref().unwrap().0, "WAVE 2 INCOMING!");
        assert!(g.events.contains(&GameEvent::WaveWarning));

        g.update(0.5);
        assert_eq!(g.spawned, BIG_WAVE_EVERY + 1);
        assert_eq!(g.zombies.len(), 2);
        assert_eq!(g.wave, 2);
        assert!(g.wave_warning.is_none());

        g.zombie_timer = 0.0;
        g.update(0.016);
        assert!(g.spawned > BIG_WAVE_EVERY + 1);
    }

    #[test]
    fn wave_ten_emits_a_milestone_event() {
        let mut g = Game::new(24);
        g.wave = FIRST_STAR_WAVE - 1;
        g.pending_big_wave = true;
        g.zombie_timer = 0.0;

        g.update(0.016);
        assert_eq!(g.wave, FIRST_STAR_WAVE);
        assert!(g.events.contains(&GameEvent::Milestone));
    }

    #[test]
    fn lawnmower_saves_a_row_only_once() {
        let mut g = Game::new(19);
        g.zombies.push(Zombie {
            x: -0.05,
            row: 2,
            hp: ZOMBIE_HP,
            max_hp: ZOMBIE_HP,
            walk_phase: 0.0,
            walking: false,
            attack: EAT_INTERVAL,
            speed: 0.0,
        });

        g.update(0.016);
        assert!(!g.lawnmowers[2]);
        assert!(g.zombies.is_empty());
        assert_eq!(g.killed, 1);
        assert_eq!(g.mower_runs.len(), 1);
        assert_eq!(g.status, Status::Playing);

        g.zombies.push(Zombie {
            x: -0.05,
            row: 2,
            hp: ZOMBIE_HP,
            max_hp: ZOMBIE_HP,
            walk_phase: 0.0,
            walking: false,
            attack: EAT_INTERVAL,
            speed: 0.0,
        });
        g.update(0.016);
        assert_eq!(g.status, Status::Lost);
    }

    #[test]
    fn lawnmower_only_catches_the_zombie_that_crossed() {
        let mut g = Game::new(21);
        for x in [-0.05, 4.0] {
            g.zombies.push(Zombie {
                x,
                row: 2,
                hp: ZOMBIE_HP,
                max_hp: ZOMBIE_HP,
                walk_phase: 0.0,
                walking: false,
                attack: EAT_INTERVAL,
                speed: 0.0,
            });
        }

        g.update(0.016);
        assert!(!g.lawnmowers[2]);
        assert_eq!(g.zombies.len(), 1);
        assert!((g.zombies[0].x - 4.0).abs() < 0.01);
        assert_eq!(g.killed, 1);
        assert_eq!(g.status, Status::Playing);
    }

    #[test]
    fn two_escapes_in_one_row_beat_a_single_mower() {
        let mut g = Game::new(22);
        for x in [-0.2, -0.1] {
            g.zombies.push(Zombie {
                x,
                row: 1,
                hp: ZOMBIE_HP,
                max_hp: ZOMBIE_HP,
                walk_phase: 0.0,
                walking: false,
                attack: EAT_INTERVAL,
                speed: 0.0,
            });
        }

        g.update(0.016);
        assert_eq!(g.zombies.len(), 1);
        assert_eq!(g.status, Status::Playing);

        g.update(0.016);
        assert_eq!(g.status, Status::Lost);
    }

    #[test]
    fn doing_nothing_loses_before_reaching_late_waves() {
        for level in 1..=MAX_LEVELS {
            let mut g = Game::new((30 + level) as u64);
            for _ in 0..6000 {
                g.update(0.05);
                if g.status != Status::Playing {
                    break;
                }
            }
            assert_eq!(g.status, Status::Lost, "level {level} should not be free");
        }
    }

    #[test]
    fn pause_freezes_time_spawns_and_blocks_actions() {
        let mut g = Game::new(25);
        g.try_plant(0, 0);
        let sun = g.sun;
        let plants = g.plants.len();
        let selected = g.selected;
        let elapsed = g.elapsed;
        let spawned = g.spawned;
        let effect_life = g.effects.first().map(|e| e.life);

        assert!(g.toggle_pause());
        g.update(5.0);
        assert_eq!(g.elapsed, elapsed);
        assert_eq!(g.spawned, spawned);
        assert_eq!(g.effects.first().map(|e| e.life), effect_life);

        g.select(PlantKind::Sunflower);
        g.try_plant(1, 1);
        assert!(!g.try_shovel(0, 0));
        assert!(!g.collect_at(0, 0));
        assert_eq!(g.selected, selected);
        assert_eq!(g.plants.len(), plants);
        assert_eq!(g.sun, sun);

        assert!(!g.toggle_pause());
        g.update(0.1);
        assert!(g.elapsed > elapsed);
    }

    #[test]
    fn pause_is_ignored_after_the_run_ends() {
        let mut g = Game::new(26);
        g.status = Status::Lost;
        assert!(!g.toggle_pause());
        assert!(!g.paused);
    }

    #[test]
    fn floating_numbers_rise_then_expire() {
        let mut g = Game::new(20);
        g.suns.push(Sun {
            col: 4,
            y: 2.0,
            target_y: 2.0,
            life: 10.0,
        });
        assert!(g.collect_at(2, 4));
        assert!(g.floaters.iter().any(|f| f.value == SUN_VALUE));
        for _ in 0..20 {
            g.update(0.05);
        }
        assert!(g.floaters.is_empty());
    }
}
