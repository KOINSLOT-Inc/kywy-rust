#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use panic_probe as _;

use kywy::{kywy_buttons_from, kywy_display_from};

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

    // Initialize display and write hello message
    info!("Initializing display");
    kywy_display_from!(p => display);
    let style = MonoTextStyle::new(&FONT_6X10, BinaryColor::Off);
    display.initialize().await;
    display.enable();
    display.clear_buffer(BinaryColor::On);
    write_message(&mut display, "Button Test Started", style);
    display.write_display().await;
    info!("Display initialized");

    // Initialize buttons
    info!("Initializing buttons");
    kywy_buttons_from!(&spawner, p => buttons);
    info!("Buttons initialized");

    // Send message to display
    display.clear(BinaryColor::Off).ok();
    write_message(&mut display, "Buttons Initialized", style);
    display.write_display().await;

    loop {
        info!("Start of main loop");
        if !buttons.left.wait().await {
            info!("Left pressed");
            write_message(&mut display, "Left Pressed", style);
        } else if !buttons.right.wait().await {
            info!("Right pressed");
            write_message(&mut display, "Right Pressed", style);
        } else if !buttons.dup.wait().await {
            info!("Up pressed");
            write_message(&mut display, "Up Pressed", style);
        } else if !buttons.ddown.wait().await {
            info!("Down pressed");
            write_message(&mut display, "Down Pressed", style);
        } else if !buttons.dleft.wait().await {
            info!("Left D-pad");
            write_message(&mut display, "D-Left Pressed", style);
        } else if !buttons.dright.wait().await {
            info!("Right D-pad");
            write_message(&mut display, "D-Right Pressed", style);
        } else if !buttons.dcenter.wait().await {
            info!("Center pressed");
            write_message(&mut display, "Center Pressed", style);
        }

        display.write_display().await;
        Timer::after(Duration::from_millis(300)).await;
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
