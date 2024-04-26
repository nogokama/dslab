use std::collections::HashMap;

use dslab_core::Id;

use crate::workload_generators::events::ExecutionRequest;

pub struct SharedInfoStorage {
    jobs_info: HashMap<u64, ExecutionRequest>,
}

impl SharedInfoStorage {
    pub fn new() -> SharedInfoStorage {
        SharedInfoStorage {
            jobs_info: HashMap::new(),
        }
    }

    pub fn get_execution_request(&self, task_id: u64) -> ExecutionRequest {
        self.jobs_info.get(&task_id).unwrap().clone()
    }

    pub fn set_execution_request(&mut self, task_id: u64, task_request: ExecutionRequest) {
        self.jobs_info.insert(task_id, task_request);
    }
}
