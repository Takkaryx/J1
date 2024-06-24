use core::marker::PhantomData;
use embedded_hal::spi;
use defmt::*;

const WHO_AM_I: u8 = 0x0F;
const DEVICE_NAME: u8 = 0x3B;

const CTRL_REG1: u8 = 0x20;
const CTRL_REG2: u8 = 0x21;
const CTRL_REG3: u8 = 0x22;

const HP_FILTER_RESET: u8 = 0x23;
const STATUS_REG: u8 = 0x27;
const T_X: u8 = 0x29;
const OUT_Y: u8 = 0x2B;
const OUT_Z: u8 = 0x2D;
const FF_WU_CFG_1: u8 = 0x30;
const FF_WU_SRC_1: u8 = 0x31;
const FF_WU_THS_1: u8 = 0x32;
const FF_WU_DURATION_1: u8 = 0x33;
const FF_WU_CFG_2: u8 = 0x34;
const FF_WU_SRC_2: u8 = 0x35;
const FF_WU_THS_2: u8 = 0x36;
const WU_DURATION_2: u8 = 0x37;
const CLICK_CFG: u8 = 0x38;
const CLICK_SRC: u8 = 0x39;
const CLICK_THSY_X: u8 = 0x3B;
const CLICK_THSZ: u8 = 0x3C;
const CLICK_TIMELIMIT: u8 = 0x3D;
const CLICK_LATENCY: u8 = 0x3E;
const CLICK_WINDOW: u8 = 0x3F;

#[derive(Clone, Debug, PartialEq)]
pub enum PowerMode {
    Active,
    Powerdown,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Scale {
    TwoG,
    EightG,
}

#[derive(Clone, Debug, PartialEq)]
pub enum DataRate {
    Rate100Hz,
    Rate400Hz,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Config {
    pub power_mode: PowerMode,
    pub scale: Scale,
    pub data_rate: DataRate,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            power_mode: PowerMode::Active,
            scale: Scale::TwoG,
            data_rate: DataRate::Rate400Hz,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Lis302Dl<Spi, SpiError>
where Spi: spi::SpiDevice
{
    spi: Spi,
    config: Config,
    _spi_err: PhantomData<SpiError>,
}

impl<Spi, SpiError> Lis302Dl<Spi, SpiError>
where Spi: spi::SpiDevice
{
    pub async fn new(spi: Spi, config: Config) -> Self {
        let mut lis302dl = Lis302Dl { spi, config, _spi_err: PhantomData};

        let name = lis302dl.read_reg(WHO_AM_I).await.unwrap();
        info!("Device name read is: {:?}", name);
        if name != DEVICE_NAME {
            // TODO: error
        }

        lis302dl.set_config();

        lis302dl
    }

    async fn read_reg(&mut self, reg_address: u8) -> Result<u8, Lis302dlError<SpiError>> {
        let mut reg_data: [u8; 1] = [0; 1];
        self.spi
            .transaction(&mut [
                spi::Operation::Write(&[reg_address]),
                spi::Operation::Read(&mut reg_data),
            ])
            .map_err(|e| Lis302dlError::SpiError(e));

        Ok(reg_data[0])
    }

    fn set_config(&mut self) -> Result<(), embassy_stm32::spi::Error> {
        Ok(())
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Lis302dlError<Spi> {
    SpiError(Spi)
}
impl core::fmt::Debug for Lis302dlError<Spi> {
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        fmt.debug_struct("Lis302DlError")
            // ...
            .field("SP", &format_args!("{:X}", self.context.sp))
            .field("KSP", &format_args!("{:X}", self.context.ksp))
            // ...
            .finish()
    }
}
