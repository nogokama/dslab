use std::{cell::RefCell, collections::VecDeque, rc::Rc};

use dslab_compute::multicore::{CompFinished, CompStarted, Compute};
use dslab_core::{async_core::task::Task, cast, event::EventId, log_debug, Event, EventHandler, Id, SimulationContext};
use log::debug;

use sugars::{rc, refcell};

use crate::events::{Start, TakeTask, TaskRequest};

struct TaskInfo {
    flops: u64,
    memory: u64,
    cores: u32,
}

pub struct Worker {
    id: Id,
    compute: Rc<RefCell<Compute>>,
    compute_id: Id,
    ctx: SimulationContext,
    tasks_queue: RefCell<VecDeque<TaskInfo>>,
}

impl Worker {
    pub fn new(compute: Rc<RefCell<Compute>>, compute_id: Id, ctx: SimulationContext) -> Self {
        Self {
            id: ctx.id(),
            compute,
            compute_id,
            ctx,
            tasks_queue: refcell!(VecDeque::new()),
        }
    }

    pub fn id(&self) -> Id {
        self.id
    }

    fn on_start(&self) {
        debug!("Worker started");
        self.ctx.spawn(self.work_loop());
    }

    fn on_task_request(&self, task_info: TaskInfo) {
        if self.tasks_queue.borrow().is_empty() {
            self.ctx.emit_self_now(TakeTask {});
        }
        self.tasks_queue.borrow_mut().push_back(task_info);
    }

    async fn work_loop(&self) {
        let mut tasks_completed = 0;
        loop {
            if self.tasks_queue.borrow().is_empty() {
                self.ctx.async_handle_self::<TakeTask>().await;
            }

            self.process_task().await;

            tasks_completed += 1;

            log_debug!(
                self.ctx,
                format!("Worker::work_loop : task {} completed", tasks_completed)
            );
        }
    }

    async fn try_process_task(&self, task_info: TaskInfo) -> bool {
        let key = self.run_task(task_info);

        return true;
    }

    async fn process_task(&self) {
        let task_info = self.tasks_queue.borrow_mut().pop_front().unwrap();
        self.run_task(task_info);

        self.ctx.async_handle_event::<CompStarted>(self.compute_id).await;

        self.ctx.async_handle_event::<CompFinished>(self.compute_id).await;
    }

    fn run_task(&self, task_info: TaskInfo) -> EventId {
        self.compute.borrow_mut().run(
            task_info.flops,
            task_info.memory,
            task_info.cores,
            task_info.cores,
            dslab_compute::multicore::CoresDependency::Linear,
            self.id(),
        )
    }
}

impl EventHandler for Worker {
    fn on(&mut self, event: Event) {
        cast!(match event.data {
            TaskRequest { flops, cores, memory } => {
                self.on_task_request(TaskInfo { flops, cores, memory });
            }
            Start {} => {
                self.on_start();
            }
            TakeTask {} => {}
        })
    }
}
