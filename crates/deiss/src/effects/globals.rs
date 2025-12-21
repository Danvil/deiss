use std::time::Instant;

use crate::effects::painter::RunningFourier;

#[derive(Debug, Default)]
pub struct Globals {
    pub rand: MinstdRand,
    pub frame: u64,
    pub last_frame_v: f32,
    pub last_frame_slope: f32,
    pub vol: VolBuffer,
    pub avg_vol_narrow: f32,
    pub avg_vol: f32,
    pub avg_vol_wide: f32,
    pub avg_vol_peaks: f32,
    pub volume_sum: u64,
    pub suggested_dampening: f32,
    pub fourier: RunningFourier,
    pub fps: Fps,
    pub fps_at_last_mode_switch: f32,
    pub big_beat_threshold: f32,
}

#[derive(Debug)]
pub struct Fps {
    start: Instant,
    frames: u32,
}

impl Default for Fps {
    fn default() -> Self {
        Self {
            start: Instant::now(),
            frames: 0,
        }
    }
}

impl Fps {
    pub fn step(&mut self) {
        self.frames += 1;
    }

    pub fn reset(&mut self) -> f32 {
        let dt = (Instant::now() - self.start).as_secs_f32();
        let fps = self.frames as f32 / dt;
        self.start = Instant::now();
        self.frames = 0;
        fps
    }
}

#[derive(Debug)]
pub struct MinstdRand {
    u: u64,
}

impl Default for MinstdRand {
    fn default() -> Self {
        Self { u: 1 }
    }
}

impl MinstdRand {
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

/// Stores current volume and keeps past volume in a ring buffer
#[derive(Debug)]
pub struct VolBuffer {
    index: usize,
    past: Vec<f32>,
    current: f32,
}

impl VolBuffer {
    pub fn push(&mut self, vol: f32) {
        self.index = (self.index + 1) % self.past.len();
        self.past[self.index] = self.current;
        self.current = vol;
    }

    pub fn current(&self) -> f32 {
        self.current
    }
}

impl Default for VolBuffer {
    fn default() -> Self {
        Self {
            index: 0,
            past: vec![0.; 120],
            current: 0.,
        }
    }
}
