use std::{cell::RefCell, rc::Rc};

use dslab_compute::multicore::Compute;
use dslab_core::{cast, Event, EventHandler, SimulationContext};
use log::debug;

use crate::events::Start;

pub struct Worker {
    compute: Rc<RefCell<Compute>>,
    ctx: SimulationContext,
}

impl Worker {
    pub fn new(compute: Rc<RefCell<Compute>>, ctx: SimulationContext) -> Self {
        Self { compute, ctx }
    }

    fn on_start(&self) {
        debug!("Worker started");
    }
}

impl EventHandler for Worker {
    fn on(&mut self, event: Event) {
        cast!(match event.data {
            Start {} => {
                self.on_start()
            }
        })
    }
}
