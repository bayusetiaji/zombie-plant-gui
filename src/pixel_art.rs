//! Tiny pixel-art sprites defined as character maps and turned into textures.
//! No external image files are needed; every sprite is built from these rows.

use macroquad::prelude::*;

const W: u16 = 16;
const H: u16 = 16;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sprite {
    Sunflower,
    Peashooter,
    Wallnut,
    Zombie,
    ZombieStep,
    Sun,
    Pea,
}

const PALETTE: &[(char, Color)] = &[
    ('.', Color::new(0.0, 0.0, 0.0, 0.0)),
    ('K', Color::new(0.10, 0.13, 0.10, 1.0)),
    ('k', Color::new(0.05, 0.05, 0.05, 1.0)),
    ('G', Color::new(0.20, 0.52, 0.24, 1.0)),
    ('g', Color::new(0.38, 0.72, 0.32, 1.0)),
    ('Y', Color::new(0.96, 0.75, 0.14, 1.0)),
    ('y', Color::new(1.00, 0.91, 0.45, 1.0)),
    ('B', Color::new(0.55, 0.36, 0.18, 1.0)),
    ('b', Color::new(0.72, 0.51, 0.27, 1.0)),
    ('W', Color::new(0.95, 0.95, 0.92, 1.0)),
    ('R', Color::new(0.86, 0.26, 0.24, 1.0)),
    ('O', Color::new(0.96, 0.48, 0.18, 1.0)),
];

fn rows_for(sprite: Sprite) -> &'static [&'static str] {
    match sprite {
        Sprite::Sunflower => &[
            "................",
            "....KKKKKK......",
            "..KKYYYYYYKK....",
            ".KYYYYYYYYYYK...",
            ".KYyYYYYYYYYK...",
            "KYYYYBBBBYYYYK..",
            "KYYyBBBBBBYYYK..",
            "KYYYYBkkBYYYYK..",
            "KYYYYBBBBYYYYK..",
            "KYYyBBBBBBYYYK..",
            "KYYYYBBBBYYYYK..",
            ".KYYYYYYYYYYK...",
            "..KKYYYYYYKK....",
            "....KKGGKK......",
            ".....KGGK.......",
            ".....KGGK.......",
        ],
        Sprite::Peashooter => &[
            "................",
            "....KKKKKK......",
            "...KGGGGGGK.....",
            "..KGGgGGGGGK....",
            "..KGGkkGGGGK....",
            "..KGGGGGGGK.....",
            "..KGGGGGGKKKK...",
            "...KGGGGGKKKKK..",
            "....KKKKKKKKK...",
            ".....KGGK.......",
            ".....KGGK.......",
            "....KGGGGK......",
            "...KGGGGGGK.....",
            "................",
            "................",
            "................",
        ],
        Sprite::Wallnut => &[
            "................",
            "................",
            ".....KKKK.......",
            "...KKBBBBKK.....",
            "..KBBBBBBBBK....",
            ".KBBbBBBBBBBK...",
            ".KBBBBBBBBBBK...",
            ".KBBkkBBkkBBK...",
            ".KBBBBBBBBBBK...",
            ".KBBbBBBBBBBK...",
            "..KBBBBBBBBK....",
            "...KKBBBBKK.....",
            ".....KKKK.......",
            "................",
            "................",
            "................",
        ],
        Sprite::Zombie => &[
            "................",
            ".....KKKKKKK....",
            ".....KOOOOOOK...",
            "....KOWOOWWOOK..",
            "....KOOOOOOOOK..",
            ".....KOOOOOOK...",
            ".....KGGGGGGK...",
            "....KGGGGGgGGK..",
            ".KOOGGGGGGGGK...",
            ".....KGGGGGGK...",
            "......KGK.KGK...",
            "......KGK.KGK...",
            ".......K...K....",
            "................",
            "................",
            "................",
        ],
        // Second walk frame: the other leg swings forward and the arm dips.
        Sprite::ZombieStep => &[
            "................",
            ".....KKKKKKK....",
            ".....KOOOOOOK...",
            "....KOWOOWWOOK..",
            "....KOOOOOOOOK..",
            ".....KOOOOOOK...",
            ".....KGGGGGGK...",
            "...KGGGGGGGGK...",
            ".KOOKGGGGGGGK...",
            ".....KGGGGGGK...",
            "....KGK..KGK....",
            "...KGK....KGK...",
            "..K.K......K.K..",
            "................",
            "................",
            "................",
        ],
        Sprite::Sun => &[
            "................",
            "....Y.Y.Y.......",
            ".....YYY........",
            "..Y.YYYYY.Y.....",
            "...YYYYYYY......",
            "..YYYYyyyYYY....",
            "...YYYYYYY......",
            "..Y.YYYYY.Y.....",
            ".....YYY........",
            "....Y.Y.Y.......",
            "................",
            "................",
            "................",
            "................",
            "................",
            "................",
        ],
        Sprite::Pea => &[
            "................",
            "................",
            "................",
            ".....KKKK.......",
            "....KGGggK......",
            "...KGGgggGK.....",
            "...KGGgggGK.....",
            "....KGGggK......",
            ".....KKKK.......",
            "................",
            "................",
            "................",
            "................",
            "................",
            "................",
            "................",
        ],
    }
}

fn texture(rows: &[&str]) -> Texture2D {
    let mut bytes = Vec::with_capacity(W as usize * H as usize * 4);
    for y in 0..H {
        let row = rows.get(y as usize).copied().unwrap_or("");
        let mut chars = row.chars();
        for _ in 0..W {
            let ch = chars.next().unwrap_or('.');
            let color = PALETTE
                .iter()
                .find(|(c, _)| *c == ch)
                .map(|(_, c)| *c)
                .unwrap_or(Color::new(0.0, 0.0, 0.0, 0.0));
            bytes.extend_from_slice(&[
                (color.r * 255.0) as u8,
                (color.g * 255.0) as u8,
                (color.b * 255.0) as u8,
                (color.a * 255.0) as u8,
            ]);
        }
    }
    let image = Image {
        bytes,
        width: W,
        height: H,
    };
    let texture = Texture2D::from_image(&image);
    texture.set_filter(FilterMode::Nearest);
    texture
}

pub struct Sprites {
    inner: [Texture2D; 7],
}

impl Sprites {
    pub fn load() -> Self {
        Self {
            inner: [
                texture(rows_for(Sprite::Sunflower)),
                texture(rows_for(Sprite::Peashooter)),
                texture(rows_for(Sprite::Wallnut)),
                texture(rows_for(Sprite::Zombie)),
                texture(rows_for(Sprite::ZombieStep)),
                texture(rows_for(Sprite::Sun)),
                texture(rows_for(Sprite::Pea)),
            ],
        }
    }

    fn get(&self, sprite: Sprite) -> &Texture2D {
        &self.inner[sprite as usize]
    }

    /// Draw a sprite centered in a `size x size` box.
    pub fn draw_centered(&self, sprite: Sprite, cx: f32, cy: f32, size: f32) {
        draw_texture_ex(
            self.get(sprite),
            cx - size / 2.0,
            cy - size / 2.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(size, size)),
                ..Default::default()
            },
        );
    }

    pub fn draw_box(&self, sprite: Sprite, x: f32, y: f32, size: f32) {
        self.draw_centered(sprite, x + size / 2.0, y + size / 2.0, size);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_sprite_has_sixteen_rows() {
        for sprite in [
            Sprite::Sunflower,
            Sprite::Peashooter,
            Sprite::Wallnut,
            Sprite::Zombie,
            Sprite::ZombieStep,
            Sprite::Sun,
            Sprite::Pea,
        ] {
            assert_eq!(rows_for(sprite).len(), H as usize);
        }
    }

    #[test]
    fn pixel_maps_only_use_known_colors() {
        for sprite in [
            Sprite::Sunflower,
            Sprite::Peashooter,
            Sprite::Wallnut,
            Sprite::Zombie,
            Sprite::ZombieStep,
            Sprite::Sun,
            Sprite::Pea,
        ] {
            for row in rows_for(sprite) {
                for ch in row.chars() {
                    assert!(
                        PALETTE.iter().any(|(c, _)| *c == ch),
                        "unknown pixel color '{ch}' in {sprite:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn zombie_arm_reaches_left_toward_the_plants() {
        for sprite in [Sprite::Zombie, Sprite::ZombieStep] {
            let xs: Vec<usize> = rows_for(sprite)
                .iter()
                .flat_map(|row| {
                    row.char_indices()
                        .filter(|(_, ch)| *ch != '.')
                        .map(|(x, _)| x)
                })
                .collect();
            assert_eq!(xs.iter().min(), Some(&1), "{sprite:?} arm points left");
            assert_eq!(xs.iter().max(), Some(&13));
        }
    }

    #[test]
    fn walk_frames_are_different() {
        assert_ne!(
            rows_for(Sprite::Zombie),
            rows_for(Sprite::ZombieStep),
            "walking needs two distinct poses"
        );
    }
}
