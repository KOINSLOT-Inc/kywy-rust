//! examples/button_test.rs: Example to test button library on the Kywy board.
//! Probably dont actually update the display on every button press, but this is just a test and I wanted to show that the commands are all processed.
//!
//! Compile with: cargo build --example button_test_async --target thumbv6m-none-eabi --release --features button-async --no-default-features

#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use panic_probe as _;

use kywy::button_async::{ButtonEvent, ButtonId, ButtonState};
use kywy::{kywy_button_async_from, kywy_display_from}; // Import the macros

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};

use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::FONT_6X10},
    pixelcolor::BinaryColor,
    prelude::*,
    text::Text,
};

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Button test example started");
    let p = embassy_rp::init(Default::default());

    // Initialize display
    info!("Initializing display");
    kywy_display_from!(p => display);
    let style = MonoTextStyle::new(&FONT_6X10, BinaryColor::Off);
    display.initialize().await;
    display.enable();
    display.clear_buffer(BinaryColor::On);
    write_message(&mut display, "Button Test Started", style);
    display.write_display().await;

    // Initialize buttons
    info!("Initializing buttons");
    kywy_button_async_from!(&spawner, p => button_channel);
    info!("Buttons initialized");

    display.clear(BinaryColor::Off).ok();
    write_message(&mut display, "Buttons Initialized", style);
    display.write_display().await;

    loop {
        let event: ButtonEvent = button_channel.receive().await;

        let msg = match (event.id, event.state) {
            (ButtonId::Left, ButtonState::Pressed) => "Left Pressed",
            (ButtonId::Left, ButtonState::Released) => "Left Released",
            (ButtonId::Right, ButtonState::Pressed) => "Right Pressed",
            (ButtonId::Right, ButtonState::Released) => "Right Released",
            (ButtonId::DUp, ButtonState::Pressed) => "D-Up Pressed",
            (ButtonId::DUp, ButtonState::Released) => "D-Up Released",
            (ButtonId::DDown, ButtonState::Pressed) => "D-Down Pressed",
            (ButtonId::DDown, ButtonState::Released) => "D-Down Released",
            (ButtonId::DLeft, ButtonState::Pressed) => "D-Left Pressed",
            (ButtonId::DLeft, ButtonState::Released) => "D-Left Released",
            (ButtonId::DRight, ButtonState::Pressed) => "D-Right Pressed",
            (ButtonId::DRight, ButtonState::Released) => "D-Right Released",
            (ButtonId::DCenter, ButtonState::Pressed) => "D-Center Pressed",
            (ButtonId::DCenter, ButtonState::Released) => "D-Center Released",
        };

        info!("{}", msg);
        write_message(&mut display, msg, style);
        display.write_display().await;

        Timer::after(Duration::from_millis(100)).await;
    }
}

fn write_message<D: DrawTarget<Color = BinaryColor>>(
    display: &mut D,
    msg: &str,
    style: MonoTextStyle<BinaryColor>,
) {
    display.clear(BinaryColor::On).ok(); // clear screen
    let _ = Text::new(msg, Point::new(5, 20), style).draw(display);
}
