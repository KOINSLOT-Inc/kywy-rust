// SPDX-FileCopyrightText: 2025 KOINSLOT Inc.
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! src/button_poll.rs
//! Button polling library for Kywy board (direct level check, no async)
//! Returns true (Pressed) or false (Released)
//! Requires feature 'button_poll'

use embassy_rp::PeripheralRef;
use embassy_rp::gpio::{Input, Level, Pull};
use embassy_rp::peripherals::*;

#[derive(Copy, Clone, Debug)]
pub enum ButtonId {
    Left,
    Right,
    DUp,
    DDown,
    DLeft,
    DRight,
    DCenter,
}

pub struct ButtonPins {
    pub left: PeripheralRef<'static, PIN_12>,
    pub right: PeripheralRef<'static, PIN_2>,
    pub dup: PeripheralRef<'static, PIN_9>,
    pub ddown: PeripheralRef<'static, PIN_3>,
    pub dleft: PeripheralRef<'static, PIN_6>,
    pub dright: PeripheralRef<'static, PIN_7>,
    pub dcenter: PeripheralRef<'static, PIN_8>,
}

pub struct ButtonPoller {
    left: Input<'static>,
    right: Input<'static>,
    dup: Input<'static>,
    ddown: Input<'static>,
    dleft: Input<'static>,
    dright: Input<'static>,
    dcenter: Input<'static>,
}

impl ButtonPoller {
    pub fn new(pins: ButtonPins) -> Self {
        fn mk_input<P: embassy_rp::gpio::Pin + 'static>(
            pin: PeripheralRef<'static, P>,
        ) -> Input<'static> {
            let mut input = Input::new(pin, Pull::Up);
            input.set_schmitt(true);
            input
        }

        Self {
            left: mk_input(pins.left),
            right: mk_input(pins.right),
            dup: mk_input(pins.dup),
            ddown: mk_input(pins.ddown),
            dleft: mk_input(pins.dleft),
            dright: mk_input(pins.dright),
            dcenter: mk_input(pins.dcenter),
        }
    }

    pub fn is_pressed(&self, id: ButtonId) -> bool {
        let level = match id {
            ButtonId::Left => self.left.get_level(),
            ButtonId::Right => self.right.get_level(),
            ButtonId::DUp => self.dup.get_level(),
            ButtonId::DDown => self.ddown.get_level(),
            ButtonId::DLeft => self.dleft.get_level(),
            ButtonId::DRight => self.dright.get_level(),
            ButtonId::DCenter => self.dcenter.get_level(),
        };

        level == Level::Low
    }

    pub fn poll_all(&self) -> u8 {
        let mut bits = 0u8;

        if self.is_pressed(ButtonId::Left) {
            bits |= 1 << 0;
        }
        if self.is_pressed(ButtonId::Right) {
            bits |= 1 << 1;
        }
        if self.is_pressed(ButtonId::DUp) {
            bits |= 1 << 2;
        }
        if self.is_pressed(ButtonId::DDown) {
            bits |= 1 << 3;
        }
        if self.is_pressed(ButtonId::DLeft) {
            bits |= 1 << 4;
        }
        if self.is_pressed(ButtonId::DRight) {
            bits |= 1 << 5;
        }
        if self.is_pressed(ButtonId::DCenter) {
            bits |= 1 << 6;
        }

        bits
    }
}
