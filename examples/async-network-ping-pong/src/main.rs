use std::{cell::RefCell, rc::Rc};

use std::collections::BTreeSet;
use std::io::Write;
use std::time::Instant;

use clap::Parser;
use dslab_core::{Id, Simulation, SimulationContext};
use dslab_network::topology_model::TopologyNetwork;
use dslab_network::{constant_bandwidth_model::ConstantBandwidthNetwork, network::Network};
use env_logger::Builder;
use process::Process;
use sugars::{rc, refcell};

mod process;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Number of processes (>= 2)
    #[clap(long, default_value_t = 5)]
    proc_count: u32,
}

fn main() {
    let args = Args::parse();
    let proc_count = args.proc_count;

    Builder::from_default_env()
        .format(|buf, record| writeln!(buf, "{}", record.args()))
        .init();
    // env_logger::init();

    let mut simulation = Simulation::new(42);

    let network_model = rc!(refcell!(ConstantBandwidthNetwork::new(1000., 0.001)));
    let network = rc!(refcell!(Network::new(network_model, simulation.create_context("net"))));
    simulation.add_handler("net", network.clone());

    let network_nodes = vec!["host1", "host2", "host3"];
    let bandwidth = vec![200., 300., 500.];
    let latency = vec![0., 0., 0.];

    for i in 0..network_nodes.len() {
        network
            .borrow_mut()
            .add_node(network_nodes[i], bandwidth[i], latency[i]);
    }

    let mut ctxs = Vec::new();

    let mut network_node_id = 0;

    for i in 1..=proc_count {
        let proc_name = format!("proc{}", i);

        let ctx = simulation.create_context(proc_name);

        network
            .borrow_mut()
            .set_location(ctx.id(), network_nodes[network_node_id]);

        ctxs.push(ctx);

        network_node_id = (network_node_id + 1) % network_nodes.len();
    }

    let peers: Vec<Id> = ctxs.iter().map(|ctx| ctx.id()).collect();

    for ctx in ctxs {
        let proc = Process::new(peers.clone(), network.clone(), ctx);
        simulation.spawn(process::start_pinger_vm(proc));
    }

    simulation.step_until_no_events();
}
