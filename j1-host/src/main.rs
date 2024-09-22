mod usb_backend;
use crate::usb_backend::*;
use micropb::{MessageDecode, MessageEncode, PbDecoder, PbRead};
use rusb::Context;

mod example {
    #![allow(clippy::all)]
    #![allow(nonstandard_style, unused, irrefutable_let_patterns)]
    // Let's assume that Example is the only message define in the .proto file that has been
    // converted into a Rust struct
    include!(concat!(env!("OUT_DIR"), "/j1_proto.rs"));
}

const ACCEL_VID: u16 = 0xcafe;
const ACCEL_PID: u16 = 0xc0de;

fn main() {
    match Context::new() {
        Ok(mut context) => match open_device(&mut context, ACCEL_VID, ACCEL_PID) {
            Some((mut device, device_desc, mut handle)) => {
                read_device(&mut device, &device_desc, &mut handle).unwrap()
            }
            None => println!("could not find device {:04x}:{:04x}", ACCEL_VID, ACCEL_PID),
        },
        Err(e) => panic!("could not initialize libusb: {}", e),
    }
}
