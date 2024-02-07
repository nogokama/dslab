use std::collections::HashMap;

use dslab_core::Id;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct HostConfig {
    pub id: Id,
    pub name: String,
    pub cpus: u32,
    pub memory: u64,

    pub cpu_speed: Option<f64>,
    pub disk_capacity: Option<u64>,
    pub disk_read_bw: Option<f64>,
    pub disk_write_bw: Option<f64>,
    pub local_newtork_bw: Option<f64>,
    pub local_newtork_latency: Option<f64>,
}

impl HostConfig {
    pub fn from_group_config(group: &GroupHostConfig, idx: Option<u32>) -> Self {
        let name = if group.count.unwrap_or(1) == 1 {
            group
                .name
                .clone()
                .unwrap_or_else(|| panic!("name is required for host group with count = 1"))
        } else {
            format!(
                "{}-{}",
                group
                    .name_prefix
                    .clone()
                    .unwrap_or_else(|| panic!("name_prefix is required for host group with count > 1")),
                idx.unwrap()
            )
        };
        Self {
            id: 0,
            name,
            cpus: group.cpus,
            memory: group.memory,
            cpu_speed: group.cpu_speed,
            disk_capacity: group.disk_capacity,
            disk_read_bw: group.disk_read_bw,
            disk_write_bw: group.disk_write_bw,
            local_newtork_bw: group.local_newtork_bw,
            local_newtork_latency: group.local_newtork_latency,
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct RawSimulationConfig {
    pub hosts: Option<Vec<GroupHostConfig>>,
    pub schedulers: Option<Vec<SchedulerConfig>>,
    pub workload: Option<Vec<ClusterWorkloadConfig>>,
    pub newtork: Option<NetworkConfig>,
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct GroupHostConfig {
    pub name: Option<String>,
    pub name_prefix: Option<String>,

    pub cpus: u32,
    pub memory: u64,

    pub cpu_speed: Option<f64>,
    pub disk_capacity: Option<u64>,
    pub disk_read_bw: Option<f64>,
    pub disk_write_bw: Option<f64>,
    pub local_newtork_bw: Option<f64>,
    pub local_newtork_latency: Option<f64>,

    pub count: Option<u32>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct NetworkConfig {
    pub shared: Option<bool>,
    pub local_latency: f64,
    pub local_bandwidth: f64,
    pub latency: f64,
    pub bandwidth: f64,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct SchedulerConfig {
    pub name: Option<String>,
    pub name_prefix: Option<String>,
    pub algorithm: String,
    pub count: Option<u32>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct ClusterWorkloadConfig {
    pub r#type: String,
    pub path: Option<String>,
    pub options: Option<HashMap<String, String>>,
}

/// Represents simulation configuration.
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct SimulationConfig {
    /// Used VM trace dataset.
    pub workload: Option<Vec<ClusterWorkloadConfig>>,
    /// Configurations of physical hosts.
    pub hosts: Vec<GroupHostConfig>,
    /// Configurations of VM schedulers.
    pub schedulers: Vec<SchedulerConfig>,
    pub network: Option<NetworkConfig>,
}

impl SimulationConfig {
    /// Creates simulation config by reading parameter values from YAM file
    /// (uses default values if some parameters are absent).
    pub fn from_file(file_name: &str) -> Self {
        let raw: RawSimulationConfig = serde_yaml::from_str(
            &std::fs::read_to_string(file_name).unwrap_or_else(|_| panic!("Can't read file {}", file_name)),
        )
        .unwrap_or_else(|_| panic!("Can't parse YAML from file {}", file_name));

        Self {
            workload: raw.workload,
            hosts: raw.hosts.unwrap_or_default(),
            schedulers: raw.schedulers.unwrap_or_default(),
            network: raw.newtork,
        }
    }
}
