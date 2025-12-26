use crate::{
    audio::{AudioListener, AudioSamples},
    fx::{self, Effect},
    painter::*,
    utils::*,
};
use std::{f32, mem};

pub struct Painter {
    pub(crate) settings: Settings,
    pub(crate) library: ModeBlueprintLibrary,
    pub(crate) globals: Globals,
    img: RgbaImage,
    next: RgbaImage,
    fx_hub: WarpMapHub,
    fx: Option<(WarpSpec, WarpMap)>,
    needs_init: bool,
    wave: Wave,
}

impl Painter {
    pub fn new(shape: Shape2) -> Self {
        const YCUT: u32 = 90;

        let mut globals = Globals::default();
        globals.chaser_offset = globals.rand.next_idx(40_000) as f32;
        globals.fps_at_last_mode_switch = 30.;
        globals.time_scale = 1.;

        let (fxh, fxw) = shape.into();

        let settings = Settings {
            volscale: 0.2,
            enable_map_dampening: false,
            fxw,
            fxh,
            y_roi: YRoi { min: YCUT, max: fxh - YCUT },
            gf: generate_gf(&mut globals.rand),
            mode_prefs: ModePrefs::new(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]),
            waveform_prefs: WaveformPrefs::default(),
        };

        let library = ModeBlueprintLibrary::new(&mut globals);

        Self {
            img: RgbaImage::black(shape),
            next: RgbaImage::black(shape),
            fx_hub: WarpMapHub::new(),
            fx: None,
            needs_init: true,
            wave: Wave::new(&globals),
            settings,
            library,
            globals,
        }
    }

    pub fn image(&self) -> &RgbaImage {
        &self.img
    }

    pub fn on_render(&mut self) {
        self.globals.frame += 1;
        self.globals.floatframe += 1.6 * (47.0 / self.globals.fps_at_last_mode_switch).min(1.);

        self.globals.fps.step();

        self.fx_hub.step(&self.settings, &self.library, &mut self.globals).ok();
        if let Some(fx) = self.fx_hub.fetch() {
            log::info!("New mode: {:?} W{:?} {:?}", fx.0.mode, fx.0.waveform, fx.0.effects);
            self.fx = Some(fx);
            self.needs_init = true;
        }

        let Some((spec, fx)) = self.fx.as_ref() else {
            return;
        };

        if self.needs_init {
            self.needs_init = false;

            if spec.mode == ModeId(1) && self.globals.rand.next_bool() {
                fx::SolarParticles { center: spec.center, count: 500 }
                    .render(&mut self.img, &mut self.globals.rand);
            }
        }

        if spec.effects[EffectKind::Shade] {
            // println!(" >> ShadeBobs");
            fx::ShadeBobs::new(spec.center, self.globals.floatframe, &mut self.globals.rand)
                .render(&mut self.img, &mut self.globals.rand);
        }

        if spec.effects[EffectKind::Chasers] {
            fx::TwoChasers::new(spec.center, 2, &self.settings, &self.globals)
                .render(&mut self.img, &mut self.globals.rand);
        }

        if spec.effects[EffectKind::Bar] {
            fx::SnackBar::new(spec.center, &self.settings, &self.globals)
                .render(&mut self.img, &mut self.globals.rand);
        }

        if spec.effects[EffectKind::Dots] {
            fx::OneDottyChaser::new(spec.center, &self.settings, &self.globals)
                .render(&mut self.img, &mut self.globals.rand);
        }

        if spec.effects[EffectKind::Nuclide] {
            fx::Nuclide::new_nuclide(spec.center, &self.settings, &mut self.globals)
                .render(&mut self.img, &mut self.globals.rand);
        }

        if spec.effects[EffectKind::Grid] {
            fx::Grid::new(&self.settings, &self.globals)
                .render(&mut self.img, &mut self.globals.rand);
        }

        if spec.effects[EffectKind::Solar] {
            fx::SolarParticles::new(spec.center, self.library[spec.mode].solar_max, &self.globals)
                .render(&mut self.img, &mut self.globals.rand);
        }

        {
            let center_dwindle = self.library[spec.mode].center_dwindle;
            if center_dwindle < 0.999 {
                let center_mode = spec.mode != ModeId(12);
                fx::DiminishCenter::new(spec.center, center_mode, center_dwindle, &self.settings)
                    .render(&mut self.img, &mut self.globals.rand);
            }
        }

        process_map(&self.settings, fx.as_slice(), &self.img, &mut self.next);
        mem::swap(&mut self.img, &mut self.next);

        // render dots on beats
        fx::Nuclide::new_beat_dots(spec.center, &self.settings, &mut self.globals)
            .render(&mut self.img, &mut self.globals.rand);

        self.wave.render(
            &mut self.img,
            spec.center,
            spec.mode,
            spec.waveform,
            &self.settings,
            &self.globals,
        );
    }
}

impl AudioListener for Painter {
    fn buffer_size(&self) -> usize {
        ((self.settings.fxw * 2) as usize).max((WAVE_5_SIZE + WAVE_5_BLEND_RANGE) * 2 + 20)
    }

    fn on_samples(&mut self, wave: &AudioSamples) {
        process_wave_data(wave, &self.settings, &mut self.globals);
    }
}

fn process_wave_data(wave: &AudioSamples, s: &Settings, g: &mut Globals) {
    let mut buf = wave.iter().map(|&v| v as f32).collect::<Vec<_>>();

    level_trigger(&mut buf, s, g);

    let vol = volume(&buf);
    g.vol.push(vol);

    let peaks = 0.; // TODO seems to be always 0 in the source

    g.avg_vol_narrow =
        blend(adjust_rate_to_fps(0.30, 30., g.fps_at_last_mode_switch), (g.avg_vol_narrow, vol));
    g.avg_vol = blend(adjust_rate_to_fps(0.85, 30., g.fps_at_last_mode_switch), (g.avg_vol, vol));
    g.avg_vol_wide =
        blend(adjust_rate_to_fps(0.96, 30., g.fps_at_last_mode_switch), (g.avg_vol_wide, vol));
    g.avg_vol_peaks =
        blend(adjust_rate_to_fps(0.90, 30., g.fps_at_last_mode_switch), (g.avg_vol_peaks, peaks));
    g.volume_sum += g.avg_vol as u64;

    g.vol_narrow.push(g.avg_vol_narrow);

    low_pass_filter_inplace(&mut buf);

    let fdiv = 1.0 / (64.0 * (640.0 / s.fxw as f32));
    let billy = s.volscale * fdiv; // * 4.;
    scale_inplace(&mut buf, billy);

    // centroid for L, R channels
    let centroid = channel_centroid(&buf);

    // translate wave so center it at zero
    sub_inplace(&mut buf, centroid);

    // compute power using fourier
    let mut net_power_change = g.fourier.fourier(&buf);
    net_power_change /= (g.volume_sum as f32) / (g.frame + 1) as f32;
    net_power_change *= 0.01;

    // dampening based on spectral variance
    if s.enable_map_dampening {
        g.suggested_dampening =
            if g.frame < 50 { 1. } else { g.suggested_dampening * 0.98 + net_power_change * 0.02 };
    } else {
        g.suggested_dampening = 1.0;
    }

    g.sound_buffer = SoundBuffer::from_vec(buf);
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
    let z = 1. / (buf.len() / 8) as f32;
    [centroid[0] * z, centroid[1] * z]
}

fn sub_inplace(buf: &mut [f32], centroid: [f32; 2]) {
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
