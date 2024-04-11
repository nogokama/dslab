use std::collections::HashMap;

use dslab_core::Id;

use crate::workload_generators::events::JobRequest;

pub struct SharedInfoStorage {
    jobs_info: HashMap<u64, JobRequest>,
}

impl SharedInfoStorage {
    pub fn new() -> SharedInfoStorage {
        SharedInfoStorage {
            jobs_info: HashMap::new(),
        }
    }

    pub fn get_job_request(&self, task_id: u64) -> JobRequest {
        self.jobs_info.get(&task_id).unwrap().clone()
    }

    pub fn set_job_request(&mut self, task_id: u64, task_request: JobRequest) {
        self.jobs_info.insert(task_id, task_request);
    }
}
