use crate::{
    audio::{ConsoleAudioListener, Playback},
    renderer::{Gpu, Renderer, Surface},
};
use eyre::Result;
use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::{Window, WindowAttributes, WindowId},
};

#[derive(Default)]
pub struct DeissApp {
    state: Option<State>,
}

impl DeissApp {
    fn resumed_impl(&mut self, event_loop: &ActiveEventLoop) -> Result<()> {
        let window =
            Arc::new(event_loop.create_window(WindowAttributes::default().with_title("DEISS"))?);

        self.state = Some(pollster::block_on(State::new(window))?);

        Ok(())
    }
}

impl ApplicationHandler for DeissApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if let Err(err) = self.resumed_impl(event_loop) {
            log::error!("resumed failed: {err:?}");
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        let Some(state) = self.state.as_mut() else {
            log::error!("app no resumed");
            return;
        };

        match event {
            WindowEvent::CloseRequested => {
                log::info!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                state.render();
                state.window().request_redraw();
            }
            WindowEvent::Resized(size) => {
                state.resize(size);
            }
            _ => (),
        }
    }
}

struct State {
    gpu: Arc<Gpu>,
    window: Arc<Window>,
    surface: Surface,
    renderer: Renderer,

    playback: Playback,
}

impl State {
    pub async fn new(window: Arc<Window>) -> Result<Self> {
        let gpu = Arc::new(Gpu::new().await?);
        let surface = Surface::new(gpu.clone(), window.clone())?;
        let renderer = Renderer::new();

        let mut playback = Playback::new()?;
        let listener = ConsoleAudioListener::new();
        playback.set_listener(listener);
        playback.play("assets/785736__alien_i_trust__stargazer-by-alien-i-trust-125_bpm.wav")?;

        Ok(Self {
            gpu,
            window,
            surface,
            renderer,
            playback,
        })
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn resize(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        self.surface.resize(size);
    }

    pub fn render(&mut self) {
        let (surface_texture, texture_view) = self
            .surface
            .texture()
            .expect("failed to acquire next swapchain texture");

        self.renderer.render(&self.gpu, texture_view);

        self.window.pre_present_notify();
        surface_texture.present();
    }
}
