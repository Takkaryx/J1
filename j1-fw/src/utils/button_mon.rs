use embassy_stm32::{exti::ExtiInput, gpio::Output};
use embassy_time::{Duration, Timer};
use defmt::*;

#[embassy_executor::task]
pub async fn button_task(mut led: Output<'static>, mut ext: ExtiInput<'static>) -> ! {
    loop {
        // Check if button got pressed
        ext.wait_for_rising_edge().await;
        led.toggle();
        info!("{:?} button press detected!", file!());
        Timer::after(Duration::from_millis(250)).await;
    }
}
