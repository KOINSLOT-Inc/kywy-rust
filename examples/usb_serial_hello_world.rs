#![no_std]
#![no_main]

use defmt_rtt as _;
use panic_probe as _;

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use kywy::usb;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    // Initialize dual CDC USB: one for log::info!(), one for echo/write
    usb::init(p.USB, spawner);

    loop {
        log::info!("Hello from log!");
        usb::write("Hello from echo\n");
        Timer::after(Duration::from_secs(2)).await;
    }
}
