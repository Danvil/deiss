use crate::{
    effects::{
        globals::{Globals, MinstdRand},
        mode::{ModeKernel, TurnScaleMode},
        settings::{Mode, Settings},
    },
    utils::*,
};

#[derive(Clone, Copy, Default)]
pub struct FxPxl {
    pub weights: [u8; 4],
    pub index: u32,
}

pub type FlowMap = Image<FxPxl>;

#[derive(Clone)]
pub struct FlowMapSpec {
    pub settings: Settings,
    pub center: Vec2,
    pub damping: f32,
    pub mode: TurnScaleMode,
}

impl FlowMapSpec {
    pub fn generate(s: &Settings, g: &mut Globals) -> Self {
        let mode = s.mode_prefs.pick(&mut g.rand);

        let gxc = (s.fxw / 2 - 1) as f32 + g.rand.next_idx(60) as f32 - 30.;
        let gyc = (s.fxh / 2 - 1) as f32 + g.rand.next_idx(30) as f32 - 15.;

        let damping = g.suggested_dampening.clamp(0.50, 1.00)
            * if s.mode_settings[mode].motion_dampened {
                0.5
            } else {
                1.0
            };

        let damping_tmp = if 10. <= g.fps_at_last_mode_switch && g.fps_at_last_mode_switch <= 120. {
            damping * 30. / g.fps_at_last_mode_switch
        } else {
            damping
        };

        let waveform = pick_compatible_waveform(mode, &mut g.rand);

        g.big_beat_threshold = 1.10; // ??

        FlowMapSpec {
            settings: s.clone(),
            center: Vec2::new(gxc, gyc),
            damping: damping_tmp,
            mode: TurnScaleMode::mode_1(&mut g.rand),
            // mode: TurnScaleMode::from_scale_turn(0.98, 0.0),
        }
    }
}

fn pick_compatible_waveform(mode: Mode, rand: &mut MinstdRand) -> u32 {
    loop {
        let waveform = rand.next_idx(NUM_WAVES * 3 - 1) / 3 + 1;

        if !((mode == Mode(6) && waveform == 5)
            || (mode == Mode(12) && (waveform == 4 || waveform == 6))
            || (mode == Mode(14) && (waveform == 3 || waveform == 4))
            || ((mode == Mode(8) || mode == Mode(23) || mode == Mode(24)) && waveform == 6))
        {
            return waveform;
        }
    }
}

pub struct FlowMapGen {
    spec: FlowMapSpec,
    y_map_pos: u32,
}

const NUM_WAVES: u32 = 6;

impl FlowMapGen {
    pub fn new(spec: FlowMapSpec) -> Self {
        FlowMapGen { spec, y_map_pos: 0 }
    }

    pub fn run(&mut self) -> FlowMap {
        let s = &self.spec.settings;
        self.y_map_pos = s.fx_ycut * s.fxw;
        bake(s, self.spec.center, self.spec.damping, &self.spec.mode)
    }
}

pub fn bake<M: ModeKernel>(s: &Settings, center: Vec2, damping: f32, mode: &M) -> Image<FxPxl> {
    let s_fxw_minus_once = (s.fxw - 1) as f32;
    // let half_fxw = s.fxw as f32 * 0.5;

    let weightsum_res_adjusted = match (s.fxw, s.fxh) {
        (640, 480) => 252.,
        _ => 255.,
    };

    Image::from_fn((s.fxh, s.fxw).into(), |(i, j)| {
        let pi = Vec2 {
            x: j as f32,
            y: i as f32,
        };

        let p2 = mode.transform(pi - center) + center;

        let p4 = pi * (1.0 - damping) + p2 * damping;

        let p = {
            let mut p = p4;
            while p.x < 0. {
                p.x += s_fxw_minus_once;
            }
            while p.x > s_fxw_minus_once {
                p.x -= s_fxw_minus_once;
            }
            p
        };

        let ix = p.x as u32;
        let iy = p.y as u32;

        // exclude bottom and top two rows
        let index = iy.clamp(2, s.fxh - 3) * s.fxw + ix;

        let weightsum_this_pixel = weightsum_res_adjusted;

        let dx = p.x - ix as f32;
        let dy = p.y - iy as f32;

        let weights = [
            ((1. - dx) * (1. - dy) * weightsum_this_pixel) as u8,
            (dx * (1. - dy) * weightsum_this_pixel) as u8,
            ((1. - dx) * dy * weightsum_this_pixel) as u8,
            (dx * dy * weightsum_this_pixel) as u8,
        ];

        FxPxl { weights, index }
    })
}

pub fn process_map(s: &Settings, fx: &[FxPxl], src: &RgbaImage, dst: &mut RgbaImage) {
    let src = src.as_slice();
    let dst = dst.as_slice_mut();

    let fx_ycut_num_lines = s.fxw * (s.fxh - s.fx_ycut * 2);

    let idx0 = (s.fxw * s.fx_ycut) as usize;
    let idx1 = idx0 + fx_ycut_num_lines as usize;

    for idx in idx0..idx1 {
        let FxPxl { weights, index } = fx[idx];
        dst[idx] = bilin_w(src, index as usize, s.fxw as usize, weights);
    }
}

fn bilin_w(src: &[Rgba], i: usize, cols: usize, weights: [u8; 4]) -> Rgba {
    let p1 = src[i];
    let p2 = src[i + 1];
    let p3 = src[i + cols];
    let p4 = src[i + cols + 1];

    Rgba([
        dot_u8([p1[0], p2[0], p3[0], p4[0]], weights),
        dot_u8([p1[1], p2[1], p3[1], p4[1]], weights),
        dot_u8([p1[2], p2[2], p3[2], p4[2]], weights),
        255,
    ])
}

fn dot_u8(a: [u8; 4], b: [u8; 4]) -> u8 {
    (((a[0] as u32) * (b[0] as u32)
        + (a[1] as u32) * (b[1] as u32)
        + (a[2] as u32) * (b[2] as u32)
        + (a[3] as u32) * (b[3] as u32))
        >> 8) as u8
}
