use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
};

use dslab_compute::multicore::{CompFinished, CompStarted, Compute, CoresDependency};
use dslab_core::{cast, event::EventId, log_debug, log_error, Event, EventHandler, Id, Simulation, SimulationContext};
use serde::Serialize;

use crate::{
    event_generator::{TaskInfo, TaskRequest},
    machine::Machine,
    proxy::Proxy,
    storage::TaskInfoStorage,
};

#[derive(Clone, Serialize)]
pub struct ScheduleError {
    pub task_id: u64,
    pub error_message: String,
}

#[derive(Clone, Serialize)]
pub struct Schedule {
    pub task_id: u64,
    pub machine_id: Id,
}

#[derive(Clone, Serialize)]
pub struct Cancel {
    pub task_id: u64,
}

#[derive(Clone, Serialize)]
pub struct TaskFinished {
    pub task_id: u64,
    pub machine_id: Id,
    pub available_cores: u32,
    pub available_memory: u64,
}

pub(crate) struct Cluster {
    machines: HashMap<Id, Rc<RefCell<Compute>>>,
    compute_id_to_machine_id: HashMap<Id, Id>,
    tasks: HashMap<Id, HashSet<u64>>,
    computation_id_to_task_id: HashMap<EventId, u64>,

    task_storage: Rc<RefCell<TaskInfoStorage>>,
    scheduler_id: Id,
    ctx: SimulationContext,
}

impl Cluster {
    pub(crate) fn new(ctx: SimulationContext, task_storage: Rc<RefCell<TaskInfoStorage>>) -> Cluster {
        Cluster {
            machines: HashMap::new(),
            tasks: HashMap::new(),
            compute_id_to_machine_id: HashMap::new(),
            computation_id_to_task_id: HashMap::new(),

            scheduler_id: u32::MAX, // must be set later
            task_storage,
            ctx,
        }
    }

    pub fn set_scheduler(&mut self, scheduler_id: Id) {
        self.scheduler_id = scheduler_id;
    }

    pub fn get_id(&self) -> Id {
        self.ctx.id()
    }

    pub fn add_compute(&mut self, id: Id, compute_id: Id, compute: Rc<RefCell<Compute>>) {
        self.machines.insert(id, compute);
        self.compute_id_to_machine_id.insert(compute_id, id);
    }

    fn get_all_machines(&self) -> Vec<Machine> {
        self.machines
            .iter()
            .map(|(id, c)| Machine {
                id: (*id) as Id,
                cpu_cores: c.borrow().cores_total(),
                memory: c.borrow().memory_total(),
            })
            .collect()
    }

    fn get_machine_info(&self, machine_id: Id) -> Machine {
        let c = self.machines.get(&machine_id).unwrap();
        Machine {
            id: machine_id,
            cpu_cores: c.borrow().cores_total(),
            memory: c.borrow().memory_total(),
        }
    }

    fn get_machine_load(&self, machine_id: Id) -> Vec<TaskInfo> {
        vec![]
    }

    fn schedule_task(&mut self, machine_id: Id, task_id: u64) {
        let c = self.machines.get(&machine_id).unwrap();

        let request = self.task_storage.borrow().get_task_request(task_id);

        let comp_id = c.borrow_mut().run(
            request.flops,
            request.memory,
            request.cpu_cores,
            request.cpu_cores,
            CoresDependency::Linear,
            self.ctx.id(),
        );

        // TODO use async to handle possible cases

        self.tasks.entry(machine_id).or_insert(HashSet::new()).insert(task_id);
        self.computation_id_to_task_id.insert(comp_id, task_id);
    }

    fn cancel_task(&self, task_id: u64) {
        log_error!(self.ctx, "cancel task: {} not implemented", task_id)
    }
}

impl EventHandler for Cluster {
    fn on(&mut self, event: Event) {
        cast!(match event.data {
            Schedule { task_id, machine_id } => {
                log_debug!(self.ctx, "schedule task: {} on machine: {}", task_id, machine_id);
                self.schedule_task(machine_id, task_id);
            }
            Cancel { task_id } => {
                self.cancel_task(task_id)
            }
            CompStarted { id: _, cores: _ } => {}
            CompFinished { id } => {
                let machine_id = *self.compute_id_to_machine_id.get(&event.src).unwrap();
                let compute = self.machines.get(&machine_id).unwrap();
                let task_id = *self.computation_id_to_task_id.get(&id).unwrap();

                self.tasks.get_mut(&machine_id).unwrap().remove(&task_id);

                self.ctx.emit_now(
                    TaskFinished {
                        task_id,
                        machine_id,
                        available_cores: compute.borrow().cores_available(),
                        available_memory: compute.borrow().memory_available(),
                    },
                    self.scheduler_id,
                );
            }
        });
    }
}
