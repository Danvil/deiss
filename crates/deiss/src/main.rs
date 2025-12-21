pub mod audio;
pub mod config;
mod deiss_app;
pub mod effects;
pub mod renderer;
pub mod utils;

use crate::{config::Config, deiss_app::DeissApp};
use eyre::{Result, bail};
use winit::event_loop::{ControlFlow, EventLoop};

fn main() -> Result<()> {
    println!("{:?}", std::env::args());

    let Some(filename) = std::env::args().skip(1).next() else {
        bail!("Usage: deiss music.wav/.mp3");
    };

    env_logger::init();

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let config = Config { filename };

    let mut app = DeissApp::new(config);
    event_loop.run_app(&mut app)?;
    Ok(())
}
