// SPDX-FileCopyrightText: 2025 KOINSLOT Inc.
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! src/buttons.rs
//! Button event system using a shared Channel for press/release detection.

use embassy_executor::Spawner;
use embassy_rp::PeripheralRef;
use embassy_rp::gpio::{Input, Level, Pull};
use embassy_rp::peripherals::*;
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::Channel;

#[derive(Clone, Copy, Debug)]
pub enum ButtonId {
    Left,
    Right,
    DUp,
    DDown,
    DLeft,
    DRight,
    DCenter,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ButtonState {
    Pressed,
    Released,
}

#[derive(Clone, Copy, Debug)]
pub struct ButtonEvent {
    pub id: ButtonId,
    pub state: ButtonState,
}

const BUTTON_CHANNEL_CAPACITY: usize = 16;

static BUTTON_CHANNEL: Channel<ThreadModeRawMutex, ButtonEvent, BUTTON_CHANNEL_CAPACITY> =
    Channel::new();

pub struct ButtonPins {
    pub left: PeripheralRef<'static, PIN_12>,
    pub right: PeripheralRef<'static, PIN_2>,
    pub dup: PeripheralRef<'static, PIN_9>,
    pub ddown: PeripheralRef<'static, PIN_3>,
    pub dleft: PeripheralRef<'static, PIN_6>,
    pub dright: PeripheralRef<'static, PIN_7>,
    pub dcenter: PeripheralRef<'static, PIN_8>,
}

pub fn init(
    spawner: &Spawner,
    pins: ButtonPins,
) -> &'static Channel<ThreadModeRawMutex, ButtonEvent, BUTTON_CHANNEL_CAPACITY> {
    spawn_button(spawner, pins.left, ButtonId::Left);
    spawn_button(spawner, pins.right, ButtonId::Right);
    spawn_button(spawner, pins.dup, ButtonId::DUp);
    spawn_button(spawner, pins.ddown, ButtonId::DDown);
    spawn_button(spawner, pins.dleft, ButtonId::DLeft);
    spawn_button(spawner, pins.dright, ButtonId::DRight);
    spawn_button(spawner, pins.dcenter, ButtonId::DCenter);

    &BUTTON_CHANNEL
}

fn spawn_button<P: embassy_rp::gpio::Pin + 'static>(
    spawner: &Spawner,
    pin: PeripheralRef<'static, P>,
    id: ButtonId,
) {
    let mut input = Input::new(pin, Pull::Up);
    input.set_schmitt(true);
    if let Err(_e) = spawner.spawn(button_task(input, id)) {
        defmt::error!("Failed to spawn button task");
    }
}

#[embassy_executor::task(pool_size = 7)] // spawns 7 tasks, one for each button
async fn button_task(mut pin: Input<'static>, id: ButtonId) {
    loop {
        pin.wait_for_any_edge().await;
        let level = pin.get_level();

        let state = if level == Level::Low {
            ButtonState::Pressed
        } else {
            ButtonState::Released
        };

        let _ = BUTTON_CHANNEL.try_send(ButtonEvent { id, state });
    }
}
