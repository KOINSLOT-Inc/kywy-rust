// SPDX-FileCopyrightText: 2025 KOINSLOT Inc.
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! This example demonstrates the use of the `Sprite` struct from the `kywy` crate.
//! It creates a sprite from a sprite sheet and animates it on a display.
//! This only uses sprites.rs and no additional engine components.

#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use embedded_graphics::{pixelcolor::BinaryColor, prelude::*};
use kywy::{
    engine::sprite::{Animation, SpriteSheet},
    kywy_display_from, kywy_spi_from,
};
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Sprite test starting");

    let p = embassy_rp::init(Default::default());

    kywy_spi_from!(p => spi_bus);
    kywy_display_from!(spi_bus, p => display);
    display.initialize().await;
    display.enable();

    static SPRITE_DATA: &[u8] = include_bytes!("../examples/Art Assets/monsters/electric.bmp"); // pulls sprite sheet image

    let sheet = SpriteSheet::new(SPRITE_DATA, Size::new(64, 64)).unwrap(); // defines the sprite sheet from raw image

    let frames: &[(u32, u32)] = &[(0, 0), (1, 0), (2, 0)]; // defines the frames of the animation from the sprite sheet
    let mut animation = Animation::new(&sheet, frames, true); // creates an animation from the frames

    loop {
        // let (fx, fy) = animation.current_frame_loc(); // get coordinates of the current frame
        // let sprite = sheet.sprite(fx, fy).unwrap(); // get the sprite at the current frame
        let sprite = animation.current_frame_sprite().unwrap();

        display.clear(BinaryColor::On).unwrap();
        sprite.draw(&mut display, Point::new(40, 40)).unwrap(); // draw the sprite we just got for this frame
        display.write_display().await;

        animation.advance();
        Timer::after(Duration::from_millis(200)).await;
    }
}
