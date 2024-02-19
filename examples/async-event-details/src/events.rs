use serde::Serialize;

use dslab_compute::multicore::{CompFailed, CompFinished, CompStarted};
use dslab_core::{async_core::await_details::EventKey, event::EventData};

#[derive(Serialize, Clone)]
pub struct Start {}

#[derive(Serialize, Clone)]
pub struct TaskCompleted {}

#[derive(Serialize, Clone)]
pub struct TaskRequest {
    pub flops: f64,
    pub memory: u64,
    pub cores: u32,
}

pub fn get_compute_start_id(data: &dyn EventData) -> EventKey {
    let event = data.downcast_ref::<CompStarted>().unwrap();
    event.id
}

pub fn get_compute_finished_id(data: &dyn EventData) -> EventKey {
    let event = data.downcast_ref::<CompFinished>().unwrap();
    event.id
}

pub fn get_compute_failed_id(data: &dyn EventData) -> EventKey {
    let event = data.downcast_ref::<CompFailed>().unwrap();
    event.id
}
