mod events;
mod process;

use std::cell::RefCell;
use std::io::Write;
use std::rc::Rc;
use std::time::Instant;

use clap::Parser;
use env_logger::Builder;
use process::Worker;
use rand::prelude::*;
use rand_pcg::Pcg64;
use sugars::{rc, refcell};

use dslab_compute::multicore::{Compute, CoresDependency};
use dslab_core::simulation::Simulation;

fn main() {
    let seed = 42;
    let mut sim = Simulation::new(seed);
    let mut rand = Pcg64::seed_from_u64(seed);
    // admin context for starting master and workers
    let mut admin = sim.create_context("admin");
    // client context for submitting tasks
    let mut client = sim.create_context("client");

    let host = "host";

    let compute_name = format!("{}::compute", host);
    let worker_name = format!("{}:worker", host);

    let compute = rc!(refcell!(Compute::new(
        rand.gen_range(1..=10),
        rand.gen_range(1..=8),
        rand.gen_range(1..=4) * 1024,
        sim.create_context(&compute_name),
    )));

    sim.add_handler(compute_name, compute.clone());

    let worker = rc!(refcell!(Worker::new(compute, sim.create_context(&worker_name))));

    sim.add_handler(worker_name, worker);
}
