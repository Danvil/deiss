use crate::{fx::Chasers, painter::*, utils::Minstd};
use core::ops;
use std::{
    sync::{Arc, Mutex},
    time::Instant,
};

#[derive(Debug, Default)]
pub struct Globals {
    pub rand: Minstd,
    pub frame: u64,
    pub floatframe: f32,
    pub last_frame_v: f32,
    pub last_frame_slope: f32,
    pub vol: VolBuffer,
    pub avg_vol_narrow: f32,
    pub vol_narrow: VolBuffer,
    pub avg_vol: f32,
    pub avg_vol_wide: f32,
    pub avg_vol_peaks: f32,
    pub volume_sum: u64,
    pub suggested_dampening: f32,
    pub fourier: RunningFourier,
    pub fps: Fps,
    pub fps_at_last_mode_switch: f32,
    pub time_scale: f32,
    pub big_beat_threshold: f32,
    pub chaser_offset: f32,
    pub chasers: Arc<Mutex<Chasers>>,
    pub sound_buffer: SoundBuffer,
}

#[derive(Debug, Default)]
pub struct SoundBuffer(Vec<f32>);

impl SoundBuffer {
    pub fn from_vec(vec: Vec<f32>) -> Self {
        SoundBuffer(vec)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn as_slice(&self) -> &[f32] {
        &self.0
    }

    pub fn iter_lch(&self) -> impl Iterator<Item = f32> {
        (0..self.0.len() / 2).map(|i| self.lch(i))
    }

    pub fn iter_rch(&self) -> impl Iterator<Item = f32> {
        (0..self.0.len() / 2).map(|i| self.rch(i))
    }

    pub fn lch(&self, i: usize) -> f32 {
        self.0[2 * i]
    }

    pub fn rch(&self, i: usize) -> f32 {
        self.0[2 * i + 1]
    }
}

impl ops::Index<usize> for SoundBuffer {
    type Output = f32;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

#[derive(Debug)]
pub struct Fps {
    start: Instant,
    frames: u32,
}

impl Default for Fps {
    fn default() -> Self {
        Self { start: Instant::now(), frames: 0 }
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

/// Stores current volume and keeps past volume in a ring buffer
#[derive(Debug)]
pub struct VolBuffer {
    index: usize,
    past: Vec<f32>,
    total: f32,
}

impl VolBuffer {
    pub fn push(&mut self, vol: f32) {
        self.index = (self.index + 1) % self.past.len();
        self.total += vol - self.past[self.index];
        self.past[self.index] = vol;

        // every time we wrap around recompute total to avoid gradual slip due to numeric instability
        if self.index == 0 {
            self.total = self.past.iter().sum::<f32>();
        }
    }

    pub fn len(&self) -> usize {
        self.past.len()
    }

    pub fn current(&self) -> f32 {
        self.past[self.index]
    }

    pub fn mean(&self) -> f32 {
        self.total / self.past.len() as f32
    }

    pub fn variance(&self) -> f32 {
        let mean = self.mean();
        self.past.iter().map(|v| (v - mean).powi(2)).sum::<f32>() / (self.past.len() - 1) as f32
    }

    pub fn std_dev(&self) -> f32 {
        self.variance().sqrt()
    }

    /// Iterates over values from oldest to newest
    pub fn iter(&self) -> impl Iterator<Item = f32> {
        (0..self.past.len()).map(|i| self.past[(i + self.index + 1) % self.past.len()])
    }

    /// Iterates over differences from oldest to newest
    pub fn iter_differences(&self) -> impl Iterator<Item = f32> {
        // current value is stored at index and thus (index-1, index) must be excluded
        (1..self.past.len()).map(|i| {
            let v1 = self.past[(i + self.index) % self.past.len()];
            let v2 = self.past[(i + 1 + self.index) % self.past.len()];
            v2 - v1
        })
    }
}

impl Default for VolBuffer {
    fn default() -> Self {
        Self { index: 0, past: vec![0.; 120], total: 0. }
    }
}
