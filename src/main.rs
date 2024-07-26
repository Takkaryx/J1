#![no_std]
#![no_main]

mod tasks;
mod utils;
mod modules;

use modules::lis302dl::Lis302Dl;

use tasks::heartbeat::{HeartBeat, heartbeat_task};
use tasks::accel_mon::accel_task;
use tasks::usb::usb_task;

use utils::button_mon::{ButtonMon, button_task};

use embassy_executor::Spawner;
use static_cell::StaticCell;
use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice;
use embassy_sync::mutex::Mutex;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_stm32::exti::Channel;
use embassy_stm32::peripherals::{DMA2_CH3, DMA2_CH2, SPI1};
use embassy_stm32::gpio::{Pin, Level, Output, Speed};
use embassy_stm32::spi::{Config as SpiConfig, Spi};
use embassy_stm32::{Config as Stm32_Config, spi};
use embassy_stm32::time::Hertz;
use panic_halt as _;
use defmt::*;
use defmt_rtt as _;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // Initialize the peripherals
    let mut config = Stm32_Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.hse = Some(Hse {
            freq: Hertz(8_000_000),
            mode: HseMode::Bypass,
        });
        config.rcc.pll_src = PllSource::HSE;
        config.rcc.pll = Some(Pll {
            prediv: PllPreDiv::DIV4,
            mul: PllMul::MUL168,
            divp: Some(PllPDiv::DIV2), // 8mhz / 4 * 168 / 2 = 168Mhz.
            divq: Some(PllQDiv::DIV7), // 8mhz / 4 * 168 / 7 = 48Mhz.
            divr: None,
        });
        config.rcc.ahb_pre = AHBPrescaler::DIV1;
        config.rcc.apb1_pre = APBPrescaler::DIV4;
        config.rcc.apb2_pre = APBPrescaler::DIV2;
        config.rcc.sys = Sysclk::PLL1_P;
    }
    let p = embassy_stm32::init(config);

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
    info!("{:?} Initializing SPI", file!());
    static SPI_BUS: StaticCell<Mutex<NoopRawMutex, spi::Spi<SPI1, DMA2_CH3, DMA2_CH2>>> = StaticCell::new();
    let mut spi_config = SpiConfig::default();
    spi_config.frequency = Hertz(1_000_000);
    spi_config.mode = spi::MODE_1;
    let spi= Spi::new(p.SPI1, p.PA5 ,p.PA7 ,p.PA6 ,p.DMA2_CH3 ,p.DMA2_CH2 , spi_config);
    let mut chip_select = Output::new(p.PE3, Level::High, Speed::High);
    chip_select.set_high();
    let spi_bus = Mutex::new(spi);
    let spi_bus = SPI_BUS.init(spi_bus);
    let spi_dev1 = SpiDevice::new(spi_bus, chip_select);

    info!("{:?} Initializing Accelerometer", file!());
    let config = modules::lis302dl::Config::default();
    let mut lis302dl_device = Lis302Dl::new(spi_dev1, config);
    let _ = lis302dl_device.init().await;
    spawner.spawn(accel_task(lis302dl_device)).unwrap();

    info!("{:?} Initializing USB", file!());
    spawner.spawn(usb_task(p.USB_OTG_FS, p.PA12, p.PA11)).unwrap();
}
    
