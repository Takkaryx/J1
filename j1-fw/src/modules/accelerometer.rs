// use crate::modules::error::Error;
// use core::fmt::Debug;

// #[derive(Serialize)]
pub struct AccelData {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

pub trait Accelerometer {
    // type Error: Debug;
    async fn read_accel(&mut self) -> AccelData;
}
