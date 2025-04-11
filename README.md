<!--
SPDX-FileCopyrightText: 2025 KOINSLOT Inc.

SPDX-License-Identifier: GPL-3.0-or-later
-->

# Rust Library for Kywy devices
Get a kywy at https://kywy.io

Kywy is a small, low-cost, low-power, open-source, microcontroller board with a sharp memory display, a 5-way joystick, two side button, a USB-C connector, a qwiic I2C interface, SPI/GPIO expansion pins and a microSD card slot all in a small keychain package. It is designed to be easy to use and program, and is perfect for learning embedded programming, creating small games, or building into your projects.

This is an rust library for building out rust programs on the Kywy device. You can also find the C++ library at https://github.com/KOINSLOT-Inc/kywy.

# Features
 - Display driver
    - Supports embedded graphics library
    - Supports text rendering (examples/hello_world.rs)
    - Supports image rendering (examples/display_image.rs)
 - Button interface
    - Async message queue or polling support (examples/button_test_async.rs and examples/button_test_polling.rs)

# Still in progress
This is a work in progress.

To do:
- [X] Implement display driver (LS013B7DH05)
- [X] Implement button interface
- [X] Implement battery interface
- [X] Improve battery reading function
- [ ] Change the way display driver handles VCOM toggle
- [ ] Add more documentation
- [ ] Add more examples/games
- [ ] Implement USB serial and reboot
- [ ] Implement SD-card interface (waiting on shared bus support from embedded-sdmmc)


# Build examples
To build this run:
`cargo build --release --example <example_name> --target thumbv6m-none-eabi`
replacing <example_name> with one of the .rs files in the examples directory.

To convert to a UF2 file run:
`elf2uf2-rs target/thumbv6m-none-eabi/release/examples/<example_name>'
you can install elf2uf2-rs by running:
`cargo install elf2uf2-rs`
(make sure your path is set properly to find the binary)

UF2 file will then be in the directory 'target/thumbv6m-none-eabi/release/examples/'

# Including as a crate
You can include this project as a crate by adding it in your Cargo.toml file

# Editing your own
You can add an example to build directly from this repository by creating or modifying a file in the examples directory.

to start your own project with minimal configuration, download this repository with 'git clone https://github.com/Jmlannan/Kywy-Rust/' you can then add a new example in the examples directory. Build it with the command above in build examples.

You can also add this to your own project with the following in your Cargo.toml file. However, this library is currently unstable and may not work as expected.
