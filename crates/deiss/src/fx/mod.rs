mod dots;
mod grid;
mod one_dotty_chaser;
mod shade_bobs;
mod snack_bar;
mod solar_particles;
mod two_chasers;

pub use dots::*;
pub use grid::*;
pub use one_dotty_chaser::*;
pub use shade_bobs::*;
pub use snack_bar::*;
pub use solar_particles::*;
pub use two_chasers::*;

use crate::utils::*;

pub trait Effect {
    fn render(&self, img: &mut RgbaImage, rand: &mut Minstd);
}
