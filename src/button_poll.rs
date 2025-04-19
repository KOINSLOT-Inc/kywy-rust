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
    pub fn new(
        pin_left: PeripheralRef<'static, PIN_2>,
        pin_right: PeripheralRef<'static, PIN_12>,
        pin_dup: PeripheralRef<'static, PIN_9>,
        pin_ddown: PeripheralRef<'static, PIN_3>,
        pin_dleft: PeripheralRef<'static, PIN_6>,
        pin_dright: PeripheralRef<'static, PIN_7>,
        pin_dcenter: PeripheralRef<'static, PIN_8>,
    ) -> Self {
        fn mk_input<P: embassy_rp::gpio::Pin + 'static>(pin: PeripheralRef<'static, P>) -> Input<'static> {
            let mut input = Input::new(pin, Pull::Up);
            input.set_schmitt(true);
            input
        }

        Self {
            left: mk_input(pin_left),
            right: mk_input(pin_right),
            dup: mk_input(pin_dup),
            ddown: mk_input(pin_ddown),
            dleft: mk_input(pin_dleft),
            dright: mk_input(pin_dright),
            dcenter: mk_input(pin_dcenter),
        }
    }

    /// Returns `true` if the button is pressed (level is Low)
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

    /// Returns a compact bitfield of all button states
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
