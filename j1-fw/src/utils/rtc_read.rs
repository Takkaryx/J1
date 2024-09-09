use chrono::NaiveDateTime;
use core::borrow::BorrowMut;
use core::cell::RefCell;
use core::sync::atomic::{AtomicBool, Ordering};
use defmt::panic;
use embassy_stm32::peripherals::RTC;
use embassy_stm32::rtc::Rtc;
use embassy_stm32::rtc::RtcConfig;
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::blocking_mutex::Mutex;

// Define the global state structure
pub struct LocalRtc {
    rtc: RefCell<Rtc>,
}

// Atomic flag to ensure initialization happens only once
static INITIALIZED: AtomicBool = AtomicBool::new(false);
pub static LOCAL_RTC: Mutex<ThreadModeRawMutex, Option<LocalRtc>> = Mutex::new(None);

pub fn init(rtc_periph: RTC) {
    // Ensure initialization only happens once
    if INITIALIZED
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_ok()
    {
        return;
    }

    // Perform the initialization
    let rtc_config = RtcConfig::default();
    let rtc = Rtc::new(rtc_periph, rtc_config);
    let local_rtc = LocalRtc {
        rtc: RefCell::new(rtc),
    };

    LOCAL_RTC.lock(|mut f| {
        let mut val = f.borrow_mut().as_ref();
        val.replace(&local_rtc);
    });
}

pub fn get_ticks() -> NaiveDateTime {
    LOCAL_RTC.lock(|f| {
        let ticks = f.as_ref().unwrap().get_ticks();
        return ticks;
    });
    panic!("RTC problems");
}

impl LocalRtc {
    pub fn get_ticks(&self) -> NaiveDateTime {
        self.rtc.borrow().now().expect("Failed to get time").into()
    }
}
