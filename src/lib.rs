#![no_std]

// SPDX-FileCopyrightText: 2025 KOINSLOT Inc.
//
// SPDX-License-Identifier: GPL-3.0-or-later

pub mod battery;
pub mod display;
pub mod sdcard;

#[macro_use]
pub mod macros;

#[cfg(all(feature = "button-async", feature = "button-poll"))]
compile_error!("You cannot enable both `button-async` and `button-poll`. Choose one.");

#[cfg(feature = "button-async")]
pub mod button_async;

#[cfg(feature = "button-poll")]
pub mod button_poll;
