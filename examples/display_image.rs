// SPDX-FileCopyrightText: 2025 KOINSLOT Inc.
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! examples/display_image.rs
//! Bitmap Graphic Demo for Kywy display
//! Make sure BMP is converted to monochrome useing magick "examples/Art Assets/image_file" -resize 144x168 -monochrome -depth 1 BMP3:"examples/Art Assets/filename.bmp"

#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_time::{Duration, Timer};
use embedded_graphics::{image::Image, pixelcolor::BinaryColor, prelude::*};
use kywy::kywy_display_from;
use kywy::kywy_spi_from;
use kywy::kywy_usb_from;
use panic_probe as _;
use tinybmp::Bmp;

#[embassy_executor::main]
async fn main(spawner: embassy_executor::Spawner) {
    info!("Starting Kywy BMP Graphic Demo!");

    let p = embassy_rp::init(Default::default());
    kywy_spi_from!(p => spi_bus);
    kywy_display_from!(spi_bus, p => display);
    kywy_usb_from!(spawner, p);

    display.initialize().await;
    display.enable();

    display.clear_buffer(BinaryColor::On);

    static IMAGE_DATA: &[u8] = include_bytes!("../examples/Art Assets/KywyRust.bmp"); //change this line to use a different image
    let bmp = Bmp::from_slice(IMAGE_DATA).unwrap();

    let image = Image::new(&bmp, Point::zero());
    image.draw(&mut display).unwrap();

    display.write_display().await;

    loop {
        display.toggle_vcom().await;
        Timer::after(Duration::from_secs(1)).await;
    }
}
