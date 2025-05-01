// SPDX-FileCopyrightText: 2025 KOINSLOT Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! USB console and bootloader monitor for RP2040.

use defmt::{info, unwrap};
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::peripherals::USB;
use embassy_rp::rom_data::reset_to_usb_boot;
use embassy_rp::usb::{Driver, InterruptHandler};
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_time::{Duration, Timer, WithTimeout};
use embassy_usb::class::cdc_acm::{CdcAcmClass, Receiver, Sender, State};
use embassy_usb::driver::EndpointError;
use embassy_usb::{Builder, Config, UsbDevice};
use heapless::spsc::Queue;
use static_cell::StaticCell;

type MyUsbDriver = Driver<'static, USB>;
type MyUsbDevice = UsbDevice<'static, MyUsbDriver>;

static CONFIG_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
static BOS_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
static CONTROL_BUF: StaticCell<[u8; 64]> = StaticCell::new();
static CDC_STATE: StaticCell<State> = StaticCell::new();

// Shared queues
static USB_RX_QUEUE: Mutex<CriticalSectionRawMutex, Queue<u8, 256>> = Mutex::new(Queue::new());
static USB_TX_QUEUE: Mutex<CriticalSectionRawMutex, Queue<u8, 256>> = Mutex::new(Queue::new());

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

        // Bootloader trigger
        if baud == 1200 && dtr {
            Timer::after_millis(100).await;
            reset_to_usb_boot(0, 0);
        }

        if !dtr {
            return Ok(());
        }

        // Read USB → enqueue to RX queue
        match receiver
            .read_packet(&mut read_buf)
            .with_timeout(Duration::from_millis(10))
            .await
        {
            Ok(Ok(n)) if n > 0 => {
                let mut rx = USB_RX_QUEUE.lock_mut();
                for &b in &read_buf[..n] {
                    let _ = rx.enqueue(b);
                }
            }
            _ => {}
        }

        // Dequeue from TX queue → send USB
        let n = {
            let mut tx = USB_TX_QUEUE.lock_mut();
            let mut i = 0;
            while i < write_buf.len() {
                if let Some(b) = tx.dequeue() {
                    write_buf[i] = b;
                    i += 1;
                } else {
                    break;
                }
            }
            i
        };

        if n > 0 {
            let _ = sender.write_packet(&write_buf[..n]).await;
        }

        Timer::after_millis(1).await;
    }
}

// === Public USB Console API ===

/// Enqueue a byte slice for USB output.
pub fn usb_print(data: &[u8]) {
    let mut q = USB_TX_QUEUE.lock_mut();
    for &b in data {
        let _ = q.enqueue(b);
    }
}

/// Print a string followed by newline.
pub fn usb_println(line: &str) {
    usb_print(line.as_bytes());
    usb_print(b"\n");
}

/// Read one character from the USB RX queue.
pub fn usb_read_char() -> Option<u8> {
    let mut q = USB_RX_QUEUE.lock_mut();
    q.dequeue()
}

/// Read a line ending in `\n` into the given buffer.
pub fn usb_read_line(buf: &mut [u8]) -> Option<usize> {
    let mut q = USB_RX_QUEUE.lock_mut();
    let mut i = 0;

    while i < buf.len() {
        match q.dequeue() {
            Some(b) => {
                buf[i] = b;
                i += 1;
                if b == b'\n' {
                    break;
                }
            }
            None => break,
        }
    }

    if i > 0 { Some(i) } else { None }
}
