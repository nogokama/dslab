use dslab_core::SimulationContext;

use crate::execution_profiles::builder::ProfileBuilder;

use super::{events::JobRequest, generator::WorkloadGenerator};

pub struct NativeWorkloadGenerator {
    pub path: String,
    profile_builder: ProfileBuilder,
}

impl NativeWorkloadGenerator {
    pub fn new(path: String, profile_builder: ProfileBuilder) -> NativeWorkloadGenerator {
        NativeWorkloadGenerator { path, profile_builder }
    }
}

impl WorkloadGenerator for NativeWorkloadGenerator {
    fn get_workload(&self, ctx: &SimulationContext) -> Vec<JobRequest> {
        vec![]
    }
}
