use chrono::NaiveDateTime;
use core::borrow::Borrow;
use core::cell::RefCell;
use embassy_stm32::peripherals::RTC;
use embassy_stm32::rtc::Rtc;
use embassy_stm32::rtc::RtcConfig;
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::blocking_mutex::Mutex;

// Define the global state structure
pub struct LocalRtc {
    init_time: NaiveDateTime,
    rtc: Rtc,
}

// Atomic flag to ensure initialization happens only once
pub static LOCAL_RTC: Mutex<ThreadModeRawMutex, RefCell<Option<LocalRtc>>> =
    Mutex::new(RefCell::new(None));

pub fn init(rtc_periph: RTC) {
    // Perform the initialization
    let rtc_config = RtcConfig::default();
    let rtc = Rtc::new(rtc_periph, rtc_config);
    let init_time: NaiveDateTime = rtc.now().unwrap().into();
    let local_rtc = LocalRtc { init_time, rtc };

    LOCAL_RTC.lock(|f| {
        f.replace(Some(local_rtc));
    });
}

pub fn get_ticks() -> NaiveDateTime {
    LOCAL_RTC.lock(|f| f.borrow().as_ref().unwrap().get_ticks())
}

pub fn get_ticks_since_boot() -> i64 {
    LOCAL_RTC.lock(|f| {
        let binding = f.borrow();
        let rtc = binding.as_ref().unwrap();
        let now = rtc.get_ticks();
        (now - rtc.init_time).num_milliseconds()
    })
}

impl LocalRtc {
    pub fn get_ticks(&self) -> NaiveDateTime {
        self.rtc.borrow().now().expect("Failed to get time").into()
    }
}
