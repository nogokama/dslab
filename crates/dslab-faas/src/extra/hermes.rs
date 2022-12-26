use std::cell::RefCell;
use std::rc::Rc;

use crate::function::Application;
use crate::host::Host;
use crate::scheduler::LeastLoadedScheduler;
use crate::scheduler::Scheduler;

/// Refer to https://arxiv.org/abs/2111.07226
pub struct HermesScheduler {
    high_load_fallback: LeastLoadedScheduler,
}

impl Default for HermesScheduler {
    fn default() -> Self {
        Self {
            high_load_fallback: LeastLoadedScheduler::new(true),
        }
    }
}

impl HermesScheduler {
    pub fn new() -> Self {
        Default::default()
    }
}

impl Scheduler for HermesScheduler {
    fn select_host(&mut self, app: &Application, hosts: &[Rc<RefCell<Host>>]) -> usize {
        let mut ans = 0;
        // 0 -> empty, no warm container
        // 1 -> empty, warm container
        // 2 -> non-empty, no warm container
        // 3 -> non-empty, warm container
        let mut priority = -1;
        for (i, host) in hosts.iter().enumerate() {
            let h = host.borrow();
            if h.get_cpu_load() < (h.get_cpu_cores() as f64) {
                let curr_priority;
                if h.get_active_invocations() > 0 {
                    if h.can_invoke(app, false) {
                        curr_priority = 3;
                    } else {
                        curr_priority = 2;
                    }
                } else if h.can_invoke(app, false) {
                    curr_priority = 1;
                } else {
                    curr_priority = 0;
                }
                if curr_priority > priority {
                    priority = curr_priority;
                    ans = i;
                }
            }
        }
        if priority != -1 {
            return ans;
        }
        self.high_load_fallback.select_host(app, hosts)
    }

    fn to_string(&self) -> String {
        "HermesScheduler".to_string()
    }
}
