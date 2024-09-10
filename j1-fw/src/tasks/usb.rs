use defmt::{panic, *};
use embassy_futures::join::join;
use embassy_stm32::peripherals::{USB_OTG_FS, PA11, PA12};
use embassy_stm32::usb_otg::{Driver, Instance};
use embassy_stm32::{bind_interrupts, peripherals, usb_otg};
use embassy_usb::class::cdc_acm::{CdcAcmClass, State};
use embassy_usb::driver::EndpointError;
use embassy_usb::Builder;
use embassy_time::{Duration, Ticker};

use heapless::Vec;
// use micropb::PbWrite;
use micropb::{
    // heapless::Vec,
    MessageEncode, PbEncoder,
};
use crate::utils::rtc_read::get_ticks_since_boot;
use crate::tasks::accel_mon::ACCEL;

mod proto {
    #![allow(clippy::all)]
    #![allow(nonstandard_style, unused, irrefutable_let_patterns)]
    // Let's assume that Example is the only message define in the .proto file that has been 
    // converted into a Rust struct
    include!(concat!(env!("OUT_DIR"), "/j1_proto.rs"));
}

use proto::accel_::*;

bind_interrupts!(struct Irqs {
    OTG_FS => usb_otg::InterruptHandler<peripherals::USB_OTG_FS>;
});

#[embassy_executor::task]
pub async fn usb_task(usb_dev: USB_OTG_FS, pin1: PA12, pin2: PA11) {
    // Create the driver, from the HAL.
    let mut ep_out_buffer = [0u8; 256];
    let mut config = embassy_stm32::usb_otg::Config::default();

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
            let _ = stream_telem(&mut class).await;
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

async fn stream_telem<'d, T: Instance + 'd>(class: &mut CdcAcmClass<'d, Driver<'d, T>>) -> Result<(), Disconnected> {
    let mut ticker = Ticker::every(Duration::from_millis(100));
    let mut stream = Vec::<u8, 64>::new();
    loop {
        stream.clear();
        let mut encoder = PbEncoder::new(&mut stream);
        let telem = construct_telem();
        telem.encode_len_delimited(&mut encoder).unwrap();
        class.write_packet(stream.as_slice()).await?;
        ticker.next().await;
    }
}
fn construct_telem() -> Accel {
    let now = get_ticks_since_boot();
    info!("now: {}", now);
    let accel_data = ACCEL.lock(|f| {
        return f.clone().unwrap();
    });
    let data = Accel {
            time: now,
            x_accel: accel_data.x,
            y_accel: accel_data.y,
            z_accel: accel_data.z,
            _has: Accel_::_Hazzer::default().init_x_accel().init_y_accel().init_z_accel()
        };
    info!("x: {}, y: {}, z: {}, timestamp: {}", data.x_accel, data.y_accel, data.z_accel, data.time);
    data
}
