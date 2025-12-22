use core::ops;

pub type Vec2 = Vec2K<f32>;

pub type Vec2i32 = Vec2K<i32>;

#[derive(Debug, Clone, Copy)]
pub struct Vec2K<K> {
    pub x: K,
    pub y: K,
}

impl<K> Vec2K<K> {
    pub fn new(x: K, y: K) -> Self {
        Vec2K { x, y }
    }

    pub fn cast<S>(&self) -> Vec2K<S>
    where
        K: Copy + CoeffCast<S>,
    {
        Vec2K { x: self.x.cast(), y: self.y.cast() }
    }
}

pub trait CoeffCast<S> {
    fn cast(self) -> S;
}

impl CoeffCast<i32> for f32 {
    fn cast(self) -> i32 {
        self as i32
    }
}

impl CoeffCast<f32> for i32 {
    fn cast(self) -> f32 {
        self as f32
    }
}

impl<K> ops::Mul<K> for Vec2K<K>
where
    K: Copy + ops::Mul<K, Output = K>,
{
    type Output = Vec2K<K>;

    fn mul(self, scalar: K) -> Vec2K<K> {
        Vec2K { x: self.x * scalar, y: self.y * scalar }
    }
}

impl<K> ops::Sub for Vec2K<K>
where
    K: Copy + ops::Sub<K, Output = K>,
{
    type Output = Vec2K<K>;

    fn sub(self, other: Vec2K<K>) -> Vec2K<K> {
        Vec2K { x: self.x - other.x, y: self.y - other.y }
    }
}

impl<K> ops::Add for Vec2K<K>
where
    K: Copy + ops::Add<K, Output = K>,
{
    type Output = Vec2K<K>;

    fn add(self, other: Vec2K<K>) -> Vec2K<K> {
        Vec2K { x: self.x + other.x, y: self.y + other.y }
    }
}
