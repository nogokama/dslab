//! Timers for simulation.

use std::{cell::RefCell, cmp::Ordering, rc::Rc};

use super::shared_state::AwaitResultSetter;
use crate::Id;

/// Timer identifier.
pub type TimerId = u64;

/// Timer will set the given `state` as completed at time.
#[derive(Clone)]
pub struct Timer {
    /// Unique identifier of timer.
    pub id: TimerId,
    /// Id of simulation component that set the timer.
    pub component_id: Id,
    /// The time when the timer will be fired.
    pub time: f64,
    /// State to set completed after the timer is fired.
    pub(crate) state: Rc<RefCell<dyn AwaitResultSetter>>,
}

impl Timer {
    /// Creates a timer.
    pub(crate) fn new(id: TimerId, component_id: Id, time: f64, state: Rc<RefCell<dyn AwaitResultSetter>>) -> Self {
        Self {
            id,
            component_id,
            time,
            state,
        }
    }
}

impl PartialEq for Timer {
    fn eq(&self, other: &Self) -> bool {
        self.time == other.time
    }
}

impl Eq for Timer {}

impl Ord for Timer {
    fn cmp(&self, other: &Self) -> Ordering {
        other.time.total_cmp(&self.time).then_with(|| other.id.cmp(&self.id))
    }
}

impl PartialOrd for Timer {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
