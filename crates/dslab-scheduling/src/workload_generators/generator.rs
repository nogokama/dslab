use dslab_core::{Id, SimulationContext};

use super::events::JobRequest;

pub trait WorkloadGenerator {
    fn get_workload(&self, ctx: &SimulationContext) -> Vec<JobRequest>;
}
