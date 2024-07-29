use embassy_stm32::{exti::{ExtiInput, AnyChannel}, gpio::{Pull, AnyPin, Input, Level, Output, Speed}};
use embassy_time::{Duration, Timer};
use defmt::*;

pub struct ButtonMon<'a>{
    led: Output<'a, AnyPin>,
    ext: ExtiInput<'a, AnyPin>,
}

impl ButtonMon<'_> {
    pub fn init(led: AnyPin, button: AnyPin, int: AnyChannel) -> ButtonMon<'static> {
        let led = Output::new(led, Level::Low, Speed::Low);
        let button = Input::new(button, Pull::None);
        let ext= ExtiInput::new(button, int);
        ButtonMon {
            led, ext
        }
    }
}

#[embassy_executor::task]
pub async fn button_task(mut b: ButtonMon<'static>) -> ! {
    loop {
        // Check if button got pressed
        b.ext.wait_for_rising_edge().await;
        b.led.toggle();
        info!("{:?} button press detected!", file!());
        Timer::after(Duration::from_millis(250)).await;
    }
}
