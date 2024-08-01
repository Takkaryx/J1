use defmt::*;
use crate::modules::accelerometer::{Accel, Accelerometer};
use embassy_time::{Duration, Ticker};
use static_cell::StaticCell;
use embassy_sync::blocking_mutex::{Mutex, raw::ThreadModeRawMutex};

static ACCEL: StaticCell<Mutex<ThreadModeRawMutex, Accel>> = StaticCell::new();

#[embassy_executor::task]
pub async fn accel_task(mut device: impl Accelerometer + 'static) -> ! {
    let mut ticker = Ticker::every(Duration::from_secs(1));
    let accel_ref = ACCEL.init(Mutex::new(Accel {x: 0.0, y: 0.0, z: 0.0}));
    loop {
        let accel_vectors = device.read_accel().await;
        ACCEL.lock(|data| {
            data.borrow_mut().x = accel_vectors.x; 
            data.borrow_mut().y = accel_vectors.y; 
            data.borrow_mut().z = accel_vectors.z;
        });

        debug!("x DATA is {:?}", accel_vectors.x);
        debug!("y DATA is {:?}", accel_vectors.y);
        debug!("z DATA is {:?}", accel_vectors.z);
        info!("total accel is {:?}", accel_vectors.x + accel_vectors.y + accel_vectors.z);
        ticker.next().await;
    }
}
