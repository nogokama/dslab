use crate::event::EventData;
use crate::{Event, Id};
use serde::Serialize;

use std::any::{Any, TypeId};
use std::rc::Rc;
use std::{cell::RefCell, future::Future, sync::Arc, task::Context};
use std::{
    pin::Pin,
    task::{Poll, Waker},
};

use super::timer::Timer;

#[derive(Serialize)]
pub struct EmptyData {}

pub enum AwaitResult<T: EventData> {
    Timeout(Event),
    Ok((Event, T)),
}

impl<T: EventData> Default for AwaitResult<T> {
    fn default() -> Self {
        Self::Timeout(Event {
            id: 0,
            time: 0.,
            src: 0,
            dest: 0,
            data: Box::new(EmptyData {}),
        })
    }
}

impl<T: EventData> AwaitResult<T> {
    pub fn timeout_with(src: Id, dest: Id) -> Self {
        Self::Timeout(Event {
            id: 0,
            time: 0.,
            src,
            dest,
            data: Box::new(EmptyData {}),
        })
    }
}

pub struct SharedState<T: EventData> {
    /// Whether or not the sleep time has elapsed
    pub completed: bool,

    pub waker: Option<Waker>,

    pub shared_content: AwaitResult<T>,
}

impl<T: EventData> Default for SharedState<T> {
    fn default() -> Self {
        Self {
            completed: false,
            waker: None,
            shared_content: AwaitResult::<T>::default(),
        }
    }
}

pub trait EventSetter: Any {
    fn set_ok_completed_with_event(&mut self, e: Event);
    fn set_completed(&mut self);
}

impl<T: EventData> EventSetter for SharedState<T> {
    fn set_ok_completed_with_event(&mut self, mut e: Event) {
        if self.completed {
            return;
        }

        let downcast_result = e.data.downcast::<T>();

        e.data = Box::new(EmptyData {});
        match downcast_result {
            Ok(data) => {
                self.shared_content = AwaitResult::Ok((e, *data));
                self.set_completed();
            }
            Err(_) => {
                panic!("internal downcast conversion error");
            }
        };
    }

    fn set_completed(&mut self) {
        if self.completed {
            return;
        }
        self.completed = true;
        if let Some(waker) = self.waker.take() {
            waker.wake()
        }
    }
}

pub struct EventFuture<T: EventData> {
    pub state: Rc<RefCell<SharedState<T>>>,
}

impl<T: EventData> Future for EventFuture<T> {
    type Output = AwaitResult<T>;
    fn poll(self: Pin<&mut Self>, _cx: &mut Context) -> Poll<Self::Output> {
        // println!("Polling EventFuture...{}", self.state.borrow().completed);
        let mut state = self.state.as_ref().borrow_mut();

        if !state.completed {
            state.waker = Some(_cx.waker().clone());
            return Poll::Pending;
        }

        let mut filler = AwaitResult::default();
        std::mem::swap(&mut filler, &mut state.shared_content);

        return Poll::Ready(filler);
    }
}

pub struct TimerFuture {
    pub state: Rc<RefCell<SharedState<EmptyData>>>,
}

impl Future for TimerFuture {
    type Output = ();
    fn poll(self: Pin<&mut Self>, _cx: &mut Context) -> Poll<Self::Output> {
        // println!("Polling EventFuture...{}", self.state.borrow().completed);
        let mut state = self.state.as_ref().borrow_mut();

        if !state.completed {
            state.waker = Some(_cx.waker().clone());
            return Poll::Pending;
        }

        return Poll::Ready(());
    }
}

/// ```rust
///
///  pub struct AwaitKey {
///     from : Id,
///     to: Id,
///     msg_type: TypeId,
///     msg_details_hash: u64,
///  }
///
///  let simulation = Simulation::new(42);
///
///  let ctx = simulation.create_context("node");
///
///  ctx.async_handle_event::<LibraryNetworkMessage>(CustomMsgDetails::LibraryNetwork(NetworkDetail{from: "somebody"}));
/// ```
///
///``` rust
/// pub type EventDetails = u64
///
/// pub trait EventDetailedData: EventData {
///     fn get_details_key(&self) -> u64
/// }
///
/// struct InternalDataTransfer {
///    src: u64,
///
///    dst: u64,
///    internal_type: String,
///}
/// impl Data {
///   fn get_details_key(&self) -> u64 {
///        static mut MAPPING: Option<HashMap<(u64, String), u64>> = None;
///        static mut NEXT_KEY: u64 = 0;
///
///        unsafe {
///            MAPPING
///                .get_or_insert_with(HashMap::new)
///                .entry((self.dst, self.internal_type.clone()))
///                .or_insert_with(|| {
///                    let key = NEXT_KEY;
///                    NEXT_KEY += 1;
///                    key
///                })
///               .clone()
///        }
///    }
/// }
///```
/// --------------------------------------
/// ```rust
/// #[derive(EventDetailedData)]
/// pub struct Data {
///     src: Id,
///     #[EventDetailsKey]
///     dst: Id,
///     #[EventDetailsKey]
///     internal_type: Type,
///     content: String,
/// }
///```
///
#[derive(Hash, PartialEq, Eq, Debug, Clone)]
pub struct AwaitKey {
    pub from: Id,
    pub to: Id,
    pub msg_type: TypeId,
}

impl AwaitKey {
    pub fn new<T: EventData>(from: Id, to: Id) -> Self {
        Self {
            from,
            to,
            msg_type: TypeId::of::<T>(),
        }
    }
}
