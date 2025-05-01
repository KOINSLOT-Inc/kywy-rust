// SPDX-FileCopyrightText: 2025 KOINSLOT Inc.
// SPDX-License-Identifier: GPL-3.0-or-later
//
//! This library monitors the usb for a baud 1200 and reboots to bootloader for programming following the Arduino protocol

use defmt::{info, unwrap};
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::peripherals::USB;
use embassy_rp::rom_data::reset_to_usb_boot;
use embassy_rp::usb::{Driver, InterruptHandler};
use embassy_time::{Duration, Timer, WithTimeout};
use embassy_usb::class::cdc_acm::{CdcAcmClass, Receiver, Sender, State};
use embassy_usb::driver::EndpointError;
use embassy_usb::{Builder, Config, UsbDevice};
use static_cell::StaticCell;

type MyUsbDriver = Driver<'static, USB>;
type MyUsbDevice = UsbDevice<'static, MyUsbDriver>;

// Static buffers for USB
static CONFIG_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
static BOS_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
static CONTROL_BUF: StaticCell<[u8; 64]> = StaticCell::new();
static CDC_STATE: StaticCell<State> = StaticCell::new();

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => InterruptHandler<USB>;
});

#[embassy_executor::task]
pub async fn usb_monitor_task(spawner: Spawner, usb: USB) {
    let driver = Driver::new(usb, Irqs);

    let config = {
        let mut config = Config::new(0xc0de, 0xcafe);
        config.manufacturer = Some("Koinslot");
        config.product = Some("Kywy");
        config.serial_number = Some("11111");
        config.max_power = 100;
        config.max_packet_size_0 = 64;
        config
    };

    let mut builder = Builder::new(
        driver,
        config,
        CONFIG_DESCRIPTOR.init([0; 256]),
        BOS_DESCRIPTOR.init([0; 256]),
        &mut [], // no MS OS descriptors
        CONTROL_BUF.init([0; 64]),
    );

    // Create and split class
    let cdc = CdcAcmClass::new(&mut builder, CDC_STATE.init(State::new()), 64);
    let (mut sender, mut receiver, control) = cdc.split_with_control();

    let usb = builder.build();
    unwrap!(spawner.spawn(run_usb_task(usb)));
    sender.wait_connection().await;
    control.control_changed().await;
    loop {
        sender.wait_connection().await;
        control.control_changed().await;

        let baud = receiver.line_coding().data_rate();
        let dtr = sender.dtr();
        info!("DTR={}, baud={}", dtr, baud);

        if baud == 1200 && dtr {
            info!("Triggering bootloader via 1200 baud + DTR");
            Timer::after_millis(100).await;
            reset_to_usb_boot(0, 0);
        }

        // Only enter echo after confirming we're not rebooting
        if dtr && baud != 1200 {
            info!("Terminal connected. Starting echo...");
            let _ = echo(&mut receiver, &mut sender).await;
        }
    }
}

#[embassy_executor::task]
async fn run_usb_task(mut usb: MyUsbDevice) -> ! {
    usb.run().await
}

#[derive(Debug)]
struct Disconnected;

impl From<EndpointError> for Disconnected {
    fn from(val: EndpointError) -> Self {
        match val {
            EndpointError::BufferOverflow => panic!("Buffer overflow"),
            EndpointError::Disabled => Disconnected,
        }
    }
}

/// Echoes received USB packets back to the sender.
async fn echo(
    receiver: &mut Receiver<'static, MyUsbDriver>,
    sender: &mut Sender<'static, MyUsbDriver>,
) -> Result<(), Disconnected> {
    let mut buf = [0; 64];

    loop {
        // Exit echo mode if DTR is dropped
        if receiver.line_coding().data_rate() == 1200 {
            Timer::after_millis(100).await;
            reset_to_usb_boot(0, 0);
        }

        if !sender.dtr() {
            return Ok(());
        }

        match receiver
            .read_packet(&mut buf)
            .with_timeout(Duration::from_millis(100))
            .await
        {
            Ok(Ok(n)) if n > 0 => {
                let _ = sender.write_packet(&buf[..n]).await;
            }
            Ok(Err(e)) => return Err(e.into()),
            Err(_) => {
                // Timeout, but we loop again to check DTR/baud
            }
            _ => {}
        }
    }
}
