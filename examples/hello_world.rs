// SPDX-FileCopyrightText: 2025 KOINSLOT Inc.
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Hello World example for Kywy display

#![no_std]
#![no_main]
use defmt::*;
use defmt_rtt as _;
use embassy_time::{Duration, Timer};
use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::FONT_6X10},
    pixelcolor::BinaryColor,
    prelude::*,
    text::Text,
};
use kywy::kywy_display_from;
use kywy::kywy_spi_from;
use kywy::kywy_usb_from;
use panic_probe as _;

#[embassy_executor::main]
async fn main(spawner: embassy_executor::Spawner) {
    info!("Starting Kywy Hello World display!");

    let p = embassy_rp::init(Default::default());
    kywy_spi_from!(p => spi_bus);
    kywy_display_from!(spi_bus, p => display);
    kywy_usb_from!(spawner, p);

    // Black Screen
    display.clear_buffer(BinaryColor::Off); // Clear buffer to black
    display.write_display().await;
    Timer::after(Duration::from_millis(1000)).await;

    // White Screen
    display.clear_buffer(BinaryColor::On); // Clear buffer to white
    display.write_display().await;
    Timer::after(Duration::from_millis(1000)).await;

    // Checkerboard pattern
    for y in 0..display.size().height as usize {
        for x in 0..display.size().width as usize {
            let on = (x + y) % 2 == 0;
            display.set_pixel(
                x,
                y,
                if on {
                    BinaryColor::On
                } else {
                    BinaryColor::Off
                },
            );
        }
    }
    display.write_display().await;
    Timer::after(Duration::from_millis(1000)).await;

    // Write Hello, Rust! to display using the embedded-graphics crate
    display.clear_buffer(BinaryColor::On); // Clear to white background
    let style = MonoTextStyle::new(&FONT_6X10, BinaryColor::Off); // Black text

    Text::new("Hello, Rust!", Point::new(10, 20), style)
        .draw(&mut display)
        .unwrap();

    display.write_display().await;

    // Main loop - toggle VCOM every second to prevent DC bias
    loop {
        display.toggle_vcom().await;
        Timer::after(Duration::from_secs(1)).await;
    }
}
