use defmt::{panic, *};
use embassy_futures::join::join;
use embassy_stm32::peripherals::{USB_OTG_FS, PA11, PA12};
use embassy_stm32::usb_otg::{Driver, Instance};
use embassy_stm32::{bind_interrupts, peripherals, usb_otg};
use embassy_usb::class::cdc_acm::{CdcAcmClass, State};
use embassy_usb::driver::EndpointError;
use embassy_usb::Builder;
use embassy_time::{Duration, Ticker};
use crate::tasks::accel_mon::ACCEL_TELEM_MUTEX_BLOCKING;

bind_interrupts!(struct Irqs {
    OTG_FS => usb_otg::InterruptHandler<peripherals::USB_OTG_FS>;
});

#[embassy_executor::task]
pub async fn usb_task(usb_dev: USB_OTG_FS, pin1: PA12, pin2: PA11) {
    // Create the driver, from the HAL.
    let mut ep_out_buffer = [0u8; 256];
    let mut config = embassy_stm32::usb_otg::Config::default();

    // Do not enable vbus_detection. This is a safe default that works in all boards.
    // However, if your USB device is self-powered (can stay powered on if USB is unplugged), you need
    // to enable vbus_detection to comply with the USB spec. If you enable it, the board
    // has to support it or USB won't work at all. See docs on `vbus_detection` for details.
    config.vbus_detection = true;

    let driver = Driver::new_fs(usb_dev, Irqs, pin1, pin2, &mut ep_out_buffer, config);

    // Create embassy-usb Config
    let mut config = embassy_usb::Config::new(0xcafe, 0xc0de);
    config.manufacturer = Some("J1 USB");
    config.product = Some("Accelerometer_Prototype");
    config.serial_number = Some("12345678");

    // Required for windows compatibility.
    // https://developer.nordicsemi.com/nRF_Connect_SDK/doc/1.9.1/kconfig/CONFIG_CDC_ACM_IAD.html#help
    config.device_class = 0xEF;
    config.device_sub_class = 0x02;
    config.device_protocol = 0x01;
    config.composite_with_iads = true;

    // Create embassy-usb DeviceBuilder using the driver and config.
    // It needs some buffers for building the descriptors.
    let mut config_descriptor = [0; 256];
    let mut bos_descriptor = [0; 256];
    let mut control_buf = [0; 64];

    let mut state = State::new();

    let mut builder = Builder::new(
        driver,
        config,
        &mut config_descriptor,
        &mut bos_descriptor,
        &mut [], // no msos descriptors
        &mut control_buf,
    );

    // Create classes on the builder.
    let mut class = CdcAcmClass::new(&mut builder, &mut state, 64);

    // Build the builder.
    let mut usb = builder.build();

    // Run the USB device.
    let usb_fut = usb.run();

    // Do stuff with the class!
    // For a serial connection, can use the command
    // cu -s 115200 -l /dev/<PATH TO cu.usb> to get a shell to this connection
    let echo_fut = async {
        loop {
            class.wait_connection().await;
            info!("Connected");
            let _ = echo(&mut class).await;
            info!("Disconnected");
        }
    };

    // Run everything concurrently.
    // If we had made everything `'static` above instead, we could do this using separate tasks instead.
    join(usb_fut, echo_fut).await;
}

struct Disconnected {}

impl From<EndpointError> for Disconnected {
    fn from(val: EndpointError) -> Self {
        match val {
            EndpointError::BufferOverflow => panic!("Buffer overflow"),
            EndpointError::Disabled => Disconnected {},
        }
    }
}

async fn echo<'d, T: Instance + 'd>(class: &mut CdcAcmClass<'d, Driver<'d, T>>) -> Result<(), Disconnected> {
    let mut buf = [0; 64];
    let mut process_ticker = Ticker::every(Duration::from_millis(100));
    loop {

        // let n = class.read_packet(&mut buf).await?;
        // let data = &buf[..64];
        // info!("data: {:x}", data);
        ACCEL_TELEM_MUTEX_BLOCKING.lock(|accel_data| {
            buf[0] = (accel_data.borrow().x * 100.0) as i8;
            buf[1] = (accel_data.borrow().y * 100.0) as i8;
            buf[2] = (accel_data.borrow().z * 100.0) as i8;
        });

        info!("x data is {:?}", buf[0]);
        info!("y data is {:?}", buf[1]);
        info!("z data is {:?}", buf[2]);
        class.write_packet(&buf).await?;
        process_ticker.next().await;
    }
}
