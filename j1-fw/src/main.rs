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
use utils::rtc_read::{get_ticks, init};

use embedded_io_async::BufRead;
use embassy_executor::Spawner;
use static_cell::StaticCell;
use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice;
use embassy_sync::mutex::Mutex;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_stm32::{
    bind_interrupts,
    exti::Channel,
    peripherals::{self, DMA2_CH3, DMA2_CH2, SPI1},
    gpio::{Pin, Level, Output, Speed},
    spi::{Config as SpiConfig, Spi},
    {Config as Stm32_Config, spi},
    time::Hertz,
    usart::{self, Config as Usart_Config, BufferedUart},
};
use panic_halt as _;
use defmt::*;
use defmt_rtt as _;

bind_interrupts!(struct Irqs {
    USART1 => usart::BufferedInterruptHandler<peripherals::USART1>;
});


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


    info!("{:?} Initializing RTC!", file!());
    init(p.RTC);
    info!("{:?} RTC init at time {}!", file!(), get_ticks().and_utc().timestamp_millis());

    // Set up a heartbeat LED to we know we're still working
    info!("{:?} Initializing heartbeat!", file!());
    let heartbeat_pin = p.PD12.degrade(); // green LED
    let heart = HeartBeat::init(heartbeat_pin, 1000);
    spawner.must_spawn(heartbeat_task(heart));

    // Set up a button to have a blink and log message

    // Select which pins are used
    info!("{:?} Initializing button!", file!());
    let led= p.PD14.degrade(); // red LED
    let button_pin = p.PA0.degrade();
    let int= p.EXTI0.degrade();
    // Configure the button monitor
    let button_monitor = ButtonMon::init(led, button_pin, int);
    // Start the button monitoring task
    spawner.must_spawn(button_task(button_monitor));

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
    spawner.must_spawn(accel_task(lis302dl_device));

    info!("{:?} Initializing USB", file!());
    spawner.must_spawn(usb_task(p.USB_OTG_FS, p.PA12, p.PA11));

    let uart_config = Usart_Config::default();
    let mut tx_buf = [0u8; 32];
    let mut rx_buf = [0u8; 32];

    let mut buf_usart = BufferedUart::new(p.USART1, Irqs, p.PA10, p.PA9, &mut tx_buf, &mut rx_buf, uart_config).unwrap();

    loop {
        let buf = buf_usart.fill_buf().await.unwrap();
        info!("Received: {}", buf);

        // Read bytes have to be explicitly consumed, otherwise fill_buf() will return them again
        let n = buf.len();
        buf_usart.consume(n);
    }
}
    
