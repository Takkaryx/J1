#![allow(unused_variables)]
#![allow(dead_code)]
use embedded_hal_async::spi;
use defmt::*;

const WHO_AM_I: u8 = 0x0F;
const DEVICE_NAME: u8 = 0x3F;

const CTRL_REG1: u8 = 0x20;
const CTRL_REG2: u8 = 0x21;
const CTRL_REG3: u8 = 0x22;

const HP_FILTER_RESET: u8 = 0x23;
const STATUS_REG: u8 = 0x27;
const OUT_X: u8 = 0x29;
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

const DATA_RATE_100_HZ: u8 = 0x00;
const DATA_RATE_400_HZ: u8 = 0x80;
const POWER_DOWN_MODE: u8 = 0x00;
const ACTIVE_MODE: u8 = 0x40;
const SCALE_PLUS_MINUS_2G: u8 = 0x00;
const SCALE_PLUS_MINUS_8G: u8 = 0x20;
const Z_ENABLE: u8 = 0x04;
const Y_ENABLE: u8 = 0x02;
const X_ENABLE: u8 = 0x01;

const READ_FLAG: u8 = 0x80;

const SCALE: f32 = 4.6 / 256.0; // When multiplied by the output give the acceleration in g's

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

pub struct Lis302Dl<SPI, SpiError>
where SPI: spi::SpiDevice<Error=SpiError>
{
    spi: SPI,
    config: Config,
}

impl<SPI, SpiError> Lis302Dl<SPI, SpiError>
where SPI: spi::SpiDevice<Error=SpiError>
{
    pub fn new(spi: SPI, config: Config) -> Self {
        Self {spi, config}
    }

    pub async fn init(&mut self) -> Result<(), Lis302dlError<SPI>> {
         let name = self.read_reg(WHO_AM_I).await?;
        info!("Device name read is: {:?}", name);
        if name != DEVICE_NAME {
            error!("Lis302DL has incorrect name!");
        }

        self.set_config().await?;
        Ok(())
    }
    
    async fn read_reg(&mut self, reg_address: u8) -> Result<u8, Lis302dlError<SPI>> {
        let mut reg_data: [u8; 2] = [0; 2];
        let _ = self.spi.transfer(&mut reg_data, &[reg_address | 1 << 7, 0]).await.map_err(|e| Lis302dlError::SpiError(e));
        debug!("Reg data is: {:?}", reg_data[1]);
        Ok(reg_data[1])
    }
    
    async fn write_reg(&mut self, reg_address: u8, val: u8) -> Result<(), Lis302dlError<SPI>> {
        // let _ = self.spi.write(&[reg_address, val]).await?;
        let _ = self.spi.write(&[reg_address, val]).await.map_err(|e| Lis302dlError::SpiError(e));
        Ok(())
    }

    pub async fn read_accel(&mut self) -> Result<(), Lis302dlError<SPI>> {
        let x = self.read_reg(OUT_X).await?;
        let y = self.read_reg(OUT_Y).await?;
        let z = self.read_reg(OUT_Z).await?;
        info!("x DATA is {:?}", x);
        info!("y DATA is {:?}", y);
        info!("z DATA is {:?}", z);
        Ok(())
    }

    async fn set_config(&mut self) -> Result<(), Lis302dlError<SPI>> {
        let mut control_byte = X_ENABLE | Y_ENABLE | Z_ENABLE;
        control_byte |= match self.config.power_mode {
            PowerMode::Active => ACTIVE_MODE,
            PowerMode::Powerdown => POWER_DOWN_MODE,
        };
        control_byte |= match self.config.scale {
            Scale::TwoG=> SCALE_PLUS_MINUS_2G,
            Scale::EightG=> SCALE_PLUS_MINUS_8G,
        };
        control_byte |= match self.config.data_rate {
            DataRate::Rate100Hz => DATA_RATE_100_HZ,
            DataRate::Rate400Hz => DATA_RATE_400_HZ,
        };
        let _ = self.write_reg(CTRL_REG1, control_byte).await;
        Ok(())
    }
}

#[derive(Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Lis302dlError<Spi> {
    SpiError(Spi)
}

impl<SpiError> Lis302dlError<SpiError> {
    fn message(&self) -> &'static str {
        match *self {
            Lis302dlError::SpiError(_) => {
                "An error occured while attempting to reach the Lis302dl over SPI."
            }
        }
    }
}

// implicit `?` syntax for SpiError to Lis302dlError
impl<SpiError> core::convert::From<SpiError> for Lis302dlError<SpiError> {
    fn from(value: SpiError) -> Self {
        Lis302dlError::SpiError(value)
    }
}
