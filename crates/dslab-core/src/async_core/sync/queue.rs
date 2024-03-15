//! Queue implementation.

use std::{cell::RefCell, collections::VecDeque};

use serde::Serialize;

use crate::{async_core::await_details::DetailsKey, event::EventData, SimulationContext};

type TicketID = u64;

#[derive(Serialize, Clone)]
struct Notify {
    ticket_id: TicketID,
}

fn get_notify_details(data: &dyn EventData) -> DetailsKey {
    let notify = data.downcast_ref::<Notify>().unwrap();
    notify.ticket_id as DetailsKey
}

/// MPMC Unbounded queue with blocking receives for any type of data.
///
/// Data is guarantied to be delivered in order that receivers call their .receive() method.
///
/// Each Future got by .receive() must be awaited.
pub struct UnboundedBlockingQueue<T> {
    ctx: SimulationContext,
    queue: RefCell<VecDeque<T>>,
    send_ticket: Ticket,
    receive_ticket: Ticket,
}

impl<T> UnboundedBlockingQueue<T> {
    pub(crate) fn new(ctx: SimulationContext) -> Self {
        ctx.register_details_getter_for::<Notify>(get_notify_details);
        Self {
            ctx,
            queue: RefCell::new(VecDeque::new()),
            send_ticket: Ticket::new(),
            receive_ticket: Ticket::new(),
        }
    }

    /// Sends data to the queue without blocking.
    pub fn send(&self, data: T) {
        self.send_ticket.next();
        self.queue.borrow_mut().push_back(data);
        if self.receive_ticket.is_after(&self.send_ticket) {
            self.ctx.emit_self_now(Notify {
                ticket_id: self.send_ticket.value(),
            });
        }
    }

    /// Async receives data from queue. Each receive must be awaited.
    pub async fn receive(&self) -> T {
        self.receive_ticket.next();
        if self.queue.borrow().is_empty() {
            self.ctx
                .async_detailed_handle_self::<Notify>(self.receive_ticket.value())
                .await;
        }

        self.queue.borrow_mut().pop_front().unwrap()
    }
}

struct Ticket {
    value: RefCell<TicketID>,
}

impl Ticket {
    fn new() -> Self {
        Self { value: RefCell::new(0) }
    }

    fn next(&self) {
        *self.value.borrow_mut() += 1;
    }

    fn is_after(&self, other: &Self) -> bool {
        *self.value.borrow() >= *other.value.borrow()
    }

    fn value(&self) -> TicketID {
        *self.value.borrow()
    }
}
