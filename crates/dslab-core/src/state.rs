use std::any::TypeId;
use std::cell::RefCell;
use std::collections::{BinaryHeap, HashMap, HashSet, VecDeque};
use std::rc::Rc;
use std::sync::mpsc::Sender;
use std::sync::Arc;

use futures::stream::FuturesUnordered;
use futures::Future;
use rand::distributions::uniform::{SampleRange, SampleUniform};
use rand::distributions::{Alphanumeric, DistString};
use rand::prelude::*;
use rand_pcg::Pcg64;

use crate::async_core::shared_state::{AwaitKey, DetailsKey, EmptyData, EventSetter, SharedState, TimerFuture};
use crate::async_core::task::Task;
use crate::async_core::timer::Timer;
use crate::component::Id;
use crate::event::{Event, EventData, EventId};
use crate::log::log_incorrect_event;

pub struct SimulationState {
    clock: f64,
    rand: Pcg64,
    events: BinaryHeap<Event>,
    ordered_events: VecDeque<Event>,
    canceled_events: HashSet<EventId>,
    event_count: u64,

    awaiters: HashMap<AwaitKey, Rc<RefCell<dyn EventSetter>>>,
    details_getters: HashMap<TypeId, fn(&dyn EventData) -> DetailsKey>,

    timers: BinaryHeap<Timer>,

    task_sender: Sender<Arc<Task>>,
}

impl SimulationState {
    pub fn new(seed: u64, task_sender: Sender<Arc<Task>>) -> Self {
        Self {
            clock: 0.0,
            rand: Pcg64::seed_from_u64(seed),
            events: BinaryHeap::new(),
            ordered_events: VecDeque::new(),
            canceled_events: HashSet::new(),
            event_count: 0,
            awaiters: HashMap::new(),
            details_getters: HashMap::new(),
            timers: BinaryHeap::new(),

            task_sender,
        }
    }

    pub fn time(&self) -> f64 {
        self.clock
    }

    pub fn set_time(&mut self, time: f64) {
        self.clock = time;
    }

    pub fn rand(&mut self) -> f64 {
        self.rand.gen_range(0.0..1.0)
    }

    pub fn gen_range<T, R>(&mut self, range: R) -> T
    where
        T: SampleUniform,
        R: SampleRange<T>,
    {
        self.rand.gen_range(range)
    }

    pub fn sample_from_distribution<T, Dist: Distribution<T>>(&mut self, dist: &Dist) -> T {
        dist.sample(&mut self.rand)
    }

    pub fn random_string(&mut self, len: usize) -> String {
        Alphanumeric.sample_string(&mut self.rand, len)
    }

    pub fn add_event<T>(&mut self, data: T, src: Id, dest: Id, delay: f64) -> EventId
    where
        T: EventData,
    {
        let event_id = self.event_count;
        let event = Event {
            id: event_id,
            time: self.clock + delay,
            src,
            dest,
            data: Box::new(data),
        };
        if delay >= 0. {
            self.events.push(event);
            self.event_count += 1;
            event_id
        } else {
            log_incorrect_event(event, &format!("negative delay {}", delay));
            panic!("Event delay is negative! It is not allowed to add events from the past.");
        }
    }

    pub fn add_ordered_event<T>(&mut self, data: T, src: Id, dest: Id, delay: f64) -> EventId
    where
        T: EventData,
    {
        if !self.can_add_ordered_event(delay) {
            panic!("Event order is broken! Ordered events should be added in non-decreasing order of their time.");
        }
        let last_time = self.ordered_events.back().map_or(f64::MIN, |x| x.time);
        let event_id = self.event_count;
        let event = Event {
            id: event_id,
            // max is used to enforce time order despite of floating-point errors
            time: last_time.max(self.clock + delay),
            src,
            dest,
            data: Box::new(data),
        };
        if delay >= 0. {
            self.ordered_events.push_back(event);
            self.event_count += 1;
            event_id
        } else {
            log_incorrect_event(event, &format!("negative delay {}", delay));
            panic!("Event delay is negative! It is not allowed to add events from the past.");
        }
    }

    pub fn can_add_ordered_event(&self, delay: f64) -> bool {
        if let Some(evt) = self.ordered_events.back() {
            // small epsilon is used to account for floating-point errors
            if delay + self.clock < evt.time - 1e-12 {
                return false;
            }
        }
        true
    }

    pub fn next_event(&mut self) -> Option<Event> {
        loop {
            let maybe_heap = self.events.peek();
            let maybe_deque = self.ordered_events.front();
            if maybe_heap.is_some() && (maybe_deque.is_none() || maybe_heap.unwrap() > maybe_deque.unwrap()) {
                let event = self.events.pop().unwrap();
                if !self.canceled_events.remove(&event.id) {
                    self.clock = event.time;
                    return Some(event);
                }
            } else if maybe_deque.is_some() {
                let event = self.ordered_events.pop_front().unwrap();
                if !self.canceled_events.remove(&event.id) {
                    self.clock = event.time;
                    return Some(event);
                }
            } else {
                return None;
            }
        }
    }

    pub fn peek_event(&self) -> Option<&Event> {
        let maybe_heap = self.events.peek();
        let maybe_deque = self.ordered_events.front();
        if maybe_heap.is_some() && (maybe_deque.is_none() || maybe_heap.unwrap() > maybe_deque.unwrap()) {
            maybe_heap
        } else {
            maybe_deque
        }
    }

    pub fn cancel_event(&mut self, id: EventId) {
        self.canceled_events.insert(id);
    }

    pub fn cancel_events<F>(&mut self, pred: F)
    where
        F: Fn(&Event) -> bool,
    {
        for event in self.events.iter() {
            if pred(event) {
                self.canceled_events.insert(event.id);
            }
        }
        for event in self.ordered_events.iter() {
            if pred(event) {
                self.canceled_events.insert(event.id);
            }
        }
    }

    /// This function does not check events from `ordered_events`.
    pub fn cancel_heap_events<F>(&mut self, pred: F)
    where
        F: Fn(&Event) -> bool,
    {
        for event in self.events.iter() {
            if pred(event) {
                self.canceled_events.insert(event.id);
            }
        }
    }

    pub fn event_count(&self) -> u64 {
        self.event_count
    }

    pub fn peek_timer(&self) -> Option<&Timer> {
        self.timers.peek()
    }

    pub fn next_timer(&mut self) -> Option<Timer> {
        if let Some(timer) = self.timers.pop() {
            self.clock = timer.time;
            return Some(timer);
        }
        return None;
    }

    pub fn has_handler_on_key(&self, key: &AwaitKey) -> bool {
        self.awaiters.contains_key(key)
    }

    pub fn set_event_for_await_key(&mut self, key: &AwaitKey, event: Event) -> bool {
        if !self.awaiters.contains_key(key) {
            return false;
        }

        let shared_state = self.awaiters.remove(key).unwrap();

        shared_state.borrow_mut().set_ok_completed_with_event(event);

        return true;
    }

    pub fn spawn(&mut self, future: impl Future<Output = ()>) {
        let task = Arc::new(Task::new(future, self.task_sender.clone()));

        self.task_sender.send(task).expect("channel is closed");
    }

    pub fn spawn_static(&mut self, future: impl Future<Output = ()> + 'static) {
        let task = Arc::new(Task::new_static(future, self.task_sender.clone()));

        self.task_sender.send(task).expect("channel is closed");
    }

    pub fn wait_for(&mut self, timeout: f64) -> TimerFuture {
        let state = Rc::new(RefCell::new(SharedState::<EmptyData>::default()));

        self.timers.push(Timer::new(self.time() + timeout, state.clone()));

        TimerFuture { state }
    }

    pub fn add_timer_on_state(&mut self, timeout: f64, state: Rc<RefCell<dyn EventSetter>>) {
        self.timers.push(Timer::new(self.time() + timeout, state.clone()));
    }

    pub fn add_awaiter_handler(&mut self, key: AwaitKey, state: Rc<RefCell<dyn EventSetter>>) {
        self.awaiters.insert(key, state);
    }

    pub fn register_details_getter_for<T: EventData>(&mut self, details_getter: fn(&dyn EventData) -> DetailsKey) {
        self.details_getters.insert(TypeId::of::<T>(), details_getter);
    }
}
