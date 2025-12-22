#[derive(Debug)]
pub struct Minstd {
    u: u64,
}

impl Default for Minstd {
    fn default() -> Self {
        Self { u: 1 }
    }
}

impl Minstd {
    pub fn next(&mut self) -> u32 {
        self.u = (self.u * 48_271) % 2_147_483_647;
        self.u as u32
    }

    pub fn next_idx(&mut self, n: u32) -> u32 {
        self.next() % n
    }

    pub fn next_01_prom(&mut self) -> f32 {
        self.next_idx(1000) as f32 * 0.001
    }

    pub fn next_bool(&mut self) -> bool {
        self.next() % 2 == 1
    }
}
