//! Hello World example for Kywy display
//! Status: Not Working (likely due to display.rs)
#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_executor;
use embassy_time::{Duration, Timer};
use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::FONT_6X10},
    pixelcolor::BinaryColor,
    prelude::*,
    text::Text,
};
use kywy::display::KywyDisplay;
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: embassy_executor::Spawner) {
    info!("Starting Kywy Hello World display!");

    let mut display = KywyDisplay::new().await;
    display.initialize().await; // This handles display enable/clear sequence
    display.enable(); // make sure display is turned on
    // Initial display test pattern
    display.clear_buffer(BinaryColor::On); // White background
    display.write_display().await;
    Timer::after(Duration::from_millis(1000)).await;

    display.clear_buffer(BinaryColor::Off); // Black background
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
    Timer::after(Duration::from_millis(5000)).await;

    // Draw text
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
