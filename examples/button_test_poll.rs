// SPDX-FileCopyrightText: 2025 KOINSLOT Inc.
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! /examples/button_test_poll.rs
//! Example to test button polling library on the Kywy board.
//! Polls buttons every frame then updates the display.
//! Compile with: cargo build --example button_test_poll --target thumbv6m-none-eabi --release --features button-poll --no-default-features

#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use panic_probe as _;

use kywy::button_poll::ButtonId;
use kywy::{kywy_button_poll_from, kywy_display_from, kywy_spi_from, kywy_usb_from};

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};

use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::FONT_6X10},
    pixelcolor::BinaryColor,
    prelude::*,
    text::Text,
};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Starting button polling example");

    let p = embassy_rp::init(Default::default());

    // Initialize display
    kywy_spi_from!(p => spi_bus);
    kywy_display_from!(spi_bus, p => display);
    kywy_usb_from!(spawner, p);
    let style = MonoTextStyle::new(&FONT_6X10, BinaryColor::Off);

    display.initialize().await;
    display.enable();

    display.clear_buffer(BinaryColor::On);
    write_message(&mut display, "Polling Buttons", style);
    display.write_display().await;

    // Initialize buttons
    kywy_button_poll_from!(p => poller);

    // Loop to update button display
    loop {
        display.clear(BinaryColor::On).ok();
        write_message(&mut display, "Polling Buttons", style);

        let buttons = [
            ("Left", ButtonId::Left),
            ("Right", ButtonId::Right),
            ("Up", ButtonId::DUp),
            ("Down", ButtonId::DDown),
            ("D-Left", ButtonId::DLeft),
            ("D-Right", ButtonId::DRight),
            ("Center", ButtonId::DCenter),
        ];

        let y_offset = 20;

        for (i, (label, id)) in buttons.iter().enumerate() {
            let is_pressed = poller.is_pressed(*id);
            let state = if is_pressed { "Pressed" } else { "Released" };

            let y = y_offset + 12 + (i as i32) * 12;

            let _ = Text::new(label, Point::new(5, y), style).draw(&mut display);
            let _ = Text::new(state, Point::new(70, y), style).draw(&mut display);
        }

        display.write_display().await;
        Timer::after(Duration::from_millis(100)).await;
    }
}

/// Write a one-line message to the top of the screen
fn write_message<D: DrawTarget<Color = BinaryColor>>(
    display: &mut D,
    msg: &str,
    style: MonoTextStyle<BinaryColor>,
) {
    let _ = Text::new(msg, Point::new(5, 20), style).draw(display);
}
