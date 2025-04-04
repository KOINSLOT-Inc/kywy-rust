#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::FONT_6X10},
    pixelcolor::BinaryColor,
    prelude::*,
    text::Text,
};
use kywy::{kywy_battery_from, kywy_display_from}; // ← 🧠 Import the macros
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    kywy_display_from!(p => display);
    kywy_battery_from!(p => battery);

    display.initialize().await;
    display.enable();

    loop {
        info!("Top of loop");
        display.clear(BinaryColor::On).unwrap();

        Text::new(
            "Battery Status Test",
            Point::new(10, 20),
            MonoTextStyle::new(&FONT_6X10, BinaryColor::Off),
        )
        .draw(&mut display)
        .unwrap();

        battery.draw_async(&mut display).await.unwrap();

        display.write_display().await;
        Timer::after(Duration::from_secs(1)).await;
    }
}
