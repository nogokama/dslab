use std::{
    collections::HashMap,
    fs::File,
    io::{BufWriter, Write},
};

use dslab_core::Id;
use serde::Serialize;

use crate::config::sim_config::HostConfig;

#[derive(Serialize, Clone, Copy)]
pub struct ResourcesState {
    pub cpu: f64,
    pub memory: f64,
    pub disk: Option<f64>,
}

impl ResourcesState {
    pub fn diff(&self, other: &ResourcesState) -> ResourcesState {
        ResourcesState {
            cpu: self.cpu - other.cpu,
            memory: self.memory - other.memory,
            disk: self.disk.zip(other.disk).map(|(a, b)| a - b),
        }
    }
}

#[derive(Clone, Serialize)]
pub struct ResourceLoad {
    pub total: f64,
    pub consumed: f64,
    pub previous_update: f64,
}

impl ResourceLoad {
    pub fn new(total: f64) -> Self {
        Self {
            total,
            consumed: 0.,
            previous_update: 0.,
        }
    }

    pub fn update(&mut self, consumed_value: f64) -> f64 {
        let previous_update = self.previous_update;
        self.previous_update = consumed_value;
        self.consumed += consumed_value;
        previous_update
    }

    pub fn add(&mut self, addition: f64) {
        let consumed_value = self.previous_update + addition;
        self.update(consumed_value);
    }

    pub fn reset(&mut self, points_cnt: u64) -> f64 {
        let result = (self.consumed / points_cnt as f64) / self.total;
        self.consumed = 0.;
        result
    }

    pub fn extend(&mut self, amount: f64) {
        self.total += amount;
    }
}

#[derive(Clone, Serialize)]
pub struct LoadInfo {
    pub cpu: ResourceLoad,
    pub memory: ResourceLoad,
    pub disk_capacity: Option<ResourceLoad>,
    pub limit_points: u32,
    pub points_cnt: u64,
}

impl LoadInfo {
    pub fn new(cpu: f64, memory: f64, disk_capacity: Option<f64>, limit_points: u32) -> Self {
        Self {
            cpu: ResourceLoad::new(cpu),
            memory: ResourceLoad::new(memory),
            disk_capacity: disk_capacity.map(ResourceLoad::new),
            limit_points,
            points_cnt: 0,
        }
    }

    pub fn extend(&mut self, other: &Self) {
        self.cpu.extend(other.cpu.total);
        self.memory.extend(other.memory.total);
        if let Some(disk_capacity) = &mut self.disk_capacity {
            if let Some(other_disk_capacity) = &other.disk_capacity {
                disk_capacity.extend(other_disk_capacity.total);
            }
        }
    }

    pub fn update(&mut self, state: ResourcesState) -> ResourcesState {
        self.points_cnt += 1;
        ResourcesState {
            cpu: self.cpu.update(state.cpu),
            memory: self.memory.update(state.memory),
            disk: state
                .disk
                .map(|v| self.disk_capacity.as_mut().map(|d| d.update(v)))
                .flatten(),
        }
    }

    pub fn add(&mut self, state: ResourcesState) {
        self.points_cnt += 1;
        self.cpu.add(state.cpu);
        self.memory.add(state.memory);
        if let Some(disk) = state.disk {
            if let Some(disk_capacity) = &mut self.disk_capacity {
                disk_capacity.add(disk);
            }
        }
    }

    pub fn dump(&mut self) -> Option<ResourcesState> {
        if self.points_cnt == self.limit_points as u64 {
            let result = ResourcesState {
                cpu: self.cpu.reset(self.points_cnt),
                memory: self.memory.reset(self.points_cnt),
                disk: self.disk_capacity.as_mut().map(|v| v.reset(self.points_cnt)),
            };
            self.points_cnt = 0;
            Some(result)
        } else {
            None
        }
    }
}

pub struct Monitoring {
    pub compression: u32,
    pub hosts: HashMap<String, LoadInfo>,
    pub groups: HashMap<String, LoadInfo>,
    pub total: LoadInfo,

    host_log_file: BufWriter<File>,
}

impl Monitoring {
    pub fn new(compression: u32) -> Monitoring {
        let host_log_file_path = "load.txt";
        let host_log_file = BufWriter::new(File::create(&host_log_file_path).unwrap());
        Monitoring {
            compression,
            hosts: HashMap::new(),
            groups: HashMap::new(),
            total: LoadInfo::new(0., 0., None, compression),
            host_log_file,
        }
    }

    pub fn add_host(&mut self, name: String, host_config: &HostConfig) {
        let host_load_info = LoadInfo::new(
            host_config.cpus as f64,
            host_config.memory as f64,
            host_config.disk_capacity.map(|v| v as f64),
            self.compression,
        );

        if let Some(group_name) = &host_config.group_prefix {
            self.groups
                .entry(group_name.clone())
                .or_insert_with(|| LoadInfo::new(0., 0., None, self.compression))
                .extend(&host_load_info);
        }

        self.total.extend(&host_load_info);

        self.hosts.insert(name.clone(), host_load_info);
    }

    pub fn update_host(
        &mut self,
        time: f64,
        name: &str,
        group_name: Option<&String>,
        cpu_used: u32,
        memory_used: u64,
        disk_used: Option<u64>,
    ) {
        let host_load_info = self.hosts.get_mut(name).unwrap();
        let mut state = ResourcesState {
            cpu: cpu_used as f64,
            memory: memory_used as f64,
            disk: disk_used.map(|v| v as f64),
        };

        let old_state = host_load_info.update(state);

        state = state.diff(&old_state);

        if let Some(group_name) = group_name {
            self.groups.get_mut(group_name).unwrap().add(state);
        }

        self.total.add(state);

        if let Some(state) = host_load_info.dump() {
            self.dump_to_log_file(state, time, name);
        }
        if let Some(group_name) = group_name {
            if let Some(state) = self.groups.get_mut(group_name).unwrap().dump() {
                self.dump_to_log_file(state, time, &format!("group-{}", group_name));
            }
        }
        if let Some(state) = self.total.dump() {
            self.dump_to_log_file(state, time, "TOTAL");
        }
    }

    fn dump_to_log_file(&mut self, state: ResourcesState, time: f64, name: &str) {
        writeln!(
            &mut self.host_log_file,
            "{} {} {} {}",
            time,
            name,
            state.cpu,
            state.memory,
            // state.disk.unwrap_or(0.)
        )
        .unwrap();
    }
}
