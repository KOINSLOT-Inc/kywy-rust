//! examples/display_image.rs
//! Bitmap Graphic Demo for Kywy display
//! Make sure BMP is converted to monochrome useing magick "examples/Art Assets/<image file>" -resize 144x168 -monochrome -depth 1 BMP3:"examples/Art Assets/<filename>.bmp"

#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_executor;
use embassy_time::{Duration, Timer};
use embedded_graphics::{image::Image, pixelcolor::BinaryColor, prelude::*};
use kywy::kywy_display_from;
use panic_probe as _;
use tinybmp::Bmp;

#[embassy_executor::main]
async fn main(_spawner: embassy_executor::Spawner) {
    info!("Starting Kywy BMP Graphic Demo!");

    let p = embassy_rp::init(Default::default());
    kywy_spi_from!(p => spi_bus);
    kywy_display_from!(spi_bus, p => display);

    display.initialize().await;
    display.enable();

    // Clear screen to white before drawing
    display.clear_buffer(BinaryColor::On);

    // === Load and draw BMP image ===
    static IMAGE_DATA: &[u8] = include_bytes!("../examples/Art Assets/KywyRust.bmp"); //change this line to use a different image
    let bmp = Bmp::from_slice(IMAGE_DATA).unwrap();

    let image = Image::new(&bmp, Point::zero());
    image.draw(&mut display).unwrap();

    display.write_display().await;

    // === VCOM maintenance loop ===
    loop {
        display.toggle_vcom().await;
        Timer::after(Duration::from_secs(1)).await;
    }
}
