use std::{collections::HashMap, rc::Rc};

use async_trait::async_trait;
use maplit::hashmap;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::workload_generators::native;

use super::{
    default::{CommunicationHomogenous, CpuBurnHomogenous},
    profile::ExecutionProfile,
};

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum ProfileDefinition {
    Simple(String),
    Detailed { r#type: String, args: serde_json::Value },
}

pub type ConstructorFn = Box<dyn Fn(&serde_json::Value) -> Rc<dyn ExecutionProfile>>;

pub struct ProfileBuilder {
    pub path: String,
    pub constructors: HashMap<String, ConstructorFn>,
}

impl ProfileBuilder {
    pub fn new(path: String) -> Self {
        Self {
            path,
            constructors: hashmap![
                CpuBurnHomogenous::get_name() => Box::new(|json| from_json::<CpuBurnHomogenous>(json)),
                CommunicationHomogenous::get_name() => Box::new(|json| from_json::<CommunicationHomogenous>(json)),
            ],
        }
    }

    pub fn parse_profiles(&mut self, json: &serde_json::Value) {
        json.as_object().unwrap().iter().for_each(|(name, value)| {
            let profile = match serde_json::from_value::<ProfileDefinition>(value.clone()) {
                Ok(profile) => profile,
                Err(e) => panic!("Can't parse profile {}: {}", name, e),
            };

            match profile {
                ProfileDefinition::Simple(profile_name) => {
                    let constructor = self.constructors.get(&profile_name).unwrap();
                    self.constructors
                        .insert(name.clone(), Box::new(move |json| constructor(json)));
                }
                ProfileDefinition::Detailed { r#type, args } => {
                    let constructor = self.constructors.get(&r#type).unwrap();
                    let new_constructor = Box::new(move |json| constructor(&args));
                    self.constructors.insert(name.clone(), new_constructor);
                }
            };
        });
    }

    pub fn build(&self, json: &serde_json::Value) -> Rc<dyn ExecutionProfile> {
        let profile = match serde_json::from_value::<ProfileDefinition>(json.clone()) {
            Ok(profile) => profile,
            Err(e) => panic!("Can't parse profile: {}", e),
        };

        match profile {
            ProfileDefinition::Simple(profile_name) => {
                let constructor = self.constructors.get(&profile_name).unwrap();
                constructor(json)
            }
            ProfileDefinition::Detailed { r#type, args } => {
                let constructor = self.constructors.get(&r#type).unwrap();
                constructor(&args)
            }
        }
    }
}

pub fn from_json<T>(json: &serde_json::Value) -> Rc<dyn ExecutionProfile>
where
    T: DeserializeOwned + ExecutionProfile + 'static,
{
    let task: T = serde_json::from_value(json.clone()).unwrap();
    Rc::new(task)
}
