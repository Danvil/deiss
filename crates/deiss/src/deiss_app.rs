use crate::{
    audio::Playback,
    config::{Config, SharedConfig},
    effects::Painter,
    renderer::{Gpu, Renderer, Surface},
};
use eyre::Result;
use std::sync::{Arc, Mutex};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::{Window, WindowAttributes, WindowId},
};

pub struct DeissApp {
    config: SharedConfig,
    state: Option<State>,
}

impl DeissApp {
    pub fn new(config: Config) -> Self {
        DeissApp {
            config: SharedConfig::new(config),
            state: None,
        }
    }

    fn resumed_impl(&mut self, event_loop: &ActiveEventLoop) -> Result<()> {
        let window = Arc::new(
            event_loop.create_window(
                WindowAttributes::default()
                    .with_title("DEISS")
                    .with_inner_size(winit::dpi::LogicalSize::new(640.0, 480.0)),
            )?,
        );

        self.state = Some(pollster::block_on(State::new(window, self.config.clone()))?);

        Ok(())
    }
}

impl ApplicationHandler for DeissApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if let Err(err) = self.resumed_impl(event_loop) {
            log::error!("resumed failed: {err:?}");
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
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
    config: SharedConfig,
    playback: Playback,
    painter: Arc<Mutex<Painter>>,
}

impl State {
    pub async fn new(window: Arc<Window>, config: SharedConfig) -> Result<Self> {
        let gpu = Arc::new(Gpu::new().await?);
        let surface = Surface::new(gpu.clone(), window.clone())?;
        let renderer = Renderer::new(&gpu);

        let painter = Arc::new(Mutex::new(Painter::new(surface.size_as_shape())));
        // let listener = ConsoleAudioListener::new();

        let mut playback = Playback::new()?;
        playback.set_listener(painter.clone());

        let filename = config.lock().filename;
        log::info!("Now playing: {filename}");
        playback.play(&filename)?;

        Ok(Self {
            gpu,
            window,
            surface,
            renderer,
            config,
            playback,
            painter,
        })
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn resize(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        self.surface.resize(size);
    }

    pub fn render(&mut self) {
        if self.playback.is_empty() {
            self.playback.play(&self.config.lock().filename).unwrap();
        }

        let (surface_texture, texture_view) = self
            .surface
            .texture()
            .expect("failed to acquire next swapchain texture");

        self.painter.lock().unwrap().on_render();

        self.renderer.render(
            &self.gpu,
            texture_view,
            self.painter.lock().unwrap().image(),
        );

        self.window.pre_present_notify();
        surface_texture.present();
    }
}
