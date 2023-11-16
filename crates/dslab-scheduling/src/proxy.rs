use std::{cell::RefCell, collections::HashMap, rc::Rc};

use dslab_core::{cast, log_debug, EventHandler, Id, SimulationContext};

use crate::{
    event_generator::{HostAdded, TaskInfo, TaskRequest},
    storage::TaskInfoStorage,
};

pub struct Proxy {
    tasks_scheduled_time: HashMap<u64, f64>,
    scheduler_id: Id,
    task_storage: Rc<RefCell<TaskInfoStorage>>,

    ctx: SimulationContext,
}

impl Proxy {
    pub fn new(ctx: SimulationContext, task_storage: Rc<RefCell<TaskInfoStorage>>) -> Proxy {
        Proxy {
            scheduler_id: u32::MAX,
            tasks_scheduled_time: HashMap::new(),
            task_storage,
            ctx,
        }
    }

    pub fn get_id(&self) -> Id {
        self.ctx.id()
    }

    pub fn set_scheduler(&mut self, scheduler_id: Id) {
        self.scheduler_id = scheduler_id;
    }
}

impl EventHandler for Proxy {
    fn on(&mut self, event: dslab_core::Event) {
        cast!(match event.data {
            TaskRequest {
                id,
                cpu_cores,
                memory,
                flops,
            } => {
                self.task_storage.borrow_mut().set_task_request(
                    id,
                    TaskRequest {
                        id,
                        cpu_cores,
                        memory,
                        flops,
                    },
                );
                self.tasks_scheduled_time.insert(id, self.ctx.time());

                self.ctx.emit_now(TaskInfo { id, cpu_cores, memory }, self.scheduler_id);
            }
            HostAdded { machine, time } => {
                log_debug!(self.ctx, "HostAdded: {}, {}", machine.id, time);
                self.ctx.emit_now(HostAdded { machine, time }, self.scheduler_id);
            }
        })
    }
}
