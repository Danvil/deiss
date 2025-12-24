use crate::{
    painter::{
        globals::Globals,
        mode_blueprint_library::ModeBlueprintLibrary,
        settings::Settings,
        warp::{WarpGen, WarpMap, WarpPixel, WarpSpec},
    },
    utils::*,
};
use eyre::{Result, bail};
use std::{
    sync::mpsc,
    thread::JoinHandle,
    time::{Duration, Instant},
};

pub struct WarpMapHub {
    current: Option<(WarpSpec, WarpMap)>,
    next_spec: Option<WarpSpec>,
    worker: WarpMapWorker,
    next_switch_time: Instant,
}

impl WarpMapHub {
    pub fn new() -> Self {
        Self {
            current: None,
            next_spec: None,
            worker: WarpMapWorker::new(),
            next_switch_time: Instant::now(),
        }
    }

    pub fn step(&mut self, s: &Settings, fx: &ModeBlueprintLibrary, g: &mut Globals) -> Result<()> {
        if self.worker.is_idle() {
            if self.next_switch_time < Instant::now() {
                let spec = WarpSpec::generate(s, fx, g);
                self.next_spec = Some(spec.clone());
                self.worker.start(spec)?;
                self.next_switch_time = Instant::now() + Duration::from_secs(3);
                g.fps_at_last_mode_switch = g.fps.reset();
                g.time_scale = 30. / g.fps_at_last_mode_switch.clamp(10., 120.);
            }
        } else {
            if let Some(map) = self.worker.retreive()? {
                self.current = Some((self.next_spec.take().unwrap(), map));
            }
        }
        Ok(())
    }

    pub fn fetch(&mut self) -> Option<(WarpSpec, WarpMap)> {
        self.current.take()
    }
}

impl Drop for WarpMapHub {
    fn drop(&mut self) {
        self.worker.terminate();
    }
}

struct WarpMapWorker {
    tx_worker_request: mpsc::Sender<WarpMapWorkerRequest>,
    rx_worker_reply: mpsc::Receiver<WarpMapWorkerReply>,
    state: WarpMapWorkerState,
    handle: Option<JoinHandle<()>>,
}

impl WarpMapWorker {
    pub fn new() -> Self {
        let (tx_worker_request, rx_worker_request) = mpsc::channel();
        let (tx_worker_reply, rx_worker_reply) = mpsc::channel();

        let state = WarpMapWorkerState::Idle;

        let handle = std::thread::spawn(move || {
            WarpMapWorkerThread::new(tx_worker_reply, rx_worker_request).run();
        });

        Self { tx_worker_request, rx_worker_reply, state, handle: Some(handle) }
    }

    pub fn is_idle(&self) -> bool {
        match self.state {
            WarpMapWorkerState::Idle => true,
            _ => false,
        }
    }

    pub fn start(&mut self, spec: WarpSpec) -> Result<()> {
        match self.state {
            WarpMapWorkerState::Idle => {
                self.tx_worker_request.send(WarpMapWorkerRequest::Start(spec))?;
                self.state = WarpMapWorkerState::Computing;
                Ok(())
            }
            WarpMapWorkerState::Computing => {
                bail!("worker busy");
            }
        }
    }

    pub fn retreive(&mut self) -> Result<Option<WarpMap>> {
        match self.state {
            WarpMapWorkerState::Computing => match self.rx_worker_reply.try_recv() {
                Ok(WarpMapWorkerReply::Finished(flow_map)) => {
                    self.state = WarpMapWorkerState::Idle;
                    Ok(Some(flow_map))
                }
                Err(mpsc::TryRecvError::Empty) => Ok(None),
                Err(mpsc::TryRecvError::Disconnected) => bail!("worker disconnected"),
            },
            WarpMapWorkerState::Idle => Ok(None),
        }
    }

    pub fn terminate(&mut self) {
        self.tx_worker_request.send(WarpMapWorkerRequest::Terminate).ok();
        if let Some(h) = self.handle.take() {
            h.join().ok();
        }
    }
}

enum WarpMapWorkerState {
    Idle,
    Computing,
}

#[allow(clippy::large_enum_variant)]
enum WarpMapWorkerRequest {
    Start(WarpSpec),
    Terminate,
}

enum WarpMapWorkerReply {
    Finished(Image<WarpPixel>),
}

struct WarpMapWorkerThread {
    rx_worker_request: mpsc::Receiver<WarpMapWorkerRequest>,
    tx_worker_reply: mpsc::Sender<WarpMapWorkerReply>,
}

impl WarpMapWorkerThread {
    pub fn new(
        tx_worker_reply: mpsc::Sender<WarpMapWorkerReply>,
        rx_worker_request: mpsc::Receiver<WarpMapWorkerRequest>,
    ) -> Self {
        Self { tx_worker_reply, rx_worker_request }
    }

    pub fn run(self) {
        loop {
            match self.rx_worker_request.try_recv() {
                Ok(WarpMapWorkerRequest::Start(spec)) => {
                    let mut fxgen = WarpGen::new(spec);
                    let fx = fxgen.run();
                    self.tx_worker_reply.send(WarpMapWorkerReply::Finished(fx)).unwrap();
                }
                Ok(WarpMapWorkerRequest::Terminate) => break,
                Err(mpsc::TryRecvError::Empty) => continue,
                Err(mpsc::TryRecvError::Disconnected) => break,
            }
        }
    }
}
