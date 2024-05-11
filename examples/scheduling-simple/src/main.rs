mod profiles;
mod round_robin;
// mod tetris;

use dslab_core::{cast, EventHandler, Id, Simulation, SimulationContext};
use dslab_scheduling::{
    cluster::{ExecutionFinished, ScheduleExecution},
    config::sim_config::SimulationConfig,
    scheduler::{CustomScheduler, Resources},
    simulation::ClusterSchedulingSimulation,
    workload_generators::random::RandomWorkloadGenerator,
};
use env_logger::Builder;
use profiles::TestProfile;
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

    // let config = SimulationConfig::from_file("configs/config.yaml");
    // let config = SimulationConfig::from_file("configs/config_with_native.yaml");
    // let config = SimulationConfig::from_file("configs/config_with_combinators.yaml");
    // let config = SimulationConfig::from_file("configs/config_with_custom_profiles.yaml");
    // let config = SimulationConfig::from_file("configs/google_config.yaml");
    // let config = SimulationConfig::from_file("configs/google_converted.yaml");

    let mut cluster_sim = ClusterSchedulingSimulation::new(sim, config, None);

    cluster_sim.register_profile::<TestProfile>("test-profile");

    // cluster_sim.run(TetrisScheduler::new(cluster_id, scheduler_context));
    // cluster_sim.run_with_custom_scheduler(RoundRobinScheduler::new(cluster_id, scheduler_context));
    cluster_sim.run_with_scheduler(RoundRobinScheduler::new());
}
