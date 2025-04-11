#![no_std]
#![no_main]

use core::fmt::Write as Write2; // ← required for core::write! to work
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::FONT_6X10},
    pixelcolor::BinaryColor,
    prelude::*,
    text::Text,
};
use heapless::String;
use kywy::battery::BatteryStatus;
use kywy::{kywy_battery_from, kywy_display_from, kywy_spi_from};
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    kywy_spi_from!(p => spi_bus);
    kywy_display_from!(spi_bus, p => display);
    kywy_battery_from!(p => battery);

    display.initialize().await;
    display.enable();

    let text_style = MonoTextStyle::new(&FONT_6X10, BinaryColor::Off);

    loop {
        display.clear(BinaryColor::On).unwrap();

        // Draw title
        Text::new("Battery Status Test", Point::new(10, 40), text_style)
            .draw(&mut display)
            .unwrap();

        // Read and display voltage
        let voltage_mv = battery.read_voltage_mv().await;
        let mut buf: String<32> = String::new();
        write!(&mut buf, "Voltage: {} mV", voltage_mv).unwrap();

        Text::new(&buf, Point::new(10, 60), text_style)
            .draw(&mut display)
            .unwrap();

        let status = battery.status();
        let mut status_buf: String<32> = String::new();
        let status_str = match status {
            BatteryStatus::Charging => "Charging",
            BatteryStatus::Charged => "Charged",
            BatteryStatus::NotCharging => "Not Charging",
        };
        core::write!(&mut status_buf, "Status: {}", status_str).unwrap();

        Text::new(&status_buf, Point::new(10, 80), text_style)
            .draw(&mut display)
            .unwrap();

        battery.draw_async(&mut display).await.unwrap();

        display.write_display().await;
        Timer::after(Duration::from_secs(1)).await;
    }
}
