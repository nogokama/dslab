use core::panic;
use std::{cell::RefCell, collections::HashMap, hash::Hash, rc::Rc};

use async_trait::async_trait;
use maplit::hashmap;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use sugars::{rc, refcell};

use crate::workload_generators::native;

use super::{
    combinators::{self, ParallelProfile, ProfileCombinator, SequentialProfile},
    default::{CommunicationHomogenous, CpuBurnHomogenous},
    profile::{ExecutionProfile, NameTrait},
};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum ProfileDefinition {
    Simple(String),
    Detailed { r#type: String, args: serde_json::Value },
}

pub type ConstructorFn = Rc<dyn Fn(&serde_json::Value) -> Rc<dyn ExecutionProfile>>;

pub struct ProfileBuilder {
    pub constructors: Rc<RefCell<HashMap<String, ConstructorFn>>>,
}

impl ProfileBuilder {
    pub fn new() -> Self {
        let constructors: Rc<RefCell<HashMap<String, ConstructorFn>>> = rc!(refcell!(HashMap::new()));
        constructors
            .borrow_mut()
            .insert(CpuBurnHomogenous::get_name(), Rc::new(from_json::<CpuBurnHomogenous>));
        constructors.borrow_mut().insert(
            CommunicationHomogenous::get_name(),
            Rc::new(from_json::<CommunicationHomogenous>),
        );

        let mut constructors_clone = constructors.clone();
        constructors.borrow_mut().insert(
            ParallelProfile::get_name(),
            Rc::new(move |json| parse_combinator::<ParallelProfile>(json, constructors_clone.clone())),
        );

        constructors_clone = constructors.clone();
        constructors.borrow_mut().insert(
            SequentialProfile::get_name(),
            Rc::new(move |json| parse_combinator::<SequentialProfile>(json, constructors_clone.clone())),
        );

        Self {
            constructors: constructors,
        }
    }

    pub fn parse_profiles(&mut self, json: &serde_json::Value) {
        json.as_object().unwrap().iter().for_each(|(name, value)| {
            println!("name: {}", &name);
            let profile = match serde_json::from_value::<ProfileDefinition>(value.clone()) {
                Ok(profile) => profile,
                Err(e) => panic!("Can't parse profile {}: {}", name, e),
            };

            match profile {
                ProfileDefinition::Simple(profile_name) => {
                    let constructor = self.constructors.borrow().get(&profile_name).cloned().unwrap();
                    self.constructors
                        .borrow_mut()
                        .insert(name.clone(), Rc::new(move |json| constructor(json)));
                }
                ProfileDefinition::Detailed { r#type, args } => {
                    let constructor = self.constructors.borrow().get(&r#type).cloned().unwrap();
                    self.constructors
                        .borrow_mut()
                        .insert(name.clone(), Rc::new(move |_| constructor(&args)));
                }
            };
        });
    }

    pub fn build(&self, profile: ProfileDefinition) -> Rc<dyn ExecutionProfile> {
        Self::build_raw(profile, &self.constructors)
    }

    fn build_raw(
        profile: ProfileDefinition,
        constructors: &Rc<RefCell<HashMap<String, ConstructorFn>>>,
    ) -> Rc<dyn ExecutionProfile> {
        match profile {
            ProfileDefinition::Simple(profile_name) => {
                let constructor = constructors
                    .borrow()
                    .get(&profile_name)
                    .cloned()
                    .unwrap_or_else(|| panic!("Profile {} not found in the constructor list", profile_name));
                constructor(&serde_json::Value::Null)
            }
            ProfileDefinition::Detailed { r#type, args } => {
                let constructor = constructors
                    .borrow()
                    .get(&r#type)
                    .cloned()
                    .unwrap_or_else(|| panic!("Profile {} not found in the constructor list", r#type));
                constructor(&args)
            }
        }
    }
}

pub fn from_json<T>(json: &serde_json::Value) -> Rc<dyn ExecutionProfile>
where
    T: DeserializeOwned + ExecutionProfile + 'static,
{
    let profile: T = serde_json::from_value(json.clone()).unwrap();
    Rc::new(profile)
}

#[derive(Serialize, Deserialize, Clone)]
struct CombinatorDefinition {
    pub repeat: Option<u32>,
    pub profiles: Vec<ProfileDefinition>,
}

pub fn parse_combinator<T: ProfileCombinator>(
    json: &serde_json::Value,
    constructors: Rc<RefCell<HashMap<String, ConstructorFn>>>,
) -> Rc<T> {
    let combinator: CombinatorDefinition = serde_json::from_value(json.clone()).unwrap();
    let profiles = combinator
        .profiles
        .iter()
        .map(|profile| ProfileBuilder::build_raw(profile.clone(), &constructors))
        .collect::<Vec<_>>();

    Rc::new(T::new(profiles, combinator.repeat))
}
