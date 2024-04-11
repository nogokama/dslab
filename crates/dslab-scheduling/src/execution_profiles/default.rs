use std::{os::unix::process, rc::Rc};

use async_trait::async_trait;
use dslab_compute::multicore::CoresDependency;
use futures::future::join_all;
use serde::Deserialize;

use crate::host::process::HostProcessInstance;

use crate::execution_profiles::profile::ExecutionProfile;

#[derive(Deserialize)]
pub struct CpuBurnHomogenous {
    pub flops: f64,
}

#[async_trait(?Send)]
impl ExecutionProfile for CpuBurnHomogenous {
    async fn run(self: Rc<Self>, processes: &Vec<HostProcessInstance>) {
        join_all(
            processes
                .iter()
                .map(|p| p.run_flops(self.flops, CoresDependency::Linear)),
        )
        .await;
    }

    fn get_name(&self) -> String {
        "cpu-burn-homogenous".to_string()
    }
}

#[derive(Deserialize)]
pub struct CommunicationHomogenous {
    pub size: f64,
}

#[async_trait(?Send)]
impl ExecutionProfile for CommunicationHomogenous {
    async fn run(self: Rc<Self>, processes: &Vec<HostProcessInstance>) {
        let mut futures = vec![];
        for i in 0..processes.len() {
            for j in 0..processes.len() {
                if i != j {
                    futures.push(processes[i].transfer_data(self.size, processes[j].id));
                }
            }
        }
    }

    fn get_name(&self) -> String {
        "communication-homogenous".to_string()
    }
}

#[derive(Deserialize)]
pub struct MasterWorkers {
    pub master_flops: f64,
    pub worker_flops: f64,
    pub data_transfer_bytes: f64,
}

#[async_trait(?Send)]
impl ExecutionProfile for MasterWorkers {
    async fn run(self: Rc<Self>, processes: &Vec<HostProcessInstance>) {
        let master_process = &processes[0];

        let worker_processes = &processes[1..];

        join_all(worker_processes.iter().map(|p| async {
            p.run_flops(self.worker_flops, CoresDependency::Linear).await;
            p.transfer_data(self.data_transfer_bytes, master_process.id).await;
        }))
        .await;

        master_process
            .run_flops(self.master_flops, CoresDependency::Linear)
            .await;
    }

    fn get_name(&self) -> String {
        "master-workers-simple".to_string()
    }
}
