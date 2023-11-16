use std::collections::HashMap;

use crate::event_generator::TaskRequest;

pub struct TaskInfoStorage {
    tasks_info: HashMap<u64, TaskRequest>,
}

impl TaskInfoStorage {
    pub fn new() -> TaskInfoStorage {
        TaskInfoStorage {
            tasks_info: HashMap::new(),
        }
    }

    pub fn get_task_request(&self, task_id: u64) -> TaskRequest {
        *self.tasks_info.get(&task_id).unwrap()
    }

    pub fn set_task_request(&mut self, task_id: u64, task_request: TaskRequest) {
        self.tasks_info.insert(task_id, task_request);
    }
}
