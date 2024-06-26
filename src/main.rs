#![no_std]
#![no_main]

mod tasks;
mod utils;
mod modules;

// use tasks::accelerometer::{AccelMon, accel_task};
// use tasks::accelerometer::{accel_task, AccelError};
use tasks::heartbeat::{HeartBeat, heartbeat_task};
use modules::lis302dl;

use utils::button_mon::{ButtonMon, button_task};

use embassy_executor::Spawner;
use static_cell::StaticCell;
use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice;
use embassy_sync::mutex::Mutex;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_stm32::exti::Channel;
use embassy_stm32::peripherals::{DMA2_CH3, DMA2_CH2, SPI1};
use embassy_stm32::gpio::{Pin, Level, Output, Speed};
use embassy_stm32::spi::{Config, Spi};
use embassy_stm32::spi;
use embassy_stm32::time::Hertz;
use panic_halt as _;
use defmt::*;
use defmt_rtt as _;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // Initialize the peripherals
    let p = embassy_stm32::init(Default::default());

    // Set up a heartbeat LED to we know we're still working
    info!("{:?} Initializing heartbeat!", file!());
    let heartbeat_pin = p.PD12.degrade(); // green LED
    let heart = HeartBeat::init(heartbeat_pin, 1000);
    spawner.spawn(heartbeat_task(heart)).unwrap();

    // Set up a button to have a blink and log message

    // Select which pins are used
    info!("{:?} Initializing button!", file!());
    let led= p.PD14.degrade(); // red LED
    let button_pin = p.PA0.degrade();
    let int= p.EXTI0.degrade();
    // Configure the button monitor
    let button_monitor = ButtonMon::init(led, button_pin, int);
    // Start the button monitoring task
    spawner.spawn(button_task(button_monitor)).unwrap();

    // Set up communication to the accelerometer
    info!("{:?} Initializing SPI for accelerometer", file!());
        static SPI_BUS: StaticCell<Mutex<NoopRawMutex, spi::Spi<SPI1, DMA2_CH3, DMA2_CH2>>> = StaticCell::new();
    let mut spi_config = Config::default();
    spi_config.frequency = Hertz(1_000_000);
    let spi= Spi::new(p.SPI1, p.PA5 ,p.PA7 ,p.PA6 ,p.DMA2_CH3 ,p.DMA2_CH2 , spi_config);
    let mut chip_select = Output::new(p.PE3, Level::High, Speed::High);
    chip_select.set_high();
    let spi_bus = Mutex::new(spi);
    let spi_bus = SPI_BUS.init(spi_bus);
    let spi_dev1 = SpiDevice::new(spi_bus, chip_select);
    let config = lis302dl::Config::default();
    let mut accel_mon = lis302dl::Lis302Dl::new(spi_dev1, config);
    let _ = accel_mon.init().await;
    // spawner.spawn(accel_task(accel_mon)).unwrap();
}
    
