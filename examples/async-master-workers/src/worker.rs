use std::cell::RefCell;
use std::rc::Rc;

use serde::Serialize;

use dslab_compute::multicore::*;
use dslab_core::async_core::await_details::EventKey;
use dslab_core::component::Id;
use dslab_core::context::SimulationContext;
use dslab_core::event::Event;
use dslab_core::handler::EventHandler;
use dslab_core::{cast, log_debug};
use dslab_network::model::*;
use dslab_network::network::Network;
use dslab_storage::disk::Disk;
use dslab_storage::events::{DataReadCompleted, DataWriteCompleted};
use dslab_storage::storage::Storage;

use crate::common::Start;
use crate::task::*;

#[derive(Serialize, Clone)]
pub struct WorkerRegister {
    pub(crate) speed: f64,
    pub(crate) cpus_total: u32,
    pub(crate) memory_total: u64,
}

#[derive(Serialize, Clone)]
pub struct TaskCompleted {
    pub(crate) id: u64,
}

pub struct AsyncWorker {
    id: Id,
    compute: Rc<RefCell<Compute>>,
    disk: Rc<RefCell<Disk>>,
    net: Rc<RefCell<Network>>,
    master_id: Id,
    ctx: SimulationContext,
}

impl AsyncWorker {
    pub fn new(
        compute: Rc<RefCell<Compute>>,
        disk: Rc<RefCell<Disk>>,
        net: Rc<RefCell<Network>>,
        master_id: Id,
        ctx: SimulationContext,
    ) -> Self {
        Self {
            id: ctx.id(),
            compute,
            disk,
            net,
            master_id,
            ctx,
        }
    }

    fn on_start(&mut self) {
        log_debug!(self.ctx, "started");
        self.ctx.emit(
            WorkerRegister {
                speed: self.compute.borrow().speed(),
                cpus_total: self.compute.borrow().cores_total(),
                memory_total: self.compute.borrow().memory_total(),
            },
            self.master_id,
            0.5,
        );
    }

    fn on_task_request(&self, req: TaskRequest) {
        self.ctx.spawn(self.process_task_request(req));
    }

    async fn process_task_request(&self, req: TaskRequest) {
        let mut task = TaskInfo {
            req,
            state: TaskState::Downloading,
        };

        // 1. download data
        self.download_data(&task).await;

        // 2. read data from disk
        task.state = TaskState::Reading;
        self.read_data(&task).await;

        // 3. run task
        task.state = TaskState::Running;
        self.run_task(&task).await;

        // 4. write data
        task.state = TaskState::Writing;
        self.write_data(&task).await;

        // 5. uploading result
        task.state = TaskState::Uploading;
        self.upload_result(&task).await;

        // 6. completed
        task.state = TaskState::Completed;
    }

    async fn download_data(&self, task: &TaskInfo) {
        let transfer_id =
            self.net
                .borrow_mut()
                .transfer_data(self.master_id, self.id, task.req.input_size as f64, self.id);
        self.ctx
            .recv_event_by_key::<DataTransferCompleted>(transfer_id as EventKey)
            .await;
        log_debug!(self.ctx, "downloaded input data for task: {}", task.req.id);
    }

    async fn read_data(&self, task: &TaskInfo) {
        let read_id = self.disk.borrow_mut().read(task.req.input_size, self.id);
        self.ctx.recv_event_by_key::<DataReadCompleted>(read_id).await;
        log_debug!(self.ctx, "read input data for task: {}", task.req.id);
    }

    async fn run_task(&self, task: &TaskInfo) {
        let comp_id = self.compute.borrow_mut().run(
            task.req.flops,
            task.req.memory,
            task.req.min_cores,
            task.req.max_cores,
            task.req.cores_dependency,
            self.id,
        );
        self.ctx.recv_event_by_key::<CompStarted>(comp_id as EventKey).await;
        log_debug!(self.ctx, "started execution of task: {}", task.req.id);

        self.ctx.recv_event_by_key::<CompFinished>(comp_id as EventKey).await;
        log_debug!(self.ctx, "completed execution of task: {}", task.req.id);
    }

    async fn write_data(&self, task: &TaskInfo) {
        let write_id = self.disk.borrow_mut().write(task.req.output_size, self.id);
        self.ctx.recv_event_by_key::<DataWriteCompleted>(write_id).await;
        log_debug!(self.ctx, "wrote output data for task: {}", task.req.id);
    }

    async fn upload_result(&self, task: &TaskInfo) {
        let transfer_id =
            self.net
                .borrow_mut()
                .transfer_data(self.id, self.master_id, task.req.output_size as f64, self.id);
        self.ctx
            .recv_event_by_key::<DataTransferCompleted>(transfer_id as EventKey)
            .await;
        log_debug!(self.ctx, "uploaded output data for task: {}", task.req.id);
        self.disk
            .borrow_mut()
            .mark_free(task.req.output_size)
            .expect("Failed to free disk space");
        self.net
            .borrow_mut()
            .send_event(TaskCompleted { id: task.req.id }, self.id, self.master_id);
    }
}

impl EventHandler for AsyncWorker {
    fn on(&mut self, event: Event) {
        cast!(match event.data {
            Start {} => {
                self.on_start();
            }
            TaskRequest {
                id,
                flops,
                memory,
                min_cores,
                max_cores,
                cores_dependency,
                input_size,
                output_size,
            } => {
                self.on_task_request(TaskRequest {
                    id,
                    flops,
                    memory,
                    min_cores,
                    max_cores,
                    cores_dependency,
                    input_size,
                    output_size,
                });
            }
        })
    }
}
