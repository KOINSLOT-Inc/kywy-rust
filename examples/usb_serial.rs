// SPDX-FileCopyrightText: 2025 KOINSLOT Inc.
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! examples/usb_serial.rs: Example to test serial communication on the Kywy board.

#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use panic_probe as _;

use kywy::kywy_usb_from;
use kywy::usb::{usb_println, usb_read_line};

use core::fmt::Write;
use embassy_executor::Spawner;
use embassy_time::Timer;
use heapless::String; // Needed for heapless::String

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("usb_serial.rs started");

    let p = embassy_rp::init(Default::default());
    kywy_usb_from!(spawner, p);

    Timer::after_millis(500).await; // wait for USB to initialize
    loop {
        usb_println("Enter your name:").await;

        let mut buf = [0u8; 64]; // buffer to store data
        let len = usb_read_line(&mut buf).await; // read line from USB

        let name = core::str::from_utf8(&buf[..len])
            .unwrap_or("stranger")
            .trim_end();

        let mut line: String<64> = String::new(); // initialize heapless::String
        let _ = core::write!(line, "Hello, {}!", name); // write formatted string to heapless::String

        usb_println(&line).await; // print hello message
    }
}
