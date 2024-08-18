use core::sync::atomic::{AtomicU32, Ordering};
use embassy_stm32::gpio::{AnyPin, Level, Output, Speed};
use embassy_time::{Duration, Timer};

pub struct HeartBeat<'a> {
    led: Output<'a, AnyPin>,
    del: AtomicU32,
}

impl HeartBeat<'_> {
    pub fn init(pin: AnyPin, delay: u32) -> HeartBeat<'static> {
        let led = Output::new(pin, Level::Low, Speed::Low);
        let del = AtomicU32::new(delay);
        HeartBeat {
            led, del
        }
    }
}

#[embassy_executor::task]
pub async fn heartbeat_task(mut h: HeartBeat<'static>) {
    loop {
        Timer::after(Duration::from_millis(h.del.load(Ordering::Relaxed) as u64)).await;
        h.led.toggle();
    }
}
