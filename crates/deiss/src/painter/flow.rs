use crate::{painter::*, utils::*};

#[derive(Clone, Copy, Default)]
pub struct FxPxl {
    pub weights: [u8; 4],
    pub index: u32,
}

pub type FlowMap = Image<FxPxl>;

#[derive(Clone)]
pub struct FlowMapSpec {
    pub settings: Settings,
    pub effects: Effects,
    pub mode: ModeId,
    pub waveform: u32,
    pub center: Vec2i32,
    pub damping: f32,
    pub tf: AnyTransform,
}

impl FlowMapSpec {
    pub fn generate(s: &Settings, fx: &ModeBlueprintLibrary, g: &mut Globals) -> Self {
        let mode = s.mode_prefs.pick(&mut g.rand);

        let [effects_min, effects_max] = fx[mode].effect_count;
        let effects =
            fx[mode].effect_freq.sample((effects_min as usize, effects_max as usize), &mut g.rand);

        let gxc = ((s.fxw / 2 - 1) as f32 + g.rand.next_idx(60) as f32 - 30.) as i32;
        let gyc = ((s.fxh / 2 - 1) as f32 + g.rand.next_idx(30) as f32 - 15.) as i32;

        let damping = g.suggested_dampening.clamp(0.50, 1.00)
            * if fx[mode].motion_dampened { 0.5 } else { 1.0 }
            * g.time_scale;

        let waveform = pick_compatible_waveform(mode, &mut g.rand);

        g.big_beat_threshold = 1.10; // ??

        let tf = fx[mode].generate_transform(&mut g.rand);

        FlowMapSpec {
            settings: s.clone(),
            effects,
            mode,
            waveform,
            center: Vec2i32::new(gxc, gyc),
            damping,
            tf,
        }
    }
}

fn pick_compatible_waveform(mode: ModeId, rand: &mut Minstd) -> u32 {
    loop {
        let waveform = rand.next_idx(NUM_WAVES * 3 - 1) / 3 + 1;

        if !((mode == ModeId(6) && waveform == 5)
            || (mode == ModeId(12) && (waveform == 4 || waveform == 6))
            || (mode == ModeId(14) && (waveform == 3 || waveform == 4))
            || ((mode == ModeId(8) || mode == ModeId(23) || mode == ModeId(24)) && waveform == 6))
        {
            return waveform;
        }
    }
}

pub struct FlowMapGen {
    spec: FlowMapSpec,
}

const NUM_WAVES: u32 = 6;

impl FlowMapGen {
    pub fn new(spec: FlowMapSpec) -> Self {
        FlowMapGen { spec }
    }

    pub fn run(&mut self) -> FlowMap {
        bake(&self.spec.settings, self.spec.center.cast(), self.spec.damping, &self.spec.tf)
    }
}

pub fn bake<M: PixelTransform>(s: &Settings, center: Vec2, damping: f32, mode: &M) -> Image<FxPxl> {
    let s_fxw_minus_once = (s.fxw - 1) as f32;
    // let half_fxw = s.fxw as f32 * 0.5;

    let weightsum_res_adjusted = match (s.fxw, s.fxh) {
        (640, 480) => 252.,
        _ => 255.,
    };

    Image::from_fn((s.fxh, s.fxw).into(), |(i, j)| {
        let pi = Vec2 { x: j as f32, y: i as f32 };

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

    let idx0 = (s.fxw * s.y_roi.min) as usize;
    let idx1 = (s.fxw * s.y_roi.max) as usize;

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
