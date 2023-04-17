use std::{cell::RefCell, collections::HashMap, rc::Rc};

use dslab_core::{cast, event::EventData, Event, EventHandler, Id, SimulationContext};
use dslab_network::network::Network;
use futures::{stream::FuturesUnordered, StreamExt};
use serde::Serialize;
use sugars::{rc, refcell};

#[derive(Serialize)]
pub struct PingMessage {
    payload: f64,
}

#[derive(Serialize)]
pub struct PongMessage {
    payload: f64,
}

#[derive(Clone)]
pub struct Process {
    peer_count: usize,
    peers: Rc<RefCell<Vec<Id>>>,
    is_pinger: bool,
    rand_delay: bool,
    iterations: u32,
    ctx: SimulationContext,
}

impl Process {
    pub fn new(
        peers: Rc<RefCell<Vec<Id>>>,
        is_pinger: bool,
        rand_delay: bool,
        iterations: u32,
        ctx: SimulationContext,
    ) -> Self {
        let peer_count = peers.borrow().len();
        Self {
            peer_count,
            peers,
            is_pinger,
            rand_delay,
            iterations,
            ctx,
        }
    }

    fn on_ping(&mut self, from: Id) {
        self.send(
            PongMessage {
                payload: self.ctx.time(),
            },
            from,
        );
    }

    fn send<T: EventData>(&mut self, event: T, to: Id) {
        let delay = if self.rand_delay { self.ctx.rand() } else { 1. };
        self.ctx.emit(event, to, delay);
    }
}

impl EventHandler for Process {
    fn on(&mut self, event: Event) {
        cast!(match event.data {
            PingMessage { payload: _ } => {
                self.on_ping(event.src);
            }
        })
    }
}

pub async fn start_process(mut process: Process) {
    if !process.is_pinger {
        return;
    }

    for i in 0..=process.iterations {
        let peer = process.peers.borrow()[process.ctx.gen_range(0..process.peer_count)];
        process.send(
            PingMessage {
                payload: process.ctx.time(),
            },
            peer,
        );
        process
            .ctx
            .async_handle_event::<PongMessage>(peer, process.ctx.id())
            .await;
    }
}

#[derive(Clone)]
pub struct NetworkProcess {
    id: Id,
    peer_count: usize,
    peers: Rc<RefCell<Vec<Id>>>,
    is_pinger: bool,
    iterations: u32,
    net: Rc<RefCell<Network>>,
    ctx: SimulationContext,
}

impl NetworkProcess {
    pub fn new(
        peers: Rc<RefCell<Vec<Id>>>,
        is_pinger: bool,
        iterations: u32,
        net: Rc<RefCell<Network>>,
        ctx: SimulationContext,
    ) -> Self {
        let peer_count = peers.borrow().len();
        Self {
            id: ctx.id(),
            peer_count,
            peers,
            is_pinger,
            iterations,
            net,
            ctx,
        }
    }

    fn on_ping(&mut self, from: Id) {
        self.net.borrow_mut().send_event(
            PongMessage {
                payload: self.ctx.time(),
            },
            self.id,
            from,
        )
    }
}

impl EventHandler for NetworkProcess {
    fn on(&mut self, event: Event) {
        cast!(match event.data {
            PingMessage { payload: _ } => {
                self.on_ping(event.src);
            }
        })
    }
}

pub async fn start_network_process(mut process: NetworkProcess) {
    if !process.is_pinger {
        return;
    }

    for i in 0..=process.iterations {
        let peer = process.peers.borrow()[process.ctx.gen_range(0..process.peer_count)];
        process.net.borrow_mut().send_event(
            PingMessage {
                payload: process.ctx.time(),
            },
            process.id,
            peer,
        );

        process.ctx.async_handle_event::<PongMessage>(peer, process.id).await;
    }
}
