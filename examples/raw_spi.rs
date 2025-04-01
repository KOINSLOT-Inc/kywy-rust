//! This example shows sending raw spi commands using the kywy spi interface (same as display)

#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_executor;
use embassy_time::{Duration, Timer};
use kywy::display::KywyDisplay;
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: embassy_executor::Spawner) {
    info!("Starting Kywy raw_spi!");

    let mut display = KywyDisplay::new().await;
    display.enable(); // make sure display is turned on
    loop {
        display.write_spi(&[0x20]).await; // toggle vcom only
        Timer::after(Duration::from_secs(1)).await;
        display.write_spi(&[0x00]).await;
        Timer::after(Duration::from_secs(1)).await;
    }
}
