use defmt::*;
use core::cell::RefCell;
use embassy_sync::blocking_mutex::{Mutex, raw::ThreadModeRawMutex};
use crate::modules::accelerometer::{AccelData, Accelerometer};
use embassy_time::{Duration, Timer};

pub static ACCEL: Mutex<ThreadModeRawMutex, RefCell<AccelData>> = Mutex::new(RefCell::new(AccelData { x: 0.0, y: 0.0, z: 0.0 }));

#[embassy_executor::task]
pub async fn accel_task(mut device: impl Accelerometer + 'static) -> ! {
    loop {
        Timer::after(Duration::from_millis(1000)).await;
        let accel_vectors = device.read_accel().await;
        ACCEL.lock(|f| {
            let mut binding = f.borrow_mut();
            binding.x = accel_vectors.x;
            binding.y = accel_vectors.y;
            binding.z = accel_vectors.z;
        });
        info!("x DATA is {:?}", accel_vectors.x);
        info!("y DATA is {:?}", accel_vectors.y);
        info!("z DATA is {:?}", accel_vectors.z);
        info!("total accel is {:?}", accel_vectors.x + accel_vectors.y + accel_vectors.z);
    }
}
