use crate::{
    painter::{
        flow::{FlowMap, FlowMapGen, FlowMapSpec, FxPxl},
        globals::Globals,
        mode_blueprint_library::ModeBlueprintLibrary,
        settings::Settings,
    },
    utils::*,
};
use eyre::{Result, bail};
use std::{
    sync::mpsc,
    thread::JoinHandle,
    time::{Duration, Instant},
};

pub struct FlowMapHub {
    current: Option<(FlowMapSpec, FlowMap)>,
    next_spec: Option<FlowMapSpec>,
    worker: FlowMapWorker,
    next_switch_time: Instant,
}

impl FlowMapHub {
    pub fn new() -> Self {
        Self {
            current: None,
            next_spec: None,
            worker: FlowMapWorker::new(),
            next_switch_time: Instant::now(),
        }
    }

    pub fn step(&mut self, s: &Settings, fx: &ModeBlueprintLibrary, g: &mut Globals) -> Result<()> {
        if self.worker.is_idle() {
            if self.next_switch_time < Instant::now() {
                let spec = FlowMapSpec::generate(s, fx, g);
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

    pub fn fetch(&mut self) -> Option<(FlowMapSpec, FlowMap)> {
        self.current.take()
    }
}

impl Drop for FlowMapHub {
    fn drop(&mut self) {
        self.worker.terminate();
    }
}

struct FlowMapWorker {
    tx_worker_request: mpsc::Sender<FlowMapWorkerRequest>,
    rx_worker_reply: mpsc::Receiver<FlowMapWorkerReply>,
    state: FlowMapWorkerState,
    handle: Option<JoinHandle<()>>,
}

impl FlowMapWorker {
    pub fn new() -> Self {
        let (tx_worker_request, rx_worker_request) = mpsc::channel();
        let (tx_worker_reply, rx_worker_reply) = mpsc::channel();

        let state = FlowMapWorkerState::Idle;

        let handle = std::thread::spawn(move || {
            FlowMapWorkerThread::new(tx_worker_reply, rx_worker_request).run();
        });

        Self { tx_worker_request, rx_worker_reply, state, handle: Some(handle) }
    }

    pub fn is_idle(&self) -> bool {
        match self.state {
            FlowMapWorkerState::Idle => true,
            _ => false,
        }
    }

    pub fn start(&mut self, spec: FlowMapSpec) -> Result<()> {
        match self.state {
            FlowMapWorkerState::Idle => {
                self.tx_worker_request.send(FlowMapWorkerRequest::Start(spec))?;
                self.state = FlowMapWorkerState::Computing;
                Ok(())
            }
            FlowMapWorkerState::Computing => {
                bail!("worker busy");
            }
        }
    }

    pub fn retreive(&mut self) -> Result<Option<FlowMap>> {
        match self.state {
            FlowMapWorkerState::Computing => match self.rx_worker_reply.try_recv() {
                Ok(FlowMapWorkerReply::Finished(flow_map)) => {
                    self.state = FlowMapWorkerState::Idle;
                    Ok(Some(flow_map))
                }
                Err(mpsc::TryRecvError::Empty) => Ok(None),
                Err(mpsc::TryRecvError::Disconnected) => bail!("worker disconnected"),
            },
            FlowMapWorkerState::Idle => Ok(None),
        }
    }

    pub fn terminate(&mut self) {
        self.tx_worker_request.send(FlowMapWorkerRequest::Terminate).ok();
        if let Some(h) = self.handle.take() {
            h.join().ok();
        }
    }
}

enum FlowMapWorkerState {
    Idle,
    Computing,
}

enum FlowMapWorkerRequest {
    Start(FlowMapSpec),
    Terminate,
}

enum FlowMapWorkerReply {
    Finished(Image<FxPxl>),
}

struct FlowMapWorkerThread {
    rx_worker_request: mpsc::Receiver<FlowMapWorkerRequest>,
    tx_worker_reply: mpsc::Sender<FlowMapWorkerReply>,
}

impl FlowMapWorkerThread {
    pub fn new(
        tx_worker_reply: mpsc::Sender<FlowMapWorkerReply>,
        rx_worker_request: mpsc::Receiver<FlowMapWorkerRequest>,
    ) -> Self {
        Self { tx_worker_reply, rx_worker_request }
    }

    pub fn run(self) {
        loop {
            match self.rx_worker_request.try_recv() {
                Ok(FlowMapWorkerRequest::Start(spec)) => {
                    let mut fxgen = FlowMapGen::new(spec);
                    let fx = fxgen.run();
                    self.tx_worker_reply.send(FlowMapWorkerReply::Finished(fx)).unwrap();
                }
                Ok(FlowMapWorkerRequest::Terminate) => break,
                Err(mpsc::TryRecvError::Empty) => continue,
                Err(mpsc::TryRecvError::Disconnected) => break,
            }
        }
    }
}
