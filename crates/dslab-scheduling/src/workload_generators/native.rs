use dslab_core::SimulationContext;

use super::{events::JobRequest, generator::WorkloadGenerator};

pub struct NativeWorkloadGenerator {
    pub path: String,
    // profile_builder: ProfileBuilder,
}

impl NativeWorkloadGenerator {
    pub fn new(path: String) -> NativeWorkloadGenerator {
        NativeWorkloadGenerator { path }
    }
}

impl WorkloadGenerator for NativeWorkloadGenerator {
    fn get_workload(&self, ctx: &SimulationContext) -> Vec<JobRequest> {}
}
