use std::rc::Rc;

use async_trait::async_trait;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::host::process::HostProcessInstance;

#[async_trait(?Send)]
pub trait ExecutionProfile {
    async fn run(self: Rc<Self>, processes: &Vec<HostProcessInstance>);

    fn get_name() -> String;

    fn default() -> Self;
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum ProfileDefinition {
    Simple(String),
    Detailed { r#type: String, args: serde_json::Value },
}
