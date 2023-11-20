use std::collections::{HashMap, VecDeque};

use dslab_core::{cast, EventHandler, Id, SimulationContext};
use dslab_scheduling::{
    cluster::{Schedule, TaskFinished},
    event_generator::{HostAdded, TaskInfo},
    machine::Machine,
    scheduler::{Resources, Scheduler},
};

pub struct RoundRobinScheduler {
    cluster_id: Id,
    machines: Vec<Machine>,
    available_resources: HashMap<Id, Resources>,
    queue: VecDeque<TaskInfo>,

    ctx: SimulationContext,
}

impl RoundRobinScheduler {
    pub fn new(cluster_id: Id, ctx: SimulationContext) -> RoundRobinScheduler {
        RoundRobinScheduler {
            cluster_id,
            machines: Vec::new(),
            available_resources: HashMap::new(),
            queue: VecDeque::new(),
            ctx,
        }
    }

    fn on_task_info(&mut self, task_id: u64, cpu_cores: u32, memory: u64) {
        self.queue.push_back(TaskInfo {
            id: task_id,
            cpu_cores,
            memory,
        });

        self.schedule();
    }

    fn on_task_finished(&mut self, task_id: u64, machine_id: Id, available_cores: u32, available_memory: u64) {
        self.available_resources.insert(
            machine_id,
            Resources {
                cpu_cores: available_cores,
                memory: available_memory,
            },
        );

        self.schedule();
    }

    fn schedule(&mut self) {
        while let Some(task) = self.queue.pop_front() {
            let mut scheduled = false;
            for machine in &self.machines {
                if let Some(resources) = self.available_resources.get_mut(&machine.id) {
                    if resources.cpu_cores >= task.cpu_cores && resources.memory >= task.memory {
                        resources.cpu_cores -= task.cpu_cores;
                        resources.memory -= task.memory;
                        self.ctx.emit_now(
                            Schedule {
                                task_id: task.id,
                                machine_id: machine.id,
                            },
                            self.cluster_id,
                        );
                        scheduled = true;
                        break;
                    }
                }
            }

            if !scheduled {
                self.queue.push_front(task);
                break;
            }
        }
    }
}

impl EventHandler for RoundRobinScheduler {
    fn on(&mut self, event: dslab_core::Event) {
        cast!(match event.data {
            HostAdded { machine, time: _ } => {
                self.available_resources.insert(
                    machine.id,
                    Resources {
                        cpu_cores: machine.cpu_cores,
                        memory: machine.memory,
                    },
                );
                self.machines.push(machine);
            }
            TaskInfo { id, cpu_cores, memory } => {
                self.on_task_info(id, cpu_cores, memory)
            }
            TaskFinished {
                task_id,
                machine_id,
                available_cores,
                available_memory,
            } => {
                self.on_task_finished(task_id, machine_id, available_cores, available_memory)
            }
        })
    }
}

impl Scheduler for RoundRobinScheduler {
    fn id(&self) -> Id {
        self.ctx.id()
    }

    fn name(&self) -> String {
        self.ctx.name().to_string()
    }
}
