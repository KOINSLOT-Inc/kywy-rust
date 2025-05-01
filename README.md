<!--
SPDX-FileCopyrightText: 2025 KOINSLOT Inc.

SPDX-License-Identifier: GPL-3.0-or-later
-->
<h1 align='center'>
  Kywy Rust Support
</h1>

<p align='center'><i>
  the tiny game device with big possibilities: education, game dev, diy electronics, and more
</i></p>

<p align='center'>
  <a href="https://linktr.ee/koinslotkywy"><img alt="Linktree" src="https://img.shields.io/badge/linktree-39E09B?style=for-the-badge&logo=linktree&logoColor=white" /></a>
  &nbsp;
  <a href="https://discord.gg/zAYym57Fy6"><img alt="Discord" src="https://img.shields.io/discord/1172988360063725629?style=for-the-badge&logo=discord" /></a>
  &nbsp;
  <a href="https://kywy.io/"><img alt="Website" src="https://img.shields.io/badge/KYWY.io-AAAAAA?style=for-the-badge&logo=data%3Aimage%2Fpng%3Bbase64%2CiVBORw0KGgoAAAANSUhEUgAAACQAAAAkCAYAAADhAJiYAAAACXBIWXMAAAAnAAAAJwEqCZFPAAAAAXNSR0IArs4c6QAAAARnQU1BAACxjwv8YQUAAAKwSURBVHgBtViNdeIwDBZ9N0A2aDa4bHDpBMcIuQ2yAd6guQngJoCbIHQC2gkCE8AGqvTivApXiuMA33t6hliSP0v%2BUbKARCBiQU1J8ouEf2deGBeSo5f%2FJPvFYnGEe4NIZCQ1SYvpaEmqqWMtJpBZUvNKkgddJ5I99FG5%2BGccKY7as6J%2FJHmZHTEflW0w247Ecd8E%2B4Jk420kVpAKMsoDR%2BeUsCv%2BXEBqO2VSFpnpxml%2BD1G%2FPk3SyMEdoSyDdcxg8ygywTiSVG0pVZOZ305IZoLXZ64pDQqdpuDXwAGv8aroFfh9Z%2FH%2FUtEb0IZOZHQqY1Yt6sgD0p2h5xSfjegv1eiAgRFChdDZGjpqWrBP3YBGC101lxC1K7Qxxe95eFALw3wGoRL7VFloYAR4vVxKfrDzfw4Rw9oY8OxFQwcR4HXa6h%2FQX4SMU6CYU8Pbn9sjyR%2FoL1N5ug79GtjmBSKgy%2FZCY508j58gZucCQptgtq0yuwptOJgI%2FFoO%2Bye4nrFEGfwvIA3PMANPI31h%2Fj8gDRUGh%2BEUMCFZXEnwmvkH%2Fdp64wH4IfY7Kvdr7DeMY42plQJ%2BXQe7iN7S2E182mrXhbn2FN%2BD30Yu3nPEqDUGHA7GEm3UI37lGVZzyt59XzYn5%2BBTTdt3T81fQ2eF9qFbit%2F78GByhtHoSS10wgJPoo347bSHnMvMMNwZA%2BWB3ljqikBXpmttOXEGoaUy%2B8bQdQqZTiG%2FsSYWRimHBwNjFWoQpfgbwW1kZCHXmQEIQv2Quhq%2FV5VVzEBWfXd5Jxsh46YYZXhdzNshTSMTnvQuxVh7r1%2FPIebXZhv4cjAHqG%2FftZ9tNmLHqdE%2B33CESrgF3vkGdXR%2B0EEOaJezDd5z5wpiHU4Hk3MpRKIfrAxyJfQVJEsuuoaPV3xh8%2Be8d0jEJ3i2%2BwETXUzkAAAAAElFTkSuQmCC&link=kywy.io" /></a>
  &nbsp;
  <a href="https://koinslot.myshopify.com"><img alt="Store" src="https://img.shields.io/badge/Store-F7931E?style=for-the-badge&logo=shopify&logoColor=232323&link=https%3A%2F%2Fkoinslot.myshopify.com" /></a>
</p>
<br />

# What is Kywy?

<p align='center'><img alt="Front and back render of a Kywy device" src="https://github.com/KOINSLOT-Inc/kywy/blob/main/assets/kywy_front_back_render.png"/></p>

The hardware features:

* a 144x168 LCD screen
* two buttons plus a joystick
* SD card slot
* all day battery life
* I/O: USB-C, GPIO headers, and I2C

The software is designed to give you easy and intuitive access to all of those with only a few lines of code.

# Where to buy
Get a kywy at https://kywy.io

# Rust library for Kywy devices

This is an rust library for building out rust programs on the Kywy device it is currently experimental. You can also find the C++/Arduino library at https://github.com/KOINSLOT-Inc/kywy or in the Ardiuno library manager https://docs.arduino.cc/libraries/kywy/.

For support, join our discord: 

[![Discord](https://img.shields.io/discord/1172988360063725629?label=Join%20us%20on%20Discord&logo=discord&style=flat&color=5865F2)](https://discord.gg/d65Xfdjp)

# Rust docs
https://koinslot-inc.github.io/kywy-rust/kywy/

# Still in progress
This is a work in progress.

To do:
- [X] Implement display driver (LS013B7DH05)
- [X] Implement button interface
- [X] Implement battery interface
- [X] Implement USB serial and reboot
- [X] Sprites
- [ ] Improve battery reading function with real data
- [ ] Build game engine
- [ ] Add more documentation
- [ ] Add more examples/games
- [ ] Implement SD-card interface (waiting on shared bus support from embedded-sdmmc)


# Build examples
This repo supports vscode environments and code spaces 

[![Open in GitHub Codespaces](https://github.com/codespaces/badge.svg)](https://github.com/codespaces/new?repo=KOINSLOT-Inc/kywy-rust)

To build this run:
`cargo build --release --target thumbv6m-none-eabi --example <example_name>`
replacing <example_name> with one of the .rs files in the examples directory.

To convert to a UF2 file run:
`elf2uf2-rs target/thumbv6m-none-eabi/release/examples/<example_name>`
you can install elf2uf2-rs by running:
`cargo install elf2uf2-rs`
(make sure your path is set properly to find the binary)

UF2 file will then be in the directory 'target/thumbv6m-none-eabi/release/examples/'

# Uploading UF2 files
Note that your code must use the kywy_usb_from! macro to support automatic rebooting. To do this start a baud 1200 terminal on the device.

You can also put the device into programming mode manually:
1. Turn off and unplug the device
2. Use a paper clip to press and hold the reset button on the back to the right of the kywy logo
3. while holding insert the USB
4. wait a second and you should see a usb device. You can now release the button

You can now copy the UF2 to the USB storage device.

# Including as a crate from crates.io [![Crates.io Version](https://img.shields.io/crates/v/kywy)](https://crates.io/crates/kywy)

use this to add kywy to a rust project:
<pre><code id="code-block">cargo add kywy</code></pre>
<button onclick="navigJmlannanator.clipboard.writeText(document.getElementById('code-block').innerText)"></button>
You will need additional setup files, it may be easier to git clone this repo and make a new example file in it.

# Editing your own
You can add an example to build directly from this repository by creating or modifying a file in the examples directory.

to start your own project with minimal configuration, download this repository with  `git clone https://github.com/KOINSLOT-INC/kywy-rust/` you can then add a new example in the examples directory. Build it with the command above in build examples. 

# Setup rust
Prerequisites:
  - Familiarity with using the terminal, installing programs, and using PATH variables
  - Familiarity with using development tools

If you want an easier method of developing for kywy, check out the arduino library: https://docs.arduino.cc/libraries/kywy/

1. Install rust with rust according to the official documents: [https://rustup.rs](https://www.rust-lang.org/tools/install)
2. Make sure you set your paths for rust and cargo bin
3. Install elf2uf2-rs: `cargo install elf2uf2-rs`
4. Test that elf2uf2-rs is in your path by running elf2uf2-rs
