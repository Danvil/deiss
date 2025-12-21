use std::{f32, mem};

use crate::{
    audio::{AudioListener, AudioSamples},
    effects::{
        flow::{FlowMapSpec, process_map},
        flow_hub::FlowMapHub,
        globals::Globals,
        settings::{ModePrefs, ModeSettings, ModeSettingsVec, Settings},
    },
    utils::*,
};

pub struct Painter {
    settings: Settings,
    globals: Globals,
    img: RgbaImage,
    next: RgbaImage,
    fx: FlowMapHub,
}

fn new_motion_settings() -> ModeSettingsVec {
    let mode_motion_dampened = [
        true, true, true, false, true, true, false, true, true,
        true, // 0 is bunk; 1-9 here (3,6 not dampened)
        true, true, true, true, true, true, true, // modes 10-16 are all dampened
        false, false, false, // 17 - 19 are normal
        false, false, false, false, false, // 20 - 24 are normal
        false, false, false, false, false, // 25 - 29 are normal
        false, false, false, false, false, // 30 - 34 are normal
        false, false, false, false, false, // 35 - 39 are normal
    ];

    let rotation_dither = [
        false, true, false, false, false, false, false, false, false,
        true, // 0-9 (0 is unused)
        false, true, false, false, false, // 10 - 14
        false, false, false, false, false, // 15 - 19
        false, false, false, false, false, // 20 - 24
        false, false, false, false, false, // 25 - 29
        false, false, false, false, false, // 30 - 34
        false, false, false, false, false, // 35 - 39
    ];

    let custom_motion_vectors = [
        // 6, 10, 12 are true
        false, false, false, false, false, false, true, false, false,
        false, // 0-9 (0 is unused)
        true, false, true, false, false, // 10 - 14
        false, false, false, false, false, // 15 - 19
        false, false, false, false, false, // 20 - 24
        false, false, false, false, false, // 25 - 29
        false, false, false, false, false, // 30 - 34
        false, false, false, false, false, // 35 - 39
    ];

    ModeSettingsVec::from_vec(
        (0..40)
            .map(|i| ModeSettings {
                motion_dampened: mode_motion_dampened[i],
                rotation_dither: rotation_dither[i],
                custom_motion_vector: custom_motion_vectors[i],
            })
            .collect(),
    )
}

impl Painter {
    pub fn new(shape: Shape2) -> Self {
        let mut globals = Globals::default();

        let (fxh, fxw) = shape.into();

        let settings = Settings {
            volscale: 0.2,
            enable_map_dampening: false,
            fxw,
            fxh,
            fx_ycut: 90,
            disp_bits: 32,
            chaser_offset: globals.rand.next_idx(40_000),
            gf: core::array::from_fn(|_| {
                ((globals.rand.next_idx(1000) as f32) * 0.001) * 0.01 + 0.02
            }),
            mode_prefs: ModePrefs::new(),
            mode_settings: new_motion_settings(),
        };

        Self {
            img: RgbaImage::black(shape),
            next: RgbaImage::black(shape),
            fx: FlowMapHub::new(),
            settings,
            globals,
        }
    }

    pub fn image(&self) -> &RgbaImage {
        &self.img
    }

    pub fn on_render(&mut self) {
        self.globals.fps.step();

        self.fx.step(&self.settings, &mut self.globals).ok();
        let Some((spec, fx)) = self.fx.current() else {
            return;
        };

        process_map(&self.settings, fx.as_slice(), &self.img, &mut self.next);
        mem::swap(&mut self.img, &mut self.next);

        render_dots(&mut self.img, &spec, &self.settings, &mut self.globals);
    }
}

impl AudioListener for Painter {
    fn buffer_size(&self) -> usize {
        ((self.settings.fxw * 2) as usize).max((314 + 50) * 2 + 20)
    }

    fn on_samples(&mut self, wave: &AudioSamples) {
        process_wave_data(wave, &self.settings, &mut self.globals);
    }
}

fn process_wave_data(wave: &AudioSamples, s: &Settings, g: &mut Globals) {
    g.frame += 1;

    let mut buf = wave.iter().map(|&v| v as f32).collect::<Vec<_>>();

    level_trigger(&mut buf, s, g);

    let vol = volume(&buf);
    g.vol.push(vol);

    let peaks = 0.; // TODO seems to be always 0 in the source

    g.avg_vol_narrow = blend(
        adjust_rate_to_fps(0.30, 30., g.fps_at_last_mode_switch),
        (g.avg_vol_narrow, vol),
    );
    g.avg_vol = blend(
        adjust_rate_to_fps(0.85, 30., g.fps_at_last_mode_switch),
        (g.avg_vol, vol),
    );
    g.avg_vol_wide = blend(
        adjust_rate_to_fps(0.96, 30., g.fps_at_last_mode_switch),
        (g.avg_vol_wide, vol),
    );
    g.avg_vol_peaks = blend(
        adjust_rate_to_fps(0.90, 30., g.fps_at_last_mode_switch),
        (g.avg_vol_peaks, peaks),
    );
    g.volume_sum += g.avg_vol as u64;

    low_pass_filter_inplace(&mut buf);

    let fdiv = 1.0 / (64.0 * (640.0 / s.fxw as f32));
    let billy = s.volscale * fdiv * 4.;
    scale_inplace(&mut buf, billy);

    // centroid for L, R channels
    let mut centroid = channel_centroid(&buf);
    centroid[0] /= (s.fxw as f32) * 0.125;
    centroid[1] /= (s.fxw as f32) * 0.125;

    // translate wave so center is at zero
    add_inplace(&mut buf, centroid);

    // compute power using fourier
    let mut net_power_change = g.fourier.fourier(&buf);
    net_power_change /= (g.volume_sum as f32) / (g.frame + 1) as f32;
    net_power_change *= 0.01;

    // dampening based on spectral variance
    if s.enable_map_dampening {
        g.suggested_dampening = if g.frame < 50 {
            1.
        } else {
            g.suggested_dampening * 0.98 + net_power_change * 0.02
        };
    } else {
        g.suggested_dampening = 1.0;
    }
}

fn adjust_rate_to_fps(per_frame_decay_rate_at_fps1: f32, fps1: f32, actual_fps: f32) -> f32 {
    // returns the equivalent per-frame decay rate at actual_fps
    // basically, do all your testing at fps1 and get a good decay rate;
    // then, in the real application, adjust that rate by the actual fps each time you use it.

    let per_second_decay_rate_at_fps1 = per_frame_decay_rate_at_fps1.powf(fps1);
    let per_frame_decay_rate_at_fps2 = per_second_decay_rate_at_fps1.powf(1.0 / actual_fps);

    per_frame_decay_rate_at_fps2
}

fn blend(p: f32, (a, b): (f32, f32)) -> f32 {
    p * a + (1.0 - p) * b
}

fn level_trigger(buf: &mut [f32], s: &Settings, g: &mut Globals) {
    let fxw_div_2 = s.fxw as usize / 2;
    let mut trigger = None;
    for i in (8..fxw_div_2).step_by(2) {
        let v_old = buf[i + fxw_div_2 - 8];
        let v = buf[i + fxw_div_2];

        let height_match = (v - g.last_frame_v).abs() <= 256.;
        let slope_match = g.last_frame_slope * (v - v_old) >= 0.;

        if height_match && slope_match {
            g.last_frame_v = v;
            g.last_frame_slope = v - v_old;
            trigger = Some(i);
            break;
        }
    }

    if let Some(trigger) = trigger {
        for i in trigger..buf.len() {
            buf[i - trigger] = buf[i];
        }
    } else {
        let v_old = buf[fxw_div_2];
        let v = buf[fxw_div_2 + 8];
        g.last_frame_v = v;
        g.last_frame_slope = v - v_old;
    }
}

fn volume(buf: &[f32]) -> f32 {
    let mut low = buf[0];
    let mut high = low;
    for &v in buf.iter().step_by(4) {
        low = low.min(v);
        high = high.max(v);
    }
    (high - low) / 256.0
}

// low pass filter for bass hits and scale
fn low_pass_filter_inplace(buf: &mut [f32]) {
    for i in 0..buf.len() - 2 {
        buf[i] = 0.8 * buf[i] + 0.2 * buf[i + 2];
    }
}

fn scale_inplace(buf: &mut [f32], scale: f32) {
    for v in buf {
        *v *= scale;
    }
}

fn channel_centroid(buf: &[f32]) -> [f32; 2] {
    let mut centroid = [0.; 2];
    for i in (0..buf.len()).step_by(8) {
        centroid[0] += buf[i];
        centroid[1] += buf[i + 1];
    }
    centroid
}

fn add_inplace(buf: &mut [f32], centroid: [f32; 2]) {
    for i in (0..buf.len()).step_by(2) {
        buf[i] -= centroid[0];
        buf[i + 1] -= centroid[1];
    }
}

#[derive(Default, Debug)]
pub struct RunningFourier {
    power: [f32; Self::FOURIER_DETAIL],
    power_smoothed: [f32; Self::FOURIER_DETAIL],
}

impl RunningFourier {
    const FOURIER_DETAIL: usize = 24;
}

impl RunningFourier {
    fn fourier(&mut self, buf: &[f32]) -> f32 {
        let mut net_power_change = 0.;

        for n in 1..Self::FOURIER_DETAIL {
            let w = f32::consts::TAU
                * (20. * 2.0_f32.powf((n as f32) / (Self::FOURIER_DETAIL as f32) * 10.) / 44_100.);

            let mut a = 0.;
            let mut b = 0.;
            for i in 0..256 {
                let (sin_th, cos_th) = (w * i as f32).sin_cos();
                let v = buf[2 * i];
                a += v * sin_th;
                b += v * cos_th;
            }

            let old_power = self.power[n];
            self.power[n] = a.hypot(b);
            net_power_change += (old_power - self.power[n]).abs();

            self.power_smoothed[n] = 0.94 * self.power_smoothed[n] + 0.06 * self.power[n];
        }

        net_power_change
    }
}

fn render_dots(img: &mut RgbaImage, spec: &FlowMapSpec, s: &Settings, g: &mut Globals) {
    if g.vol.current() <= g.avg_vol_narrow * 1.1 {
        return;
    }

    let nodes = 3 + g.rand.next_idx(5);

    let phase = g.rand.next_idx(1000) as f32;

    let mut r =
        (if s.fxw == 320 { 2. } else { 3. } + 40. * (g.vol.current() / g.avg_vol_narrow - 1.1));
    r = r.max(1.);
    let r = if s.fxw == 320 { r.min(7.) } else { r.min(10.) };

    let rad = if s.fxw == 320 {
        (22 + g.rand.next_idx(6)) as f32
    } else {
        (34 + g.rand.next_idx(8)) as f32 * ((s.fxw as f32) / 1024.).max(1.)
    };

    let col = if s.disp_bits > 8 {
        let i = (g.frame + s.chaser_offset as u64) as f32;
        let f = 7. * (i * 0.007 + 29.).sin() + 5. * (i * 0.0057 + 27.).cos();
        [
            0.58 + 0.21 * (i * s.gf[0] + 20. - f).sin() + 0.21 * (i * s.gf[3] + 17. + f).cos(),
            0.58 + 0.21 * (i * s.gf[1] + 42. + f).sin() + 0.21 * (i * s.gf[4] + 26. - f).cos(),
            0.58 + 0.21 * (i * s.gf[2] + 57. - f).sin() + 0.21 * (i * s.gf[5] + 35. + f).cos(),
        ]
    } else {
        [1., 1., 1.]
    };
    // println!("{cr} {cg} {cb}");

    for n in 0..nodes {
        let (th_cos, th_sin) = ((n as f32) / (nodes as f32) * f32::consts::TAU + phase).sin_cos();
        let cx = (spec.center.x + rad * th_cos) as i32;
        let cy = (spec.center.y + rad * th_sin) as i32;
        // println!("{cx} {cy}");

        for y in -10..=10 {
            for x in -10..=10 {
                let scl = (r - ((x * x + y * y) as f32).sqrt()) * 25.;
                if scl > 0. {
                    if s.disp_bits == 8 {
                        todo!()
                    } else {
                        let pxl_idx = ((cy + y) as u32, (cx + x) as u32);
                        rgba_scl_add_inpl(&mut img[pxl_idx], scl, col);
                    }
                }
            }
        }
    }
}

fn rgba_scl_add_inpl(v: &mut Rgba, scl: f32, d: [f32; 3]) {
    for i in 0..3 {
        v[i] = v[i].saturating_add((scl * d[i]) as u8);
    }
}
