/// Small deterministic PRNG (xorshift64*) so the game needs no extra dependency.
pub struct Rng {
    state: u64,
}

impl Rng {
    pub fn new(seed: u64) -> Self {
        Self {
            state: seed ^ 0x9E37_79B9_7F4A_7C15,
        }
    }

    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.state = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    pub fn range(&mut self, max: usize) -> usize {
        (self.next_u64() % max as u64) as usize
    }

    pub fn range_f32(&mut self, min: f32, max: f32) -> f32 {
        let unit = (self.next_u64() % 100_000) as f32 / 100_000.0;
        min + unit * (max - min)
    }
}
