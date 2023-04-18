use std::{cell::RefCell, collections::HashMap, rc::Rc};

use dslab_core::{cast, event::EventData, Event, EventHandler, Id, SimulationContext};
use dslab_network::network::Network;
use futures::{stream::FuturesUnordered, StreamExt};
use serde::Serialize;
use sugars::{rc, refcell};

#[derive(Serialize)]
pub struct StartMessage {}

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
    peers: RefCell<Vec<Id>>,
    is_pinger: bool,
    rand_delay: bool,
    iterations: u32,
    ctx: SimulationContext,
}

impl Process {
    pub fn new(peers: Vec<Id>, is_pinger: bool, rand_delay: bool, iterations: u32, ctx: SimulationContext) -> Self {
        let peer_count = peers.len();
        Self {
            peer_count,
            peers: RefCell::new(peers),
            is_pinger,
            rand_delay,
            iterations,
            ctx,
        }
    }

    fn on_start(&self) {
        self.ctx.spawn(self.start_process());
    }

    async fn start_process(&self) {
        if !self.is_pinger {
            return;
        }

        for i in 0..=self.iterations {
            let peer = self.peers.borrow()[self.ctx.gen_range(0..self.peer_count)];
            self.send(
                PingMessage {
                    payload: self.ctx.time(),
                },
                peer,
            );
            self.ctx.async_handle_event::<PongMessage>(peer, self.ctx.id()).await;
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

    fn send<T: EventData>(&self, event: T, to: Id) {
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
            StartMessage {} => {
                self.on_start();
            }
        })
    }
}

#[derive(Clone)]
pub struct NetworkProcess {
    id: Id,
    peer_count: usize,
    peers: RefCell<Vec<Id>>,
    is_pinger: bool,
    iterations: u32,
    net: Rc<RefCell<Network>>,
    ctx: SimulationContext,
}

impl NetworkProcess {
    pub fn new(
        peers: Vec<Id>,
        is_pinger: bool,
        iterations: u32,
        net: Rc<RefCell<Network>>,
        ctx: SimulationContext,
    ) -> Self {
        let peer_count = peers.len();
        Self {
            id: ctx.id(),
            peer_count,
            peers: RefCell::new(peers),
            is_pinger,
            iterations,
            net,
            ctx,
        }
    }

    fn on_start(&self) {
        self.ctx.spawn(self.start_network_process());
    }

    pub async fn start_network_process(&self) {
        if !self.is_pinger {
            return;
        }

        for i in 0..=self.iterations {
            let peer = self.peers.borrow()[self.ctx.gen_range(0..self.peer_count)];
            self.net.borrow_mut().send_event(
                PingMessage {
                    payload: self.ctx.time(),
                },
                self.id,
                peer,
            );

            self.ctx.async_handle_event::<PongMessage>(peer, self.id).await;
        }
    }

    fn on_ping(&self, from: Id) {
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
            StartMessage {} => {
                self.on_start();
            }
        })
    }
}
