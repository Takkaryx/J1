use embassy_stm32::peripherals::{USB_OTG_FS, PA11, PA12};
use embassy_stm32::usb_otg::Driver;
use embassy_stm32::{bind_interrupts, peripherals, usb_otg};
use embassy_usb::Builder;

use postcard_rpc::{
    define_dispatch,
    target_server::{
        buffers::AllBuffers, configure_usb, example_config, rpc_dispatch, sender::Sender,
        SpawnContext,
    },
    WireHeader,
};

static ALL_BUFFERS: ConstStaticCell<AllBuffers<256, 256, 256>> =
    ConstStaticCell::new(AllBuffers::new());

pub struct Context {
    pub unique_id: u64,
    pub ws2812: Ws2812<'static, PIO0, 0, 24>,
    pub ws2812_state: [RGB8; 24],
    pub accel: &'static Mutex<ThreadModeRawMutex, Accel>,
}

pub struct SpawnCtx {
    pub accel: &'static Mutex<ThreadModeRawMutex, Accel>,
}

impl SpawnContext for Context {
    type SpawnCtxt = SpawnCtx;
    fn spawn_ctxt(&mut self) -> Self::SpawnCtxt {
        SpawnCtx { accel: self.accel }
    }
}

define_dispatch! {
    dispatcher: Dispatcher<
        Mutex = ThreadModeRawMutex,
        Driver = usb::Driver<'static, USB>,
        Context = Context,
    >;
    PingEndpoint => blocking ping_handler,
    StartAccelerationEndpoint => spawn accelerometer_handler,
    StopAccelerationEndpoint => blocking accelerometer_stop_handler,
}

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

    // Create embassy-usb DeviceBuilder using the driver and config.
    // It needs some buffers for building the descriptors.
    let mut config_descriptor = [0; 256];
    let mut bos_descriptor = [0; 256];
    let mut control_buf = [0; 64];
    let builder = Builder::new(
        driver,
        config,
        &mut config_descriptor,
        &mut bos_descriptor,
        &mut [], // no msos descriptors
        &mut control_buf,
    );

    // Build the builder.
    let mut usb = builder.build();

    // Run the USB device.

    usb.run().await;
}
