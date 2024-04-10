use std::{collections::HashMap, option, rc::Rc};

use dslab_core::{log_info, Id, Simulation, SimulationContext};
use serde::{Deserialize, Serialize};

use crate::execution_profiles::default::CpuBurnHomogenous;

use super::{
    events::{JobRequest, ResourceRequirements},
    generator::WorkloadGenerator,
};

#[derive(Serialize, Deserialize)]
pub struct RandomWorkloadGenerator {
    jobs_count: u32,
    cpu_min: u32,
    cpu_max: u32,
    memory_min: u64,
    memory_max: u64,
    delay_min: f64,
    delay_max: f64,
    load_min: f64,
    load_max: f64,
    start_time: Option<f64>,
    nodes_count_min: Option<u32>,
    nodes_count_max: Option<u32>,
}

impl RandomWorkloadGenerator {
    pub fn from_options(options: &HashMap<String, String>) -> Self {
        serde_yaml::from_str(&serde_yaml::to_string(options).unwrap().replace("\"", "")).unwrap()
    }
}

impl WorkloadGenerator for RandomWorkloadGenerator {
    fn get_workload(&self, ctx: &SimulationContext) -> Vec<JobRequest> {
        let mut workload = Vec::new();
        workload.reserve(self.jobs_count as usize);

        let mut time = self.start_time.unwrap_or(0.);

        for id in 0..self.jobs_count as u64 {
            let job = JobRequest {
                id: None,
                time,
                resources: ResourceRequirements {
                    nodes_count: 1,
                    cpu_per_node: ctx.gen_range(self.cpu_min..=self.cpu_max),
                    memory_per_node: ctx.gen_range(self.memory_min..=self.memory_max),
                },
                wall_time_limit: None,
                profile: Rc::new(CpuBurnHomogenous {
                    flops: ctx.gen_range(self.load_min..=self.load_max),
                }),
            };

            time += ctx.gen_range(self.delay_min..=self.delay_max);

            workload.push(job);
        }

        workload
    }
}
