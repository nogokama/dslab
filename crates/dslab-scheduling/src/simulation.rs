use core::{panic, time};
use std::{cell::RefCell, collections::HashSet, rc::Rc, vec};

use dslab_compute::multicore::{AllocationSuccess, CompFinished, CompStarted, Compute, DeallocationSuccess};
use dslab_core::{event, EventHandler, Id, Simulation, SimulationContext};
use dslab_network::{
    models::{ConstantBandwidthNetworkModel, SharedBandwidthNetworkModel},
    Network, NetworkModel,
};
use sugars::{boxed, rc, refcell};

use crate::{
    cluster::Cluster,
    cluster_events::HostAdded,
    config::sim_config::{GroupHostConfig, HostConfig, NetworkConfig, SimulationConfig},
    host::cluster_host::ClusterHost,
    proxy::Proxy,
    scheduler::Scheduler,
    storage::SharedInfoStorage,
    workload_generators::{
        generator::WorkloadGenerator, random::RandomWorkloadGenerator, workload_type::workload_resolver,
    },
};

pub struct ClusterSchedulingSimulation {
    sim: Simulation,
    workload_generators: Vec<Box<dyn WorkloadGenerator>>,

    cluster: Rc<RefCell<Cluster>>,
    proxy: Rc<RefCell<Proxy>>,
    shared_storage: Rc<RefCell<SharedInfoStorage>>,
    // TODO: monitoring service connected to proxy & cluster
}

impl ClusterSchedulingSimulation {
    pub fn new(
        mut sim: Simulation,
        config: SimulationConfig,
        network_opt: Option<Rc<RefCell<Network>>>,
    ) -> ClusterSchedulingSimulation {
        let shared_storage = rc!(refcell!(SharedInfoStorage::new()));
        let cluster_ctx = sim.create_context("cluster");
        let cluster_id = cluster_ctx.id();
        let cluster = rc!(refcell!(Cluster::new(cluster_ctx, shared_storage.clone())));
        sim.add_handler("cluster", cluster.clone());

        let proxy_ctx = sim.create_context("proxy");
        let proxy = rc!(refcell!(Proxy::new(proxy_ctx, cluster_id, shared_storage.clone())));
        sim.add_handler("proxy", proxy.clone());

        let generator_ctx = sim.create_context("generator");

        let mut cluster_simulation = ClusterSchedulingSimulation {
            sim,
            workload_generators: config
                .workload
                .as_ref()
                .unwrap()
                .iter()
                .map(|w| workload_resolver(w))
                .collect::<Vec<_>>(),
            cluster,
            proxy,
            shared_storage,
        };

        cluster_simulation.register_key_getters();

        cluster_simulation.build_cluster(config.hosts, config.network, network_opt);

        cluster_simulation
    }

    pub fn get_cluster_id(&self) -> Id {
        self.cluster.borrow().get_id()
    }

    fn build_network(&mut self, network_config: &NetworkConfig) -> Rc<RefCell<Network>> {
        let network_model: Box<dyn NetworkModel> = if network_config.shared.unwrap_or(false) {
            boxed!(SharedBandwidthNetworkModel::new(
                network_config.bandwidth,
                network_config.latency
            ))
        } else {
            boxed!(ConstantBandwidthNetworkModel::new(
                network_config.bandwidth,
                network_config.latency
            ))
        };

        let network_ctx = self.sim.create_context("network");
        let network = rc!(refcell!(Network::new(network_model, network_ctx)));
        self.sim.add_handler("network", network.clone());

        network
    }

    pub fn build_cluster(
        &mut self,
        hosts_groups: Vec<GroupHostConfig>,
        network_config: Option<NetworkConfig>,
        mut network: Option<Rc<RefCell<Network>>>,
    ) {
        if network.is_none() && network_config.is_some() {
            network = Some(self.build_network(network_config.as_ref().unwrap()));
        }

        for host_group in hosts_groups {
            if host_group.count.unwrap_or(1) == 1 {
                self.build_host(
                    HostConfig::from_group_config(&host_group, None),
                    network_config.as_ref(),
                    network.clone(),
                );
            } else {
                for i in 0..host_group.count.unwrap() {
                    self.build_host(
                        HostConfig::from_group_config(&host_group, Some(i)),
                        network_config.as_ref(),
                        network.clone(),
                    );
                }
            }
        }
    }

    pub fn build_host(
        &mut self,
        host_config: HostConfig,
        network_config: Option<&NetworkConfig>,
        network: Option<Rc<RefCell<Network>>>,
    ) {
        let cluster = self.cluster.borrow();
        let proxy_id = self.proxy.borrow().get_id();
        let ctx = self.sim.create_context("cluster");

        let host_name = format!("host-{}", host_config.id);
        let compute_name = format!("compute-{}", host_config.id);
        let compute_ctx = self.sim.create_context(&compute_name);
        let compute = rc!(refcell!(Compute::new(
            host_config.cpu_speed.unwrap_or(1000.),
            host_config.cpus,
            host_config.memory,
            compute_ctx
        )));

        if let Some(network) = network {
            network.borrow_mut().add_node(
                host_name,
                boxed!(SharedBandwidthNetworkModel::new(
                    host_config
                        .local_newtork_bw
                        .unwrap_or(network_config.unwrap().local_bandwidth),
                    host_config
                        .local_newtork_latency
                        .unwrap_or(network_config.unwrap().local_latency),
                )),
            );
        }

        let host = rc!(ClusterHost::new(
            host_config.id,
            compute,
            None,
            None,
            self.shared_storage.clone(),
            ctx
        ));

        cluster.add_host(host_config.clone(), host.clone());
    }

    pub fn run_with_custom_scheduler<T: EventHandler + Scheduler + 'static>(&mut self, scheduler: T) {
        let scheduler_id = scheduler.id();
        let name = scheduler.name().clone();
        self.sim.add_handler(name, rc!(refcell!(scheduler)));

        let host_generator_ctx = self.sim.create_context("host_generator");
        let hosts = self.cluster.borrow().get_hosts();
        for host in hosts {
            host_generator_ctx.emit_now(HostAdded { host }, scheduler_id);
        }

        self.cluster.borrow_mut().set_scheduler(scheduler_id);
        self.proxy.borrow_mut().set_scheduler(scheduler_id);

        self.generate_workload();

        // TODO for long simulation make a while loop
        self.sim.step_until_no_events();

        println!("SIMULATION FINISHED AT: {}", self.sim.time());
    }

    fn generate_workload(&mut self) {
        let proxy_id = self.proxy.borrow().get_id();

        let generator_ctx = self.sim.create_context("generator");

        let mut next_job_id = 0u64;
        let mut used_ids = HashSet::new();

        for workload_generator in self.workload_generators.iter() {
            let mut workload = workload_generator.get_workload(&generator_ctx);

            for job_request in workload.iter_mut() {
                if let Some(id) = job_request.id {
                    if used_ids.contains(&id) {
                        panic!("Job id {} is used twice", id);
                    }
                    used_ids.insert(id);
                } else {
                    while used_ids.contains(&next_job_id) {
                        next_job_id += 1;
                    }
                    job_request.id = Some(next_job_id);
                    next_job_id += 1;
                }
            }

            for job_request in workload {
                let time = job_request.time;
                generator_ctx.emit(job_request, proxy_id, time);
            }
        }
    }

    fn register_key_getters(&self) {
        self.sim.register_key_getter_for::<CompFinished>(|c| c.id);
        self.sim.register_key_getter_for::<CompStarted>(|c| c.id);
        self.sim.register_key_getter_for::<AllocationSuccess>(|c| c.id);
        self.sim.register_key_getter_for::<DeallocationSuccess>(|c| c.id);
    }
}
