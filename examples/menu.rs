//! examples/menu.rs: Example of building menus with the Kywy device.
//! This example uses the embedded_menu library to display a simple dummy menu
//! Compile with: cargo build --example button_test_async --target thumbv6m-none-eabi --release

#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use panic_probe as _;

use kywy::button_async::{ButtonEvent, ButtonId, ButtonState};
use kywy::{kywy_button_async_from, kywy_display_from}; // Import the macros

use embassy_executor::Spawner;

use embedded_graphics::Drawable;
use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::FONT_6X10},
    pixelcolor::BinaryColor,
    prelude::*,
    text::Text,
};
use embedded_menu::{
    Menu, MenuStyle,
    interaction::programmed::Programmed,
    interaction::{Action, Interaction, Navigation},
    selection_indicator::style::Triangle,
};

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Button test example started");
    let p = embassy_rp::init(Default::default());

    // Initialize display
    info!("Initializing display");
    kywy_display_from!(p => display);
    display.initialize().await;
    display.enable();
    display.clear_buffer(BinaryColor::On);
    let style = MonoTextStyle::new(&FONT_6X10, BinaryColor::Off);
    write_message(&mut display, "Display Initialized", style);

    // Initialize buttons
    kywy_button_async_from!(&spawner, p => button_channel);

    // initialize menu with theme
    let menu_style = MenuStyle::new(BinaryColor::Off) // color should be off (black), the default is on and that is just white on white, that one really threw me for a loop there
        .with_font(&FONT_6X10) // set font size
        .with_selection_indicator(Triangle) // set indicator type
        .with_input_adapter(Programmed); // set input adapter type, Programmed is what we want for mapping button inputs

    let mut menu = Menu::with_style("Main Menu", menu_style) // here we define the actual menu layout, these dont actually do anything
        .add_item("Start", ">", |_| ()) // add item to menu, the first string is the name of the item, the second is the indicator, and the third is a closure that is called when the item is selected
        .add_item("Settings", false, |_| ()) // add item to menu, the first string is the name of the item, the second is the indicator, and the third is a closure that is called when the item is selected but this time with a boolean indicator
        .add_item("About", false, |_| ()) // same as last but says about, none of these actually do anything
        .build();
    display.clear_buffer(BinaryColor::On); // clear perevious message
    menu.update(&display); // update the menu with the init state
    menu.draw(&mut display).unwrap(); // write to display buffer
    display.write_display().await; // update display

    loop {
        let event: ButtonEvent = button_channel.receive().await; // wait for button event
        let interaction = match (event.id, event.state) {
            // match button event to desired input and map to a menu interaction
            (ButtonId::DUp, ButtonState::Pressed) => {
                Some(Interaction::Navigation(Navigation::Previous))
            }
            (ButtonId::DDown, ButtonState::Pressed) => {
                Some(Interaction::Navigation(Navigation::Next))
            }
            (ButtonId::Right, ButtonState::Pressed) => Some(Interaction::Action(Action::Select)),
            _ => None,
        };
        if let Some(i) = interaction {
            display.clear_buffer(BinaryColor::On);
            menu.interact(i); // sends the interaction to the menu system
            menu.update(&display); // update the menu with the new state
            menu.draw(&mut display).unwrap(); // draws the menu to display buffer
            display.write_display().await; // update display
        }
    }
}

fn write_message<D: DrawTarget<Color = BinaryColor>>(
    // this function is used to write a string to the display
    display: &mut D,
    msg: &str,
    style: MonoTextStyle<BinaryColor>,
) {
    display.clear(BinaryColor::On).ok(); // clear screen
    let _ = Text::new(msg, Point::new(5, 20), style).draw(display);
}
