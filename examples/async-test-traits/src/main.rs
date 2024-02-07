use std::io::Write;
use std::{collections::HashMap, fs::File, io::Read};

use async_trait::async_trait;
use dslab_core::{log_info, simulation::Simulation, SimulationContext};

use env_logger::Builder;
use serde::Serialize;
use serde::{de::DeserializeOwned, Deserialize};
use serde_json::Value;

#[async_trait(?Send)]
pub trait JobProfile {
    async fn run(self: Box<Self>, ctx: SimulationContext);
}

// #[async_trait(?Send)]
// pub trait Job {
//     async fn run(self: Box<Self>, allocation: Vec<Allocation>);
// }

// #[async_trait(?Send)]
// pub trait Task {
//     async fn run(self: Box<Self>, job_context: JobContext);
//     fn assign_allocation();
// }

// user create tasks by themselves and define their behaviour
#[derive(Deserialize)]
pub struct TestJob {
    time: f64,
    id: u32,
    // and all other fields parsed from json
}

#[async_trait(?Send)]
impl JobProfile for TestJob {
    // all simulation-related is passed as arguments
    async fn run(self: Box<Self>, ctx: SimulationContext) {
        log_info!(ctx, "Processing TestTask {}", self.id);
        ctx.sleep(self.time).await;
        log_info!(ctx, "TestTask {} done", self.id);
    }
}

#[derive(Deserialize)]
pub struct ProdJob {
    time: f64,
    id: u32,
    extra_time: f64,
    // to_send_bytes: u32,
}

#[async_trait(?Send)]
impl JobProfile for ProdJob {
    async fn run(self: Box<Self>, ctx: SimulationContext) {
        log_info!(ctx, "Processing ProdTask {}", self.id);
        ctx.sleep(self.time).await;
        log_info!(ctx, "ProdTask {} main part done", self.id);
        ctx.sleep(self.extra_time).await;
        log_info!(ctx, "ProdTask {} done", self.id);

        futures::join!(Self::start_first_cmp(ctx.clone()), Self::start_second_cmp(ctx.clone()));
    }
}

impl ProdJob {
    async fn start_first_cmp(ctx: SimulationContext) {
        ctx.sleep(5.).await;
    }

    async fn start_second_cmp(ctx: SimulationContext) {
        ctx.sleep(10.).await;
    }

    async fn single_process(process: SimulationContext) {
        process.sleep(1.).await;
        // process.send(100, "task_processor").await;
    }
}

/// Job::run (self, allocations: Vec<Allocation>) {
///     let master_task = Task::new(allocations[0]);
///
///     let worker_task = Task::new(allocations[1]);
/// }
///
///

fn from_json<T>(json: &Value) -> Box<dyn JobProfile>
where
    T: DeserializeOwned + JobProfile + 'static,
{
    let task: T = serde_json::from_value(json.clone()).unwrap();
    Box::new(task)
}

struct TaskTypesStorage {
    tasks: HashMap<String, Box<dyn Fn(&Value) -> Box<dyn JobProfile>>>,
}

impl TaskTypesStorage {
    fn new() -> Self {
        TaskTypesStorage { tasks: HashMap::new() }
    }

    fn add_task<T>(&mut self, name: &str)
    where
        T: DeserializeOwned + JobProfile + 'static,
    {
        self.tasks.insert(name.to_string(), Box::new(from_json::<T>));
    }

    fn get_task_constructor(&self, name: &str) -> Option<&Box<dyn Fn(&Value) -> Box<dyn JobProfile>>> {
        self.tasks.get(name)
    }
}

fn test_json_reader() {
    Builder::from_default_env()
        .format(|buf, record| writeln!(buf, "{}", record.args()))
        .init();

    let mut task_storage = TaskTypesStorage::new();

    task_storage.add_task::<TestJob>("TestTask");
    task_storage.add_task::<ProdJob>("ProdTask");

    let mut file = File::open("src/tasks_list.json").unwrap();

    // Read the file contents into a string
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    let json_data: Value = serde_json::from_str(&contents).unwrap();

    let tasks = json_data
        .get("tasks")
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .map(|task| {
            let task_type = task.get("type").unwrap().as_str().unwrap();
            let task_constructor = task_storage.get_task_constructor(task_type).unwrap();
            task_constructor(task)
        })
        .collect::<Vec<Box<dyn JobProfile>>>();

    let mut sim = Simulation::new(42);

    let ctx = sim.create_context("task_processor");

    sim.spawn(async move {
        for task in tasks {
            println!("Task:");
            task.run(ctx.clone()).await;
        }
    });

    sim.step_until_no_events();
}

#[derive(Serialize, Deserialize)]
pub struct Resources {
    cpu: u32,
    memory: u64,
    disk: Option<u64>,
}

#[derive(Serialize, Deserialize)]
pub enum ResourcesRequirements {
    #[serde(rename = "homogenous")]
    Homogenous { nodes: u32, requirements: Resources },
    #[serde(rename = "precise")]
    Specific { nodes: Vec<Resources> },
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct ClusterWorkloadConfig {
    pub r#type: String,
    pub path: Option<String>,
    pub options: Option<HashMap<String, String>>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct RawConfig {
    pub workload: Option<Vec<ClusterWorkloadConfig>>,
}

fn main() {
    let yaml_file_path = "test.yaml";
    let raw: RawConfig = serde_yaml::from_str(
        &std::fs::read_to_string(yaml_file_path).unwrap_or_else(|_| panic!("Can't read file {}", yaml_file_path)),
    )
    .unwrap();

    println!("{:?}", raw);
}
