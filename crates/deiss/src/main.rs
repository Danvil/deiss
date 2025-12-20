pub mod audio;
mod deiss_app;
pub mod effects;
pub mod renderer;

use crate::deiss_app::DeissApp;
use eyre::Result;
use winit::event_loop::{ControlFlow, EventLoop};

fn main() -> Result<()> {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = DeissApp::default();
    event_loop.run_app(&mut app)?;
    Ok(())
}
