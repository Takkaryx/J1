// use defmt::*;
// // use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice;
// use embassy_embedded_hal;

// use crate::modules::lis302dl::Lis302Dl;

// #[derive(Copy, Clone, Debug)]
// pub enum AccelError<Dev, Rat>
// {
//     DevError(Dev),
//     RatError(Rat)
// }

// pub struct AccelMon <Dev, Rat> {
//     dev: Lis302Dl,
//     _dev_err: PhantomData<AccelError<Dev, Rat>>,
// }

// impl<DevError, RatError>  AccelMon <DevError, RatError> {
//     pub fn init(spi_bus: SpiBus, mut chip_select: OutputPin) -> Result<Self, AccelError<DevError, RatError>> {
//         chip_select.set_high().map_err(|e| AccelError::DevError(e));
//         let spi_device = ExclusiveDevice::new_no_delay(spi_bus, chip_select);
//         // let config = lis302dl::Config::default();
//         // let dev = Lis302Dl::new(spi_bus, cs, config).await;
//        Ok(Self{ spi: spi_bus, cs: chip_select, _spi_err: PhantomData, _cs_err: PhantomData})
//     }
// }

// #[embassy_executor::task]
// pub async fn accel_task() -> ! {
//     loop {
//         info!("accel task running");
// }
// }
