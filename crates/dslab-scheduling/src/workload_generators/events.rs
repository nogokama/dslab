use std::rc::Rc;

use serde::Serialize;

use crate::execution_profiles::ExecutionProfile;

#[derive(Serialize, Clone)]
pub struct JobRequest {
    pub id: Option<u64>,
    pub time: f64,
    pub resources: ResourceRequirements,
    #[serde(skip)]
    pub profile: Rc<dyn ExecutionProfile>,
    pub wall_time_limit: Option<f64>,
}

#[derive(Serialize, Clone)]
pub struct ResourceRequirements {
    pub nodes_count: u32,
    pub cpu_per_node: u32,
    pub memory_per_node: u64,
}
