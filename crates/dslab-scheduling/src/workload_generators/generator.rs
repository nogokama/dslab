use dslab_core::{Id, SimulationContext};

use super::events::ExecutionRequest;

pub trait WorkloadGenerator {
    fn get_workload(&self, ctx: &SimulationContext) -> Vec<ExecutionRequest>;
}
