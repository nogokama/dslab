mod round_robin;

use dslab_core::{cast, EventHandler, Id, Simulation, SimulationContext};
use dslab_scheduling::{
    cluster::{Schedule, TaskFinished},
    event_generator::{HostAdded, RandomEventGenerator, TaskInfo},
    machine::Machine,
    scheduler::{Resources, Scheduler},
    simulation::ClusterSchedulingSimulation,
};
use env_logger::Builder;
use round_robin::RoundRobinScheduler;
use serde::Serialize;
use std::{
    collections::{HashMap, VecDeque},
    io::Write,
};
use sugars::{rc, refcell};

fn main() {
    Builder::from_default_env()
        .format(|buf, record| writeln!(buf, "{}", record.args()))
        .init();

    let mut sim = Simulation::new(42);

    let generator = RandomEventGenerator::new(sim.create_context("generator"), 20000);

    let scheduler_context = sim.create_context("scheduler");

    let mut cluster_sim = ClusterSchedulingSimulation::new(sim, generator);

    let cluster_id = cluster_sim.get_cluster_id();
    cluster_sim.run(RoundRobinScheduler::new(cluster_id, scheduler_context));
}
