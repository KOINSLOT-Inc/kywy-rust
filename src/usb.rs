// SPDX-FileCopyrightText: 2025 KOINSLOT Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! USB console and bootloader monitor for RP2040.

use defmt::{info, unwrap};
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::peripherals::USB;
use embassy_rp::rom_data::reset_to_usb_boot;
use embassy_rp::usb::{Driver, InterruptHandler};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::{Duration, Timer, WithTimeout};
use embassy_usb::class::cdc_acm::{CdcAcmClass, Receiver, Sender, State};
use embassy_usb::driver::EndpointError;
use embassy_usb::{Builder, Config, UsbDevice};
use static_cell::StaticCell;

type MyUsbDriver = Driver<'static, USB>;
type MyUsbDevice = UsbDevice<'static, MyUsbDriver>;

static CONFIG_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
static BOS_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
static CONTROL_BUF: StaticCell<[u8; 64]> = StaticCell::new();
static CDC_STATE: StaticCell<State> = StaticCell::new();

// Channels for USB I/O
static USB_RX_CHANNEL: Channel<CriticalSectionRawMutex, u8, 256> = Channel::new();
static USB_TX_CHANNEL: Channel<CriticalSectionRawMutex, u8, 256> = Channel::new();

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => InterruptHandler<USB>;
});

#[embassy_executor::task]
pub async fn usb_monitor_task(spawner: Spawner, usb: USB) {
    let driver = Driver::new(usb, Irqs);

    let config = {
        let mut config = Config::new(0x2e8a, 0x00c0); // Arduino VID, Pico PID
        config.manufacturer = Some("Arduino");
        config.product = Some("RaspberryPi Pico");
        config.serial_number = Some("A63064E62CA5324B"); // or None if you want dynamic serial
        config.max_power = 250; // 500mA / 2, as per USB spec units
        config.max_packet_size_0 = 64;
        config.device_class = 0xef; // Miscellaneous Device
        config.device_sub_class = 0x02; // Common Class
        config.device_protocol = 0x01; // Interface Association Descriptor
        config
    };

    let mut builder = Builder::new(
        driver,
        config,
        CONFIG_DESCRIPTOR.init([0; 256]),
        BOS_DESCRIPTOR.init([0; 256]),
        &mut [],
        CONTROL_BUF.init([0; 64]),
    );

    let cdc = CdcAcmClass::new(&mut builder, CDC_STATE.init(State::new()), 64);
    let (mut sender, mut receiver, control) = cdc.split_with_control();

    let usb = builder.build();
    unwrap!(spawner.spawn(run_usb_task(usb)));

    sender.wait_connection().await;
    control.control_changed().await;

    loop {
        sender.wait_connection().await;
        control.control_changed().await;

        let dtr = sender.dtr();
        let baud = receiver.line_coding().data_rate();
        info!("DTR={}, baud={}", dtr, baud);

        if dtr {
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
            EndpointError::BufferOverflow => panic!("USB buffer overflow"),
            EndpointError::Disabled => Disconnected,
        }
    }
}

/// Full duplex USB communication: receive → RX queue, TX queue → send.
async fn echo(
    receiver: &mut Receiver<'static, MyUsbDriver>,
    sender: &mut Sender<'static, MyUsbDriver>,
) -> Result<(), Disconnected> {
    let mut read_buf = [0; 64];
    let mut write_buf = [0; 64];

    loop {
        let dtr = sender.dtr();
        let baud = receiver.line_coding().data_rate();

        if baud == 1200 && dtr {
            Timer::after_millis(100).await;
            reset_to_usb_boot(0, 0);
        }

        if !dtr {
            return Ok(());
        }

        // Read USB → echo to terminal and enqueue for app
        match receiver
            .read_packet(&mut read_buf)
            .with_timeout(Duration::from_millis(10))
            .await
        {
            Ok(Ok(n)) if n > 0 => {
                for &b in &read_buf[..n] {
                    // 1. Send to application
                    let _ = USB_RX_CHANNEL.try_send(b);
                    // 2. Echo back to user
                    let _ = USB_TX_CHANNEL.try_send(b);
                }
            }
            _ => {}
        }

        // Flush queued app responses
        let mut n = 0;
        while n < write_buf.len() {
            match USB_TX_CHANNEL.try_receive() {
                Ok(b) => {
                    write_buf[n] = b;
                    n += 1;
                }
                Err(_) => break,
            }
        }

        if n > 0 {
            let _ = sender.write_packet(&write_buf[..n]).await;
        }

        Timer::after_millis(1).await;
    }
}

// === Public USB Console API ===

/// Enqueue a byte slice for USB output.
pub async fn usb_print(data: &[u8]) {
    for &b in data {
        let _ = USB_TX_CHANNEL.send(b).await;
    }
}

/// Print a string followed by newline.
pub async fn usb_println(line: &str) {
    usb_print(line.as_bytes()).await;
    usb_print(b"\r\n").await;
}

/// Read one character from the USB RX channel.
pub async fn usb_read_char() -> u8 {
    USB_RX_CHANNEL.receive().await
}

/// Read a line into the given buffer, ending on `\r`, `\n`, or `\r\n`.
pub async fn usb_read_line(buf: &mut [u8]) -> usize {
    let mut i = 0;

    while i < buf.len() {
        let b = USB_RX_CHANNEL.receive().await;

        // Treat \r or \n as end of line
        if b == b'\r' || b == b'\n' {
            // Optional: skip \n after \r to handle \r\n
            if b == b'\r' {
                if let Ok(b2) = USB_RX_CHANNEL.try_receive() {
                    if b2 != b'\n' {
                        // Not a \r\n combo, push b2 back manually if needed
                        let _ = USB_RX_CHANNEL.try_send(b2);
                    }
                }
            }
            break;
        }

        buf[i] = b;
        i += 1;
    }

    i
}
