use std::mem;

use dslab_core::{log_info, Id, SimulationContext};
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
            let machine = match id % 2 {
                0 => Machine {
                    id,
                    cpu_cores: self.ctx.gen_range(8..20),
                    memory: self.ctx.gen_range(50_000..128_000),
                },
                1 => Machine {
                    id,
                    cpu_cores: self.ctx.gen_range(40..60),
                    memory: self.ctx.gen_range(10_000..30_000),
                },
                _ => panic!("unreachable"),
            };
            log_info!(self.ctx, "{:?}", machine);
            let ev = HostAdded { machine, time: 0. };

            self.ctx.emit_now(ev.clone(), proxy_id);
            machines.push(ev);
        }

        let mut time = 1.;
        for id in 0..self.tasks_count {
            let mut task_request: Option<TaskRequest>;
            if self.ctx.gen_range(1..10) < 5 {
                task_request = Some(TaskRequest {
                    id: id as u64,
                    cpu_cores: self.ctx.gen_range(5..10),
                    memory: self.ctx.gen_range(1000..5000),
                    flops: 5. * self.ctx.rand(),
                });
            } else {
                task_request = Some(TaskRequest {
                    id: id as u64,
                    cpu_cores: self.ctx.gen_range(1..6),
                    memory: self.ctx.gen_range(5000..20000),
                    flops: 5. * self.ctx.rand(),
                });
            }
            self.ctx.emit(task_request.unwrap(), proxy_id, time);

            time += self.ctx.rand() / 1000.;
        }

        machines
    }
}
