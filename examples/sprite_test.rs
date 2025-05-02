// SPDX-FileCopyrightText: 2025 KOINSLOT Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! This example demonstrates the use of the `Sprite` struct from the `kywy` crate.
//! It creates a sprite from a sprite sheet and animates it on a display.
//! This only uses sprites.rs and no additional engine components to create animations with button events and move a sprite around with the dpad.

#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use embedded_graphics::{pixelcolor::BinaryColor, prelude::*};
use heapless::Vec;
use kywy::{
    button_async::{ButtonId, ButtonState},
    engine::sprite::{Animation, Rotation, SpriteInstance, SpriteOptions, SpriteSheet},
    kywy_button_async_from, kywy_display_from, kywy_spi_from, kywy_usb_from,
};
use panic_probe as _;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Sprite D-Pad control test");

    let p = embassy_rp::init(Default::default());
    kywy_spi_from!(p => spi_bus);
    kywy_display_from!(spi_bus, p => display);
    kywy_button_async_from!(&spawner, p => button_channel);
    kywy_usb_from!(spawner, p);

    display.initialize().await;
    display.enable();

    static SPRITE_DATA: &[u8] = include_bytes!("../examples/Art Assets/monsters/electric.bmp");
    let sheet = SpriteSheet::new(SPRITE_DATA, Size::new(64, 64)).unwrap();

    let idle: &[(u32, u32)] = &[(0, 0), (1, 0), (2, 0)];
    let left_trigger: &[(u32, u32)] = &[(0, 1), (1, 1), (2, 1)];
    let right_trigger: &[(u32, u32)] = &[(0, 2), (1, 2), (2, 2)];

    let mut animations: Vec<_, 4> = Vec::new();
    animations.push(Animation::new(&sheet, idle, true)).unwrap();
    animations
        .push(Animation::new(&sheet, left_trigger, false))
        .unwrap();
    animations
        .push(Animation::new(&sheet, right_trigger, false))
        .unwrap();

    let mut sprite = SpriteInstance::new(animations, Point::new(40, 40));
    let mut sprite_options = SpriteOptions {
        flip_x: false,
        flip_y: false,
        rotation: Rotation::None,
    };

    let mut velocity = Point::zero();

    loop {
        // Capture all button events
        while let Ok(event) = button_channel.try_receive() {
            match event.state {
                ButtonState::Pressed => match event.id {
                    ButtonId::Left => sprite.trigger(1),
                    ButtonId::Right => sprite.trigger(2),
                    ButtonId::DLeft => {
                        velocity.x = -5;
                        sprite_options.flip_x = true;
                    }
                    ButtonId::DRight => {
                        velocity.x = 5;
                        sprite_options.flip_x = false;
                    }
                    ButtonId::DUp => velocity.y = -5,
                    ButtonId::DDown => velocity.y = 5,
                    _ => {}
                },
                ButtonState::Released => match event.id {
                    ButtonId::DLeft | ButtonId::DRight => velocity.x = 0,
                    ButtonId::DUp | ButtonId::DDown => velocity.y = 0,
                    _ => {}
                },
            }
        }

        // Move sprite by velocity
        sprite.move_by(velocity.x, velocity.y);

        // Draw current frame
        let frame = sprite.current().current_frame_sprite().unwrap();
        display.clear(BinaryColor::On).unwrap();
        frame
            .draw(&mut display, sprite.position, sprite_options)
            .unwrap();
        display.write_display().await;

        // Update animation
        sprite.update(0);

        Timer::after(Duration::from_millis(100)).await;
    }
}
