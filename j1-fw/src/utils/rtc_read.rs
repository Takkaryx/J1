use chrono::{NaiveDate, NaiveDateTime};
use defmt::info;
use embassy_stm32::peripherals::RTC;
use embassy_stm32::rtc::*;
// use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, mutex::Mutex};
use static_cell::StaticCell;

static LOCAL_RTC: StaticCell<Rtc> = StaticCell::new();

pub fn rtc_init(rtc_periph: RTC) {
    let rtc_config = embassy_stm32::rtc::RtcConfig::default();
    let rtc = embassy_stm32::rtc::Rtc::new(rtc_periph, rtc_config);
    let now: NaiveDateTime = rtc.now().unwrap().into();
    LOCAL_RTC.init(rtc);

    info!("Initializing RTC at {:?}", now.and_utc().timestamp());
}

#[allow(dead_code)]
pub fn read_rtc() -> NaiveDateTime {
    let now = NaiveDate::from_ymd_opt(2020, 5, 15)
        .unwrap()
        .and_hms_opt(10, 30, 15)
        .unwrap();
    now.into()
}
