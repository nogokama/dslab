use std::mem;

use dslab_core::{Id, SimulationContext};
use serde::Serialize;

use crate::machine::Machine;

pub trait EventGenerator {
    fn schedule_events(&mut self, proxy_id: Id) -> Vec<HostAdded>;
}

#[derive(Serialize, Clone, Copy)]
pub struct TaskRequest {
    pub id: u64,
    pub cpu_cores: u32,
    pub memory: u64,
    pub flops: f64,
}

#[derive(Serialize, Clone)]
pub struct TaskInfo {
    pub id: u64,
    pub cpu_cores: u32,
    pub memory: u64,
}

#[derive(Serialize, Clone)]
pub struct HostAdded {
    pub machine: Machine,
    pub time: f64,
}

#[derive(Serialize, Clone)]
pub struct HostRemoved {
    pub id: Id,
}

pub struct RandomEventGenerator {
    ctx: SimulationContext,
    tasks_count: u32,
}

impl RandomEventGenerator {
    pub fn new(ctx: SimulationContext, tasks_count: u32) -> RandomEventGenerator {
        return Self { ctx, tasks_count };
    }
}

impl EventGenerator for RandomEventGenerator {
    fn schedule_events(&mut self, proxy_id: Id) -> Vec<HostAdded> {
        let hosts_count = 3;
        let mut machines = Vec::new();
        for id in 0..hosts_count {
            let ev = HostAdded {
                machine: Machine {
                    cpu_cores: self.ctx.gen_range(8..64),
                    memory: self.ctx.gen_range(5_000..128_000),
                    id,
                },
                time: 0.,
            };
            self.ctx.emit_now(ev.clone(), proxy_id);
            machines.push(ev);
        }

        let mut time = 1.;
        for id in 0..self.tasks_count {
            self.ctx.emit(
                TaskRequest {
                    id: id as u64,
                    cpu_cores: self.ctx.gen_range(1..8),
                    memory: self.ctx.gen_range(1000..5000),
                    flops: self.ctx.rand(),
                },
                proxy_id,
                time,
            );

            time += self.ctx.rand() / 1000.;
        }

        machines
    }
}
