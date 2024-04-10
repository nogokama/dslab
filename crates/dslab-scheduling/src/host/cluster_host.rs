use std::{cell::RefCell, rc::Rc};

use dslab_compute::multicore::{CompFinished, CompStarted, Compute, CoresDependency};
use dslab_core::{cast, EventHandler, Id, SimulationContext};
use dslab_network::{models::shared, DataTransferCompleted, Network};
use dslab_storage::disk::Disk;
use dslab_storage::events::{DataReadCompleted, DataReadFailed, DataWriteCompleted, DataWriteFailed};
use dslab_storage::storage::{Storage, StorageInfo};
use futures::{select, FutureExt};

use crate::logger::{log_compute_load, log_memory_load};
use crate::storage::ProcessHostStorage;

use super::process::ProcessId;

pub struct ClusterHost {
    pub compute: Rc<RefCell<Compute>>,
    pub network: Option<Rc<RefCell<Network>>>,
    pub disk: Option<Rc<RefCell<Disk>>>,
    shared_info_storage: Rc<RefCell<ProcessHostStorage>>,
    ctx: SimulationContext,
}

impl ClusterHost {
    pub fn new(
        compute: Rc<RefCell<Compute>>,
        network: Option<Rc<RefCell<Network>>>,
        disk: Option<Rc<RefCell<Disk>>>,
        shared_info_storage: Rc<RefCell<ProcessHostStorage>>,
        ctx: SimulationContext,
    ) -> ClusterHost {
        ClusterHost {
            compute,
            network,
            disk,
            shared_info_storage,
            ctx,
        }
    }

    pub fn id(&self) -> Id {
        self.ctx.id()
    }

    pub async fn sleep(&self, time: f64) {
        self.ctx.sleep(time).await;
    }

    pub async fn run_flops(&self, flops: f64, compute_allocation_id: u64, cores_dependency: CoresDependency) {
        let req_id =
            self.compute
                .borrow_mut()
                .run_on_allocation(flops, compute_allocation_id, cores_dependency, self.ctx.id());

        self.ctx.recv_event_by_key::<CompStarted>(req_id).await;

        self.log_compute_load();

        self.ctx.recv_event_by_key::<CompFinished>(req_id).await;

        self.log_compute_load();
    }

    pub async fn transfer_data(&self, size: f64, dst_process: ProcessId) {
        let dst_host = self.shared_info_storage.borrow().get_host_id(dst_process);

        let network = self.network.as_ref().unwrap();

        let req_id = network
            .borrow_mut()
            .transfer_data(self.ctx.id(), dst_host, size, self.ctx.id());

        self.ctx.recv_event_by_key::<DataTransferCompleted>(req_id as u64).await;
    }

    pub async fn write_data(&self, size: u64) -> Result<(), String> {
        let req_id = self
            .disk
            .as_ref()
            .expect("disk must be configured to call disk operations")
            .borrow_mut()
            .write(size, self.ctx.id());

        select! {
            _ = self.ctx.recv_event_by_key::<DataWriteCompleted>(req_id).fuse() => {
                Result::Ok(())
            }
            failed = self.ctx.recv_event_by_key::<DataWriteFailed>(req_id).fuse() => {
                Result::Err(failed.data.error)
            }
        }
    }

    pub async fn read_data(&self, size: u64) -> Result<(), String> {
        let req_id = self
            .disk
            .as_ref()
            .expect("disk must be configured to call disk operations")
            .borrow_mut()
            .read(size, self.ctx.id());

        select! {
            _ = self.ctx.recv_event_by_key::<DataReadCompleted>(req_id).fuse() => {
                Result::Ok(())
            }
            failed = self.ctx.recv_event_by_key::<DataReadFailed>(req_id).fuse() => {
                Result::Err(failed.data.error)
            }
        }
    }

    fn log_compute_load(&self) {
        log_compute_load(
            self.ctx.time(),
            self.ctx.id(),
            1. - self.compute.borrow().cores_available() as f64 / self.compute.borrow().cores_total() as f64,
        );
        log_memory_load(
            self.ctx.time(),
            self.ctx.id(),
            1. - self.compute.borrow().memory_available() as f64 / self.compute.borrow().memory_total() as f64,
        );
    }
}

impl EventHandler for ClusterHost {
    fn on(&mut self, event: dslab_core::Event) {
        cast!(match event.data {
            CompFinished { id } => {
                println!("comp finished: {}", id);
            }
        })
    }
}
