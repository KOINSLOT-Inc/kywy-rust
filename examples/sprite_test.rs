#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use embedded_graphics::{pixelcolor::BinaryColor, prelude::*};
use kywy::{
    engine::sprite::{Animation, SpriteSheet},
    kywy_display_from, kywy_spi_from,
};
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Sprite test starting");

    let p = embassy_rp::init(Default::default());

    kywy_spi_from!(p => spi_bus);
    kywy_display_from!(spi_bus, p => display);
    display.initialize().await;
    display.enable();

    static SPRITE_DATA: &[u8] = include_bytes!("../examples/Art Assets/monsters/electric.bmp");

    let sheet = SpriteSheet::new(SPRITE_DATA, Size::new(64, 64)).unwrap();

    let frames: &[(u32, u32)] = &[(0, 0), (1, 0), (2, 0)];
    let mut animation = Animation::new(frames, true);

    loop {
        let (fx, fy) = animation.current_frame();
        let sprite = sheet.sprite(fx, fy).unwrap();

        display.clear(BinaryColor::On).unwrap();
        sprite.draw(&mut display, Point::new(40, 40)).unwrap();
        display.write_display().await;

        animation.advance();
        Timer::after(Duration::from_millis(200)).await;
    }
}
