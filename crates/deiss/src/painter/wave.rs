use crate::{
    painter::{Globals, ModeId, Settings, WaveformId, color_gen},
    utils::{Rgba, RgbaImage, Vec2i32},
};

pub struct Wave {
    beat_mode: bool,
    big_beat_threshold: f32,
    big_beat: bool,
}

const USE_BEAT_DETECTION: bool = true;

pub const WAVE_5_BLEND_RANGE: usize = 50;
pub const WAVE_5_SIZE: usize = 314;

impl Wave {
    pub fn new(g: &Globals) -> Self {
        Self { beat_mode: false, big_beat_threshold: 1.1, big_beat: false }
    }

    pub fn render(
        &mut self,
        img: &mut RgbaImage,
        center: Vec2i32,
        mode: ModeId,
        waveform: WaveformId,
        s: &Settings,
        g: &Globals,
    ) {
        let mut base = (g.vol.current() * 6.0 - g.avg_vol * 3.5) * 10. - 40.;

        let avg_vol_uniform = g.vol_narrow.mean();

        // Beat mode detection
        {
            let mut beat_strength = g
                .vol_narrow
                .iter_differences()
                .map(|d| (d.abs() - avg_vol_uniform * 0.15).max(0.))
                .sum::<f32>()
                / (g.vol_narrow.len() - 1) as f32;
            beat_strength *= 10.;
            if avg_vol_uniform < 10. {
                beat_strength = 0.;
            }

            if beat_strength > 90. + 19. {
                self.beat_mode = true;
            }
            if beat_strength < 90. - 19. {
                self.beat_mode = false;
            }
        }

        // beat-mode brightness
        let mean = g.vol_narrow.mean();
        let std_dev = g.vol_narrow.std_dev();

        // Note that we use the latest 1/3. The original just used the first 1/3 in the ring
        // buffer no matter the current index of the current value.
        let max_vol = g
            .vol_narrow
            .iter()
            .take((2 * g.vol_narrow.len()) / 3)
            .reduce(|acc, v| acc.max(v))
            .unwrap();
        self.big_beat = g.avg_vol_narrow > max_vol * self.big_beat_threshold;

        // There might be some code missing in the original.
        // TODO For now something simple based on mean and std_dev
        let brite_scale = (g.avg_vol_narrow - mean) / std_dev.max(0.1) * 2.;
        let brite_scale = brite_scale.clamp(0., 1.);
        if self.beat_mode && USE_BEAT_DETECTION && waveform != WaveformId(6) {
            base *= brite_scale;
        }

        // TODO implement "shifting"

        let base = base.clamp(0., 155.);

        // TODO set SoundEmpty

        // RGB
        // TODO sync color to sound
        let col = {
            let t = (g.frame as f32 + g.chaser_offset) * g.time_scale;
            let f = 7. * (t * 0.006 + 59.).sin() + 5. * (t * 0.0077 + 17.).cos();
            let dat = color_gen(s.gf, f, t, [0.55, 0.50], [10., 37., 32., 16., 87., 25.]);
            Rgba::from_f3([
                base * 1.07 * (1. + dat[0]) * (1. + dat[1]),
                base * 1.07 * (1. + dat[2]) * (1. + dat[3]),
                base * 1.07 * (1. + dat[4]) * (1. + dat[5]),
            ])
        };

        // TODO more samples via interpolation for high resolutions

        if g.sound_buffer.is_empty() {
            return;
        }

        match waveform {
            WaveformId(1) => {
                let (y_center, start, end) = if mode == ModeId(10) {
                    ((((img.rows() - 90) + img.cols() / 2) / 2) as f32, 10_u32, img.cols() - 10)
                } else {
                    (center.y as f32, 0, img.cols())
                };

                let mut zl = g.sound_buffer.lch(start as usize) + y_center;
                // println!("{zl} {y_center}");
                let mut prev_zl;
                for i in start..end {
                    prev_zl = zl;
                    zl = g.sound_buffer.lch(i as usize) + y_center;
                    zl = prev_zl * 0.90 + zl * 0.10;
                    let yl = zl as u32;
                    if s.y_roi.contains(yl) {
                        img[(yl, i)] = col;
                    }
                }
            }
            WaveformId(2) => {
                let div = 0.7;
                let h1 = center.y as f32 - s.fxh as f32 * 0.12;
                let h2 = center.y as f32 + s.fxh as f32 * 0.12;
                let mut zl = g.sound_buffer.lch(0) * div + h1;
                let mut zr = g.sound_buffer.rch(0) * div + h1;
                for j in 0..s.fxw {
                    let prev_zl = zl;
                    let prev_zr = zr;
                    zl = g.sound_buffer.lch(j as usize) * div + h1;
                    zr = g.sound_buffer.rch(j as usize) * div + h2;
                    zl = prev_zl * 0.9 + zl * 0.1;
                    zr = prev_zr * 0.9 + zr * 0.1;
                    let yl = zl as u32;
                    let yr = zr as u32;
                    if s.y_roi.contains(yl) {
                        img[(yl, j)] = col;
                    }
                    if s.y_roi.contains(yr) {
                        img[(yr, j)] = col;
                    }
                }
            }
            WaveformId(3) => {
                let mut zl = g.sound_buffer.lch(s.y_roi.min as usize) + center.x as f32;
                for i in s.y_roi.min..s.y_roi.max {
                    let prev_zl = zl;
                    zl = g.sound_buffer.lch(i as usize) + center.x as f32;
                    zl = prev_zl * 0.9 + zl * 0.1;
                    let xl = zl as i32;
                    if 0 <= xl && xl < s.fxw as i32 {
                        img[(i, xl as u32)] = col;
                    }
                }
            }
            WaveformId(4) => {
                let div = 0.9;
                let mut zl = g.sound_buffer.lch(s.y_roi.min as usize) * div;
                let mut zr = g.sound_buffer.rch(s.y_roi.min as usize) * div;
                for i in s.y_roi.min..s.y_roi.max {
                    let prev_zl = zl;
                    let prev_zr = zr;
                    zl = g.sound_buffer.lch(i as usize) * div;
                    zr = g.sound_buffer.rch(i as usize) * div;
                    zl = prev_zl * 0.9 + zl * 0.1;
                    zr = prev_zr * 0.9 + zr * 0.1;
                    let xl = zl as i32 + i as i32;
                    let xr = zr as i32 + i as i32 + (s.fxw - s.fxh) as i32;
                    if 0 <= xl && xl < s.fxw as i32 {
                        img[(i, xl as u32)] = col;
                    }
                    if 0 <= xr && xr < s.fxw as i32 {
                        img[(i, xr as u32)] = col;
                    }
                }
            }
            WaveformId(5) => {
                let div = 0.7;

                // Take first N samples from sound buffer and blend with the tail to make
                // it a repeating circle.
                let tmp = (0..WAVE_5_SIZE)
                    .map(|i| {
                        let val = g.sound_buffer.lch(i);
                        if i < WAVE_5_BLEND_RANGE {
                            let amt = i as f32 / WAVE_5_BLEND_RANGE as f32;
                            val * amt + (1. - amt) * g.sound_buffer.lch(i + WAVE_5_SIZE)
                        } else {
                            val
                        }
                    })
                    .collect::<Vec<_>>();

                let base_rad = s.fxw as f32 / 640. * 60.0;

                let mut rad = base_rad + tmp[0] * div;
                for i in 0..WAVE_5_SIZE {
                    rad = rad * 0.5 + 0.5 * (base_rad + tmp[i] * div);
                    if rad >= 5. {
                        let (si, ci) = (i as f32 * 0.02).sin_cos();
                        let px = (center.x as f32 + rad * ci) as u32;
                        let py = (center.y as f32 + rad * si) as u32;
                        if px < img.cols() && s.y_roi.contains(py) {
                            img[(py, px)] = col;
                        }
                    }
                }
            }
            WaveformId(6) => {
                let div = 1.2;
                let ang = (g.frame as f32 * 0.01).sin();
                let (sinang, cosang) = ang.sin_cos();
                let mut px2 = g.sound_buffer.lch(0);
                let mut py2 = g.sound_buffer.rch(0);
                for i in 0..WAVE_5_SIZE {
                    px2 = px2 * 0.5 + 0.5 * g.sound_buffer.lch(i) * div;
                    py2 = py2 * 0.5 + 0.5 * g.sound_buffer.rch(i) * div;
                    let px = (px2 * cosang + py2 * sinang + center.x as f32) as u32;
                    let py = (px2 * (-sinang) + py2 * cosang + center.y as f32) as u32;
                    if s.y_roi.contains(py) {
                        img[(py, px)] = col;
                    }
                }
            }
            WaveformId(7) | _ => {
                let (dx, dy) = (g.frame as f32 * 0.03).sin_cos();

                if dx.abs() > 0.001 {
                    if dx.abs() > dy.abs() {
                        let m = dy / dx;
                        let b = center.y as f32 - m * center.x as f32;
                        for x in 0..s.fxw {
                            let y = (m * x as f32 + b) as u32;
                            if s.y_roi.contains(y) {
                                let t = (g.sound_buffer.lch(x as usize) / 64.0).min(1.);
                                img[(y, x)] = col.scaled(t);
                            }
                        }
                    } else {
                        let m = dx / dy;
                        let b = center.x as f32 - m * center.y as f32;
                        for y in s.y_roi.min..s.y_roi.max {
                            let x = (m * y as f32 + b) as u32;
                            let t = (g.sound_buffer.lch(y as usize) / 64.0).min(1.);
                            img[(y, x)] = col.scaled(t);
                        }
                    }
                }
            }
        }
    }
}
