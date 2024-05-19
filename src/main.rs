#![no_std]
#![no_main]

mod tasks {
    pub mod heartbeat;
    pub mod accelerometer;
}
use tasks::accelerometer::{AccelMon, accel_task};
use tasks::heartbeat::{HeartBeat, heartbeat_task};

mod utils {
    pub mod button_mon;
}
use utils::button_mon::{ButtonMon, button_task};

use embassy_executor::Spawner;
use embassy_stm32::exti::Channel;
use embassy_stm32::gpio::Pin;
use embassy_stm32::spi::{Config, Spi};
use embassy_stm32::time::Hertz;
use panic_halt as _;
use defmt::*;
use defmt_rtt as _;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // Initialize the peripherals
    let p = embassy_stm32::init(Default::default());

    // Set up a heartbeat LED to we know we're still working
    let heartbeat_pin = p.PD12.degrade(); // green LED
    let heart = HeartBeat::init(heartbeat_pin, 1000);
    info!("{:?} Initializing heartbeat!", file!());
    spawner.spawn(heartbeat_task(heart)).unwrap();

    // Set up a button to have a blink and log message

    // Select which pins are used
    let led= p.PD14.degrade(); // red LED
    let button_pin = p.PA0.degrade();
    let int= p.EXTI0.degrade();
    // Configure the button monitor
    let button_monitor = ButtonMon::init(led, button_pin, int);
    // Start the button monitoring task
    info!("{:?} Initializing button!", file!());
    spawner.spawn(button_task(button_monitor)).unwrap();

    // Set up communication to the accelerometer
    let mut spi_config = Config::default();
    spi_config.frequency = Hertz(1_000_000);
    let spi_bus = Spi::new(p.SPI1, p.PA5 ,p.PA7 ,p.PA6 ,p.DMA2_CH3 ,p.DMA2_CH2 , spi_config);
    info!("{:?} Initializing SPI for accelerometer", file!());
    let accel_monitor = AccelMon::init(spi_bus);
    spawner.spawn(accel_task(accel_monitor)).unwrap();

}
