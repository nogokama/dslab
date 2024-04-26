use std::rc::Rc;

use serde::{Deserialize, Serialize};

use crate::execution_profiles::profile::ExecutionProfile;

#[derive(Serialize, Clone)]
pub struct ExecutionRequest {
    pub id: Option<u64>,
    pub name: Option<String>,
    pub collection_id: Option<u64>,
    pub time: f64,
    pub resources: ResourceRequirements,
    #[serde(skip)]
    pub profile: Rc<dyn ExecutionProfile>,
    pub wall_time_limit: Option<f64>,
    pub priority: Option<u64>,
}

#[derive(Serialize, Clone)]
pub struct CancelRequest {
    pub execution_id: u64,
    pub collection_id: Option<u64>,
    pub time: f64,
}

#[derive(Serialize, Clone)]
pub struct CollectionEvent {
    pub id: u64,
    pub priority: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ResourceRequirements {
    pub nodes_count: u32,
    pub cpu_per_node: u32,
    pub memory_per_node: u64,
}
