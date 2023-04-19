use dslab_compute::multicore::{CompFailed, CompFinished, CompStarted, Compute};
use dslab_core::{async_core::shared_state::DetailsKey, event::EventData};

use serde::Serialize;

#[derive(Serialize)]
pub struct Start {}

#[derive(Serialize)]
pub struct TaskRequest {
    pub flops: u64,
    pub memory: u64,
    pub cores: u32,
}

#[derive(Serialize)]
pub struct TakeTask {}

#[derive(Serialize)]
pub struct TaskCompleted {}

pub fn get_compute_start_id(data: &dyn EventData) -> DetailsKey {
    let event = data.downcast_ref::<CompStarted>().unwrap();
    return event.id;
}

pub fn get_compute_finished_id(data: &dyn EventData) -> DetailsKey {
    let event = data.downcast_ref::<CompFinished>().unwrap();
    return event.id;
}

pub fn get_compute_failed_id(data: &dyn EventData) -> DetailsKey {
    let event = data.downcast_ref::<CompFailed>().unwrap();
    return event.id;
}
