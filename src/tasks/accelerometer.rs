use embassy_stm32::spi::Spi;
use defmt::*;

pub struct AccelMon<'a>{
    spi_bus: Spi<'a, Spi>
}

impl AccelMon<'_> {
    pub fn init(spi_bus: Spi) -> AccelMon<'static> {
        AccelMon {
            spi_bus
    }
    }

}

#[embassy_executor::task]
pub async fn accel_task(mut a: AccelMon<'static>) -> ! {
    loop {
        info!("accel task running");
}
}
