mod round_robin;
// mod tetris;

use dslab_core::{cast, EventHandler, Id, Simulation, SimulationContext};
use dslab_scheduling::{
    cluster::{JobFinished, Schedule},
    config::sim_config::SimulationConfig,
    scheduler::{Resources, Scheduler},
    simulation::ClusterSchedulingSimulation,
    workload_generators::random::RandomWorkloadGenerator,
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

    let scheduler_context = sim.create_context("scheduler");

    let config = SimulationConfig::from_file("config.yaml");

    let mut cluster_sim = ClusterSchedulingSimulation::new(sim, config, None);

    let cluster_id = cluster_sim.get_cluster_id();
    // cluster_sim.run(TetrisScheduler::new(cluster_id, scheduler_context));
    cluster_sim.run_with_custom_scheduler(RoundRobinScheduler::new(cluster_id, scheduler_context));
}
