#![no_std]

use postcard::experimental::schema::Schema;
use postcard_rpc::{endpoint, topic};
use serde::{Deserialize, Serialize};

endpoint!(PingEndpoint, u32, u32, "ping");
endpoint!(StartAccelerationEndpoint, StartAccel, (), "accel/start");
endpoint!(StopAccelerationEndpoint, (), bool, "accel/stop");

topic!(AccelTopic, Acceleration, "accel/data");

#[derive(Serialize, Deserialize, Schema, Debug, PartialEq)]
pub struct Acceleration {
    pub x: i16,
    pub y: i16,
    pub z: i16,
}

#[derive(Serialize, Deserialize, Schema, Debug, PartialEq)]
pub struct StartAccel {
    pub interval_ms: u32,
}
