use std::rc::Rc;

use async_trait::async_trait;
use serde::de::DeserializeOwned;

use crate::host::process::HostProcessInstance;

pub mod default;

#[async_trait(?Send)]
pub trait ExecutionProfile {
    async fn run(self: Rc<Self>, processes: &Vec<HostProcessInstance>);

    fn get_name(&self) -> String;
}

pub fn from_json<T>(json: &serde_json::Value) -> Rc<dyn ExecutionProfile>
where
    T: DeserializeOwned + ExecutionProfile + 'static,
{
    let task: T = serde_json::from_value(json.clone()).unwrap();
    Rc::new(task)
}
