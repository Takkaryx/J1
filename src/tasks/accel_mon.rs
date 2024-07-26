use defmt::*;
use crate::modules::accelerometer::Accelerometer;
use embassy_time::{Duration, Timer};

#[embassy_executor::task]
pub async fn accel_task(mut device: impl Accelerometer + 'static) -> ! {
    loop {
        Timer::after(Duration::from_millis(1000)).await;
        let accel_vectors = device.read_accel().await;
        info!("x DATA is {:?}", accel_vectors.x);
        info!("y DATA is {:?}", accel_vectors.y);
        info!("z DATA is {:?}", accel_vectors.z);
        info!("total accel is {:?}", accel_vectors.x + accel_vectors.y + accel_vectors.z);
    }
}
