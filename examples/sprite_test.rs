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
use heapless::Vec;
use kywy::{
    button_async::{ButtonEvent, ButtonId, ButtonState},
    engine::sprite::{Animation, SpriteInstance, SpriteSheet},
    kywy_button_async_from, kywy_display_from, kywy_spi_from,
};
use panic_probe as _;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Sprite test with multi-button input");

    let p = embassy_rp::init(Default::default());
    kywy_spi_from!(p => spi_bus);
    kywy_display_from!(spi_bus, p => display);
    kywy_button_async_from!(&spawner, p => button_channel);

    display.initialize().await;
    display.enable();

    static SPRITE_DATA: &[u8] = include_bytes!("../examples/Art Assets/monsters/electric.bmp");
    let sheet = SpriteSheet::new(SPRITE_DATA, Size::new(64, 64)).unwrap();

    let idle: &[(u32, u32)] = &[(0, 0), (1, 0), (2, 0)];
    let left_trigger: &[(u32, u32)] = &[(0, 1), (1, 1), (2, 1)];
    let right_trigger: &[(u32, u32)] = &[(0, 2), (1, 2), (2, 2)];

    let mut animations: Vec<_, 4> = Vec::new();
    animations.push(Animation::new(&sheet, idle, true)).unwrap(); // index 0
    animations
        .push(Animation::new(&sheet, left_trigger, false))
        .unwrap(); // index 1
    animations
        .push(Animation::new(&sheet, right_trigger, false))
        .unwrap(); // index 2

    let mut sprite = SpriteInstance::new(animations, Point::new(40, 40));

    loop {
        // Draw current frame
        let frame = sprite.current().current_frame_sprite().unwrap();
        display.clear(BinaryColor::On).unwrap();
        frame.draw(&mut display, sprite.position).unwrap();
        display.write_display().await;

        // Advance and revert if animation ends
        sprite.update(0);

        // Handle input
        if let Ok(event) = button_channel.try_receive() {
            match event {
                ButtonEvent {
                    id: ButtonId::Left,
                    state: ButtonState::Pressed,
                } => sprite.trigger(1),
                ButtonEvent {
                    id: ButtonId::Right,
                    state: ButtonState::Pressed,
                } => sprite.trigger(2),
                _ => {}
            }
        }

        Timer::after(Duration::from_millis(100)).await;
    }
}
