mod usb_backend;

use micropb::{MessageDecode, PbDecoder};
use usb_backend::setup_hotplug;

use std::io;
use std::time::Duration;

mod proto {
    #![allow(clippy::all)]
    #![allow(nonstandard_style, unused, irrefutable_let_patterns)]
    // Let's assume that Example is the only message define in the .proto file that has been
    // converted into a Rust struct
    include!(concat!(env!("OUT_DIR"), "/j1_proto.rs"));
}
use proto::accel_::*;

fn main() {
    let _ = setup_hotplug();
    let baud_rate: u32 = 115200;
    let port_name = "/dev/tty.usbmodem123456781";

    let port = serialport::new(port_name, baud_rate)
        .timeout(Duration::from_millis(10))
        .open();

    match port {
        Ok(mut port) => {
            let mut serial_buf: Vec<u8> = vec![0; 1000];
            println!("Receiving data on {} at {} baud:", &port_name, &baud_rate);
            loop {
                match port.read(serial_buf.as_mut_slice()) {
                    Ok(_) => {
                        let mut decoder = PbDecoder::new(serial_buf.as_slice());
                        let mut pkt_out = Accel::default();
                        pkt_out.decode_len_delimited(&mut decoder).unwrap();
                        println!(
                            "x: {}, y: {}, z: {}, timestamp: {}",
                            pkt_out.x_accel, pkt_out.y_accel, pkt_out.z_accel, pkt_out.time
                        );
                    }
                    Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
                    Err(e) => eprintln!("{:?}", e),
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to open \"{}\". Error: {}", port_name, e);
            ::std::process::exit(1);
        }
    }
}
