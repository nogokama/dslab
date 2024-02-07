use std::{
    collections::{HashMap, VecDeque},
    hash::Hash,
};

use dslab_core::{cast, EventHandler, Id, SimulationContext};
use dslab_scheduling::{
    cluster::{JobFinished, Schedule},
    cluster_events::HostAdded,
    config::sim_config::HostConfig,
    scheduler::{Resources, Scheduler},
    workload_generators::events::JobRequest,
};

pub struct JobInfo {
    id: u64,
    cpu_cores: u32,
    memory: u64,
}

pub struct RoundRobinScheduler {
    cluster_id: Id,
    hosts: Vec<HostConfig>,
    available_resources: HashMap<Id, Resources>,
    queue: VecDeque<JobInfo>,
    tasks: HashMap<u64, Resources>,

    ctx: SimulationContext,
}

impl RoundRobinScheduler {
    pub fn new(cluster_id: Id, ctx: SimulationContext) -> RoundRobinScheduler {
        RoundRobinScheduler {
            cluster_id,
            hosts: Vec::new(),
            available_resources: HashMap::new(),
            queue: VecDeque::new(),
            tasks: HashMap::new(),
            ctx,
        }
    }

    fn on_task_info(&mut self, task_id: u64, cpu_cores: u32, memory: u64) {
        self.queue.push_back(JobInfo {
            id: task_id,
            cpu_cores,
            memory,
        });

        self.tasks.insert(task_id, Resources { cpu_cores, memory });

        self.schedule();
    }

    fn on_task_finished(&mut self, task_id: u64, hosts: Vec<Id>) {
        let resources = self.tasks.remove(&task_id).unwrap();
        for host_id in hosts {
            if let Some(host) = self.available_resources.get_mut(&host_id) {
                host.cpu_cores += resources.cpu_cores;
                host.memory += resources.memory;
            }
        }

        self.schedule();
    }

    fn schedule(&mut self) {
        while let Some(job) = self.queue.pop_front() {
            let mut scheduled = false;
            for machine in &self.hosts {
                if let Some(resources) = self.available_resources.get_mut(&machine.id) {
                    if resources.cpu_cores >= job.cpu_cores && resources.memory >= job.memory {
                        resources.cpu_cores -= job.cpu_cores;
                        resources.memory -= job.memory;
                        self.ctx.emit_now(
                            Schedule {
                                job_id: job.id,
                                host_ids: vec![machine.id],
                            },
                            self.cluster_id,
                        );
                        scheduled = true;
                        break;
                    }
                }
            }

            if !scheduled {
                self.queue.push_front(job);
                break;
            }
        }
    }
}

impl EventHandler for RoundRobinScheduler {
    fn on(&mut self, event: dslab_core::Event) {
        cast!(match event.data {
            HostAdded { host } => {
                self.available_resources.insert(
                    host.id,
                    Resources {
                        cpu_cores: host.cpus,
                        memory: host.memory,
                    },
                );
                self.hosts.push(host);
            }
            JobRequest { id, resources, .. } => {
                self.on_task_info(id.unwrap(), resources.cpu_per_node, resources.memory_per_node);
            }
            JobFinished { job_id, hosts } => {
                self.on_task_finished(job_id, hosts);
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
