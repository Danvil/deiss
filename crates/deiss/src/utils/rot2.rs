use crate::utils::Vec2f;

/// 2D rotation
#[derive(Debug, Clone)]
pub struct Rot2f {
    sin: f32,
    cos: f32,
}

impl Rot2f {
    pub fn from_angle(angle: f32) -> Self {
        let (sin, cos) = angle.sin_cos();
        Self { sin, cos }
    }

    pub fn transform(&self, p: Vec2f) -> Vec2f {
        let x = p.x * self.cos - p.y * self.sin;
        let y = p.x * self.sin + p.y * self.cos;
        Vec2f::new(x, y)
    }
}
