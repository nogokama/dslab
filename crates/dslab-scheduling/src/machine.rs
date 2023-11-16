use dslab_core::Id;
use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct Machine {
    pub id: Id,
    pub cpu_cores: u32,
    pub memory: u64,
}
