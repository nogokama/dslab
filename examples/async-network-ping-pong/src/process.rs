use std::{cell::RefCell, collections::HashMap, rc::Rc};

use dslab_core::{cast, Event, EventHandler, Id, SimulationContext};
use dslab_network::network::Network;
use futures::{stream::FuturesUnordered, StreamExt};
use serde::Serialize;
use sugars::{rc, refcell};

#[derive(Serialize)]
pub struct PingMessage {
    attempt: u32,
}

#[derive(Serialize)]
pub struct PongMessage {
    attempt: u32,
}

#[derive(Clone)]
pub struct Process {
    id: Id,
    peer_count: usize,
    peers: Rc<RefCell<Vec<Id>>>,
    delays: Rc<RefCell<HashMap<Id, f64>>>,
    net: Rc<RefCell<Network>>,
    ctx: SimulationContext,
}

impl Process {
    pub fn new(peers: Vec<Id>, net: Rc<RefCell<Network>>, ctx: SimulationContext) -> Self {
        Self {
            id: ctx.id(),
            peer_count: peers.len(),
            peers: rc!(refcell!(peers)),
            delays: rc!(refcell!(HashMap::new())),
            net,
            ctx,
        }
    }
}

pub async fn start_pinger_vm(process: Process) {
    let mut futures = FuturesUnordered::new();
    for id in process.peers.borrow().iter() {
        futures.push(get_latency_for(process.clone(), *id));
    }

    for id in process.peers.borrow().iter() {
        process.ctx.spawn(response_on_pings(process.clone(), *id));
    }

    while let Some((id, result)) = futures.next().await {
        process.delays.borrow_mut().insert(id, result);
    }

    println!("process {} finished", process.id);
}

async fn get_latency_for(mut process: Process, target: Id) -> (Id, f64) {
    process
        .net
        .borrow_mut()
        .send_event(PingMessage { attempt: 0 }, process.id, target);

    let result = process
        .ctx
        .async_wait_for_event::<PongMessage>(target, process.id, 100500.)
        .await;

    println!(
        "node {}, awaited {} at time: {}",
        process.id,
        target,
        process.ctx.time()
    );

    (target, 100500.)
}

async fn response_on_pings(mut process: Process, target: Id) {
    loop {
        let (event, ping) = process.ctx.async_handle_event::<PingMessage>(target, process.id).await;
        process
            .net
            .borrow_mut()
            .send_event(PongMessage { attempt: ping.attempt }, process.id, event.src);
    }
}
