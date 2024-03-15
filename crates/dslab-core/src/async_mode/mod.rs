//! Asynchronous programming support.

use crate::async_mode_enabled;
pub(crate) mod build_macro_rules;

async_mode_enabled!(
    pub mod await_details;
    pub mod sync;

    pub(crate) mod event_future;
    pub(crate) mod executor;
    pub(crate) mod promise_storage;
    pub(crate) mod task;
    pub(crate) mod timer_future;
    pub(crate) mod waker;

    pub use await_details::EventKey;
    pub use await_details::AwaitResult;
);
