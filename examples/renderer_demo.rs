#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use embedded_graphics::{pixelcolor::BinaryColor, prelude::*};
use kywy::{
    engine::{
        renderer::Renderer,
        sprite::{Animation, SpriteInstance, SpriteSheet},
    },
    kywy_display_from, kywy_spi_from,
};
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Renderer demo starting");

    let p = embassy_rp::init(Default::default());
    kywy_spi_from!(p => spi_bus);
    kywy_display_from!(spi_bus, p => display);
    display.initialize().await;
    display.enable();

    static FIRE_DATA: &[u8] = include_bytes!("../examples/Art Assets/monsters/fire.bmp");
    static ELECTRIC_DATA: &[u8] = include_bytes!("../examples/Art Assets/monsters/electric.bmp");

    let fire_sheet = SpriteSheet::new(FIRE_DATA, Size::new(64, 64)).unwrap();
    let electric_sheet = SpriteSheet::new(ELECTRIC_DATA, Size::new(64, 64)).unwrap();

    let fire_loop = Animation::new(&fire_sheet, &[(0, 0), (1, 0), (2, 0)], true);
    let electric_loop = Animation::new(&electric_sheet, &[(0, 0), (1, 0), (2, 0)], true);

    let fire_flash = Animation::new(&fire_sheet, &[(0, 1), (1, 1), (2, 1)], false);
    let electric_flash = Animation::new(&electric_sheet, &[(0, 1), (1, 1), (2, 1)], false);

    let fire_sprite = SpriteInstance::new(fire_loop, Point::new(10, 40));
    let electric_sprite = SpriteInstance::new(electric_loop, Point::new(90, 40));

    let mut renderer = Renderer::new(BinaryColor::Off);

    renderer.add_static(fire_sprite, 1);
    renderer.add_static(electric_sprite, 1);
    renderer.add_animation(fire_flash, Point::new(10, 40), 2);
    renderer.add_animation(electric_flash, Point::new(90, 40), 2);

    let mut trigger_timer = 0;

    loop {
        renderer.update();
        display.clear(BinaryColor::On);
        renderer.draw(&mut display).unwrap();
        display.write_display().await;

        trigger_timer += 1;
        if trigger_timer == 20 {
            renderer.trigger_animation(2); // fire flash
            renderer.trigger_animation(3); // electric flash
        }

        Timer::after(Duration::from_millis(100)).await;
    }
}
