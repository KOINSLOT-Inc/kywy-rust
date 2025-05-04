// SPDX-FileCopyrightText: 2025 KOINSLOT Inc.
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! examples/template.rs
//! Boilerplate for Kywy programs

#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use panic_probe as _;

// macros
use kywy::{
    kywy_button_async_from, kywy_button_poll_from, kywy_display_from, kywy_spi_from, kywy_usb_from,
};

// embassy requirements
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};

// Uncomment these common dependencies to use them in your program

// images, use include_bytes! to include images in your program
//use embedded_graphics::image::Image;
//use tinybmp::Bmp;

// structs
// use core::cell::Cell;
// use itoa::Buffer;
// use heapless::Vec;

// math
// use rand::{Rng, SeedableRng, rngs::SmallRng}; //random number generators
// use micromath::F32Ext; //fast math functions

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // initialize kywy
    let p = embassy_rp::init(Default::default()); //gets peripherals
    kywy_spi_from!(p => spi_bus); //create the spi bus instance, needed for display
    kywy_display_from!(spi_bus, p => display); //create the display instance
    kywy_usb_from!(spawner, p); //initialize usb monitor, needed for using default programming stuff
    kywy_button_async_from!(&spawner, p => button_channel); // async button channel, create a queue of button events in button_channel
    //kywy_button_poll_from!(p => poller); // button poller function, gets current state of the buttons

    // initialize display and turn it on
    display.initialize().await;
    display.enable();

    // Add your code here
}
