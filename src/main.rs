#![no_std]
#![no_main]

use cortex_m::asm;
use cortex_m::peripheral::Peripherals;
use cortex_m_rt::entry;
use panic_halt as _;

#[entry]
fn main() -> ! {
    asm::nop();
    let peripherals = Peripherals::take().unwrap();

    loop {}
}
