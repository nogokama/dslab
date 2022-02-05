use log::debug;
use std::collections::BTreeMap;

use core::actor::ActorContext;

use crate::model::*;

#[derive(Debug, Clone)]
struct DataTransfer {
    size_left: f64,
    last_speed: f64,
    last_time: f64,
    receive_event: u64,
    data: Data,
}

pub struct SharedBandwidthNetwork {
    bandwidth: f64,
    latency: f64,
    transfers: BTreeMap<usize, DataTransfer>,
}

impl SharedBandwidthNetwork {
    pub fn new(bandwidth: f64, latency: f64) -> SharedBandwidthNetwork {
        return SharedBandwidthNetwork {
            bandwidth,
            latency,
            transfers: BTreeMap::new(),
        };
    }

    fn recalculate_receive_time(&mut self, ctx: &mut ActorContext) {
        let cur_time = ctx.time();
        for (_, send_elem) in self.transfers.iter_mut() {
            let delivery_time = cur_time - send_elem.last_time;
            send_elem.size_left -= delivery_time * send_elem.last_speed;
            ctx.cancel_event(send_elem.receive_event);
        }

        let new_bandwidth = self.bandwidth / (self.transfers.len() as f64);

        for (_, send_elem) in self.transfers.iter_mut() {
            send_elem.last_speed = new_bandwidth;
            send_elem.last_time = cur_time;
            let data_delivery_time = send_elem.size_left / new_bandwidth;
            send_elem.receive_event = ctx.emit(
                DataReceive {
                    data: send_elem.data.clone(),
                },
                ctx.id.clone(),
                data_delivery_time,
            );
            debug!(
                "System time: {}, Calculate. Data ID: {}, From: {}, To {}, Size: {}, SizeLeft: {}, New Time: {}",
                ctx.time(),
                send_elem.data.id,
                send_elem.data.src,
                send_elem.data.dest,
                send_elem.data.size,
                send_elem.size_left,
                data_delivery_time
            );
        }
    }
}

impl NetworkConfiguration for SharedBandwidthNetwork {
    fn latency(&self) -> f64 {
        self.latency
    }
}

impl DataOperation for SharedBandwidthNetwork {
    fn send_data(&mut self, data: Data, ctx: &mut ActorContext) {
        let new_send_data_progres = DataTransfer {
            size_left: data.size,
            last_speed: 0.,
            last_time: 0.,
            receive_event: 0,
            data,
        };

        let data_id = new_send_data_progres.data.id;
        if self.transfers.insert(data_id, new_send_data_progres).is_some() {
            panic!("SharedBandwidthNetwork: data with id {} already exist", data_id);
        }

        self.recalculate_receive_time(ctx);
    }

    fn receive_data(&mut self, data: Data, ctx: &mut ActorContext) {
        self.transfers.remove(&data.id);
        self.recalculate_receive_time(ctx);
    }
}

impl NetworkModel for SharedBandwidthNetwork {}