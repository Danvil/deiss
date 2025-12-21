use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct Config {
    pub filename: String,
}

#[derive(Clone)]
pub struct SharedConfig(Arc<Mutex<Config>>);

impl SharedConfig {
    pub fn new(config: Config) -> Self {
        SharedConfig(Arc::new(Mutex::new(config)))
    }

    pub fn lock(&self) -> Config {
        self.0.lock().unwrap().clone()
    }
}
