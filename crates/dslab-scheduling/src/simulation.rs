use std::{cell::RefCell, rc::Rc};

use dslab_compute::multicore::Compute;
use dslab_core::{event, EventHandler, Id, Simulation};
use sugars::{rc, refcell};

use crate::{
    cluster::Cluster,
    event_generator::EventGenerator,
    proxy::Proxy,
    scheduler::{self, Scheduler},
    storage::TaskInfoStorage,
};

pub struct ClusterSchedulingSimulation<G: EventGenerator> {
    sim: Simulation,
    generator: G,

    cluster: Rc<RefCell<Cluster>>,
    proxy: Rc<RefCell<Proxy>>,
}

impl<G: EventGenerator> ClusterSchedulingSimulation<G> {
    pub fn new(mut sim: Simulation, generator: G) -> ClusterSchedulingSimulation<G> {
        let task_storage = rc!(refcell!(TaskInfoStorage::new()));
        let cluster_ctx = sim.create_context("cluster");
        let cluster = rc!(refcell!(Cluster::new(cluster_ctx, task_storage.clone())));
        sim.add_handler("cluster", cluster.clone());

        let proxy_ctx = sim.create_context("proxy");
        let proxy = rc!(refcell!(Proxy::new(proxy_ctx, task_storage.clone())));
        sim.add_handler("proxy", proxy.clone());

        ClusterSchedulingSimulation {
            sim,
            generator,
            cluster,
            proxy,
        }
    }

    pub fn get_cluster_id(&self) -> Id {
        self.cluster.borrow().get_id()
    }

    pub fn run<T: EventHandler + Scheduler + 'static>(&mut self, scheduler: T) {
        let scheduler_id = scheduler.id();
        let name = scheduler.name().clone();
        self.sim.add_handler(name, rc!(refcell!(scheduler)));

        self.cluster.borrow_mut().set_scheduler(scheduler_id);
        self.proxy.borrow_mut().set_scheduler(scheduler_id);

        let proxy_id = self.proxy.borrow().get_id();
        let machines_events = self.generator.schedule_events(proxy_id);

        for event in machines_events {
            let compute_name = format!("machine-{}", event.machine.id);
            let ctx = self.sim.create_context(&compute_name);
            let compute_id = ctx.id();
            let compute = rc!(refcell!(Compute::new(
                1.,
                event.machine.cpu_cores,
                event.machine.memory,
                ctx
            )));

            self.sim.add_handler(&compute_name, compute.clone());
            self.cluster
                .borrow_mut()
                .add_compute(event.machine.id, compute_id, compute);
        }
        // let events = self.sim.dump_events();

        // TODO for long simulation make a while loop
        self.sim.step_until_no_events();
    }
}
