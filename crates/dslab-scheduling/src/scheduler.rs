use dslab_core::{EventHandler, Id};
use serde::Serialize;

pub struct Resources {
    pub cpu_cores: u32,
    pub memory: u64,
}

pub trait Scheduler {
    fn name(&self) -> String;
    fn id(&self) -> Id;
}
