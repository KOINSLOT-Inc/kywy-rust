# Rust Library for Kywy devices
Get a kywy at https://kywy.io

Kywy is a small, low-cost, low-power, open-source, microcontroller board with a sharp memory display, a 5-way joystick, two side button, a USB-C connector, a qwiic I2C interface, SPI/GPIO expansion pins and a microSD card slot all in a small keychain package. It is designed to be easy to use and program, and is perfect for learning embedded programming, creating small games, or building into your projects.

This is an rust library for building out rust programs on the Kywy device. You can also find the C++ library at https://github.com/KOINSLOT-Inc/kywy.

# Still in progress
This is a work in progress and is not yet ready for use.

To do:
- [X] Implement display driver (LS013B7DH05)
- [ ] Implement button interface
- [ ] Implement SD-card interface
- [ ] Implement USB debug interface
- [ ] Implement USB mass storage interface
- [ ] Implement reboot to USB program mode
- [ ] Add support for qwiic devices
- [ ] Add support for SPI "backpack" devices
- [ ] Add more documentation
- [ ] Add more examples/games


# Build examples
To build this run:
`cargo build --release --example <example_name> --target thumbv6m-none-eabi`
replacing <example_name> with one of the .rs files in the examples directory.

To convert to a UF2 file run:
`elf2uf2-rs target/thumbv6m-none-eabi/release/examples/<example_name> target/thumbv6m-none-eabi/release/examples/<example_name>`

you can install elf2uf2-rs by running:
`cargo install elf2uf2-rs`

UF2 file will them be in the directory target/thumbv6m-none-eabi/release/examples/

#Including as a crate
You can include this project as a crate by adding it in your Cargo.toml file

#Editing your own
You can add an example to build directly from this repository by creating or modifying a file in the examples directory.
