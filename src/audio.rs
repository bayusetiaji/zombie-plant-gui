use crate::rng::Rng;
use rodio::buffer::SamplesBuffer;
use rodio::{DeviceSinkBuilder, MixerDeviceSink};
use std::num::NonZero;

const SAMPLE_RATE: u32 = 44_100;
const TAU: f32 = std::f32::consts::TAU;

#[derive(Clone, Copy)]
enum Wave {
    Sine,
    Square,
    Triangle,
    Saw,
}

#[derive(Clone, Copy)]
pub enum Sound {
    Select,
    Plant,
    Shoot,
    Hit,
    ZombieDie,
    Sun,
    Win,
    Lose,
}

pub struct Audio {
    sink: Option<MixerDeviceSink>,
    muted: bool,
    step: usize,
    step_timer: f32,
}

impl Audio {
    pub fn new() -> Self {
        let sink = match DeviceSinkBuilder::open_default_sink() {
            Ok(mut sink) => {
                sink.log_on_drop(false);
                Some(sink)
            }
            Err(_) => None,
        };
        Self {
            sink,
            muted: false,
            step: 0,
            step_timer: 0.0,
        }
    }

    pub fn available(&self) -> bool {
        self.sink.is_some()
    }

    pub fn muted(&self) -> bool {
        self.muted
    }

    pub fn toggle_mute(&mut self) -> bool {
        self.muted = !self.muted;
        self.muted
    }

    pub fn play(&self, sound: Sound) {
        if self.muted {
            return;
        }
        let Some(sink) = &self.sink else { return };
        let mut rng = Rng::new(self.step as u64 ^ 0xA5A5_5A5A);
        let samples = match sound {
            Sound::Select => tone(660.0, 0.06, 0.05, Wave::Square),
            Sound::Plant => sweep(220.0, 90.0, 0.14, 0.10, Wave::Triangle),
            Sound::Shoot => sweep(900.0, 420.0, 0.07, 0.06, Wave::Sine),
            Sound::Hit => concat(vec![
                noise(0.07, 0.07, &mut rng),
                tone(140.0, 0.06, 0.05, Wave::Square),
            ]),
            Sound::ZombieDie => sweep(320.0, 70.0, 0.28, 0.11, Wave::Saw),
            Sound::Sun => concat(vec![
                tone(784.0, 0.07, 0.07, Wave::Sine),
                tone(988.0, 0.07, 0.07, Wave::Sine),
                tone(1319.0, 0.10, 0.07, Wave::Sine),
            ]),
            Sound::Win => concat(vec![
                tone(523.25, 0.12, 0.07, Wave::Triangle),
                tone(659.25, 0.12, 0.07, Wave::Triangle),
                tone(783.99, 0.12, 0.07, Wave::Triangle),
                tone(1046.5, 0.25, 0.07, Wave::Triangle),
            ]),
            Sound::Lose => concat(vec![
                tone(392.0, 0.18, 0.07, Wave::Saw),
                tone(329.63, 0.18, 0.07, Wave::Saw),
                tone(261.63, 0.18, 0.07, Wave::Saw),
                tone(196.0, 0.30, 0.07, Wave::Saw),
            ]),
        };
        sink.mixer().add(buffer(samples));
    }

    /// Tiny step sequencer for background music. `intense` speeds it up when
    /// several zombies are on screen.
    pub fn tick(&mut self, dt: f32, intense: bool) {
        if self.muted || self.sink.is_none() {
            return;
        }
        self.step_timer -= dt;
        if self.step_timer > 0.0 {
            return;
        }
        let beat = if intense { 0.24 } else { 0.32 };
        self.step_timer += beat;
        const PATTERN: [f32; 8] = [261.63, 329.63, 392.0, 523.25, 392.0, 349.23, 293.66, 329.63];
        let note = PATTERN[self.step % PATTERN.len()];
        self.play_note(note, beat * 0.75, 0.028);
        if self.step % 4 == 0 {
            self.play_note(note / 2.0, beat * 1.6, 0.022);
        }
        self.step += 1;
    }

    fn play_note(&self, freq: f32, dur: f32, vol: f32) {
        if let Some(sink) = &self.sink {
            sink.mixer()
                .add(buffer(tone(freq, dur, vol, Wave::Triangle)));
        }
    }
}

fn buffer(samples: Vec<f32>) -> SamplesBuffer {
    SamplesBuffer::new(
        NonZero::new(1u16).unwrap(),
        NonZero::new(SAMPLE_RATE).unwrap(),
        samples,
    )
}

fn shape(kind: Wave, phase: f32) -> f32 {
    let p = phase - phase.floor();
    match kind {
        Wave::Sine => (p * TAU).sin(),
        Wave::Square => {
            if p < 0.5 {
                1.0
            } else {
                -1.0
            }
        }
        Wave::Triangle => 1.0 - 4.0 * (p - 0.5).abs(),
        Wave::Saw => 2.0 * p - 1.0,
    }
}

fn tone(freq: f32, dur: f32, vol: f32, kind: Wave) -> Vec<f32> {
    let n = (SAMPLE_RATE as f32 * dur) as usize;
    let mut out = Vec::with_capacity(n);
    let mut phase = 0.0;
    for i in 0..n {
        let u = i as f32 / n as f32;
        let envelope = (1.0 - u).powi(2);
        out.push(shape(kind, phase) * envelope * vol);
        phase += freq / SAMPLE_RATE as f32;
    }
    out
}

fn sweep(f0: f32, f1: f32, dur: f32, vol: f32, kind: Wave) -> Vec<f32> {
    let n = (SAMPLE_RATE as f32 * dur) as usize;
    let mut out = Vec::with_capacity(n);
    let mut phase = 0.0;
    for i in 0..n {
        let u = i as f32 / n as f32;
        let freq = f0 + (f1 - f0) * u;
        let envelope = (1.0 - u).powi(2);
        out.push(shape(kind, phase) * envelope * vol);
        phase += freq / SAMPLE_RATE as f32;
    }
    out
}

fn noise(dur: f32, vol: f32, rng: &mut Rng) -> Vec<f32> {
    let n = (SAMPLE_RATE as f32 * dur) as usize;
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let u = i as f32 / n as f32;
        let envelope = (1.0 - u).powi(3);
        let r = rng.next_u64() as f32 / u64::MAX as f32 * 2.0 - 1.0;
        out.push(r * envelope * vol);
    }
    out
}

fn concat(parts: Vec<Vec<f32>>) -> Vec<f32> {
    let gap = SAMPLE_RATE as usize / 60;
    let mut out = Vec::new();
    for (i, part) in parts.into_iter().enumerate() {
        if i > 0 {
            out.extend(std::iter::repeat(0.0).take(gap));
        }
        out.extend(part);
    }
    out
}
