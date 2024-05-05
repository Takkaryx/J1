#![no_std]
#![no_main]

mod tasks {
    pub mod heartbeat;
}

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::gpio::{Input, Level, Output, Pin, Pull, Speed};
use embassy_time::{Duration, Timer};
use panic_halt as _;
use tasks::heartbeat::heartbeat;

#[embassy_executor::main]
async fn main(spawner: Spawner) -> ! {
    let p = embassy_stm32::init(Default::default());
    let heartbeat_pin = p.PD14.degrade();
    let other_pin = p.PD12.degrade();

    spawner.spawn(heartbeat(heartbeat_pin, 1000)).unwrap();
    let button = Input::new(p.PA0, Pull::None);
    let mut button = ExtiInput::new(button, p.EXTI0);
    let mut led = Output::new(other_pin, Level::Low, Speed::Low);

    loop {
        // Check if button got pressed
        button.wait_for_rising_edge().await;
        led.toggle();
        info!("{:?} button press detected!", file!());
        Timer::after(Duration::from_millis(250)).await;
    }
}
