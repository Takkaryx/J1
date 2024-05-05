use core::sync::atomic::{AtomicU32, Ordering};
use embassy_stm32::gpio::{AnyPin, Level, Output, Speed};
use embassy_time::{Duration, Timer};

#[embassy_executor::task]
pub async fn heartbeat(pin: AnyPin, delay: u32) {
    let del = AtomicU32::new(delay);
    let mut led = Output::new(pin, Level::Low, Speed::Low);
    loop {
        Timer::after(Duration::from_millis(del.load(Ordering::Relaxed) as u64)).await;
        led.toggle();
    }
}
