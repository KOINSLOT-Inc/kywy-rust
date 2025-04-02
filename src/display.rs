//! Status: Working
//!  Potential to do:
//!    - Increase buffer size to send more lines to the DMA at a time
//! The purpose of this library is to create a driver interface for the sharp memory display
//!
//! Display is a 144x168 pixel monochrome display (LS013B7DH05)
//! Clock frequency: 1MHz (1.1MHz max), though testing shows it can be overdriven
//! Pins: 18=SCK 19=MOSI 16=MISO 16=CS 22=DISP
//! EXTMOD=Low, Should enable automatic VCOM control but not supported on the DH05 model
//!
//! Display commands
//!     In little-endian format, LSB left, MSB right
//! Clear Command
//!     1Byte, 0x60
//!     M0: Data update mode, low
//!     M1: Frame inversion/VCOM, set vcom high/low
//!     M2: All clear
//!     Commands are sent M0 first, followed by additional data
//! Toggle VCOM:
//!     1Byte, 0x40 or 0x00 (Remember internal data, VCOM bit, dont clear, then dummy bites)
//! Write Line/display command divided into bytes:
//!     Mode select: 3command(M0, M1, M2) + 5dummy bits 1byte
//!        M0: High = Data update mode (one line), Low=display update mode (full display, contenuous)
//!        M1: Frame inversion/VCOM, set vcom high/low
//!        M2: All clear
//!     Line address: 8 byte
//!     Data: 144 bits per line (18 bytes) little-endian, leftmost pixel first
//!     Dummy: 16 bits (recomdended to just send low)
//!     If M0 low, loop back to data for n-1 line
//!
//! CS: Chip select, active high,
//! Display pin: Active high, low to disable, does not clear display, can be used or blanking. Wired to be pulled low at startup so display can be cleared to remove random data.
//! Command bit is reset after CS goes low

use core::ops::Not;
use embassy_rp::Peri;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::peripherals::SPI0;
use embassy_rp::peripherals::*;
use embassy_rp::spi::{Config as SpiConfig, Phase, Polarity, Spi};
use embedded_graphics::{
    Pixel,
    draw_target::DrawTarget,
    pixelcolor::BinaryColor,
    prelude::{OriginDimensions, Size},
};

// Constants
const DISPLAY_FREQ: u32 = 1_000_000; // 1MHz
const WIDTH: usize = 144;
const HEIGHT: usize = 168;
const TOTAL_BUFFER_SIZE: usize = WIDTH * HEIGHT / 8; // buffer that only holds 1-bit pixel data
const BYTES_PER_LINE: usize = WIDTH / 8; // 144 bits / 8
const LINE_PACKET_SIZE: usize = 2 + BYTES_PER_LINE + 2; // Size of the SPI command buffer for updating a line: command + address + data + dummy

// Command definitions
#[derive(Clone, Copy, PartialEq, Eq)]
enum Vcom {
    Lo = 0x00,
    Hi = 0x40,
}

impl Not for Vcom {
    type Output = Self;
    fn not(self) -> Self::Output {
        match self {
            Vcom::Lo => Vcom::Hi,
            Vcom::Hi => Vcom::Lo,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Command {
    Nop = 0x00,
    ClearMemory = 0x20,
    WriteLine = 0x80,
}

pub struct KywyDisplay<'a> {
    spi: Spi<'a, SPI0, embassy_rp::spi::Async>,
    cs: Output<'a>,
    disp: Output<'a>,
    buffer: [u8; TOTAL_BUFFER_SIZE], // 144x168 / 8 framebuffer (separate from spi command buffer)
    line_buf: [u8; LINE_PACKET_SIZE], // buffer for line updates to send to DMA
    vcom: Vcom,
}

impl<'a> KywyDisplay<'a> {
    pub async fn new(
        spi: Peri<'static, SPI0>,
        dma_tx: Peri<'static, DMA_CH0>,
        dma_rx: Peri<'static, DMA_CH1>,
        sck: Peri<'static, PIN_18>,
        mosi: Peri<'static, PIN_19>,
        miso: Peri<'static, PIN_16>,
        cs: Peri<'static, PIN_17>,
        disp: Peri<'static, PIN_22>,
    ) -> Self {
        let mut config = SpiConfig::default();
        config.frequency = DISPLAY_FREQ;
        config.polarity = Polarity::IdleLow;
        config.phase = Phase::CaptureOnFirstTransition; // Mode 0

        let spi = Spi::new(spi, sck, mosi, miso, dma_tx, dma_rx, config);

        let cs = Output::new(cs, Level::Low);
        let disp = Output::new(disp, Level::Low);

        Self {
            spi,
            cs,
            disp,
            buffer: [0x00; TOTAL_BUFFER_SIZE], // Start with all white
            line_buf: [0x00; LINE_PACKET_SIZE], // Zeroed line buffer
            vcom: Vcom::Hi,
        }
    }

    pub async fn initialize(&mut self) {
        self.disable(); // Disable display, should be already disabled

        // Clear display (send twice)
        self.vcom = Vcom::Hi;
        self.clear_display().await;

        self.enable();
    }

    pub fn enable(&mut self) {
        self.disp.set_high();
    }

    pub fn disable(&mut self) {
        self.disp.set_low();
    }

    pub async fn write_spi(&mut self, data: &[u8]) {
        //write a single command
        self.cs.set_high();
        self.spi.write(data).await.unwrap();
        self.cs.set_low();
    }

    pub async fn write_display(&mut self) {
        self.vcom = !self.vcom;

        for line in 0..HEIGHT {
            // M0 = 1 (line update mode), M1 = VCOM, M2 = 0 (no clear)
            self.line_buf[0] = Command::WriteLine as u8 | self.vcom as u8 | 0x80; // ensure M0 = 1
            self.line_buf[1] = (line as u8 + 1).reverse_bits();

            let buf_start = line * BYTES_PER_LINE;
            for i in 0..BYTES_PER_LINE {
                self.line_buf[2 + i] = self.buffer[buf_start + i].reverse_bits();
            }

            self.line_buf[2 + BYTES_PER_LINE] = 0x00;
            self.line_buf[3 + BYTES_PER_LINE] = 0x00;

            self.cs.set_high();
            self.spi.write(&self.line_buf).await.unwrap();
            self.cs.set_low();
        }
    }

    pub async fn toggle_vcom(&mut self) {
        self.vcom = !self.vcom;
        self.write_spi(&[Command::Nop as u8 | self.vcom as u8, 0x00])
            .await;
    }

    pub async fn clear_display(&mut self) {
        self.vcom = !self.vcom;
        self.write_spi(&[Command::ClearMemory as u8 | self.vcom as u8, 0x00])
            .await;
    }

    pub fn clear_buffer(&mut self, color: BinaryColor) {
        let fill_value = if color.is_on() { 0xFF } else { 0x00 };
        self.buffer.fill(fill_value);
    }

    // LSB-first ordering
    pub fn set_pixel(&mut self, x: usize, y: usize, color: BinaryColor) {
        if x >= WIDTH || y >= HEIGHT {
            return;
        }

        let index = (y * BYTES_PER_LINE) + (x / 8);
        let bit = x % 8;

        if color.is_on() {
            self.buffer[index] |= 1 << bit; // LSB-first
        } else {
            self.buffer[index] &= !(1 << bit);
        }
    }
}

// Implement OriginDimensions trait
impl<'a> OriginDimensions for KywyDisplay<'a> {
    fn size(&self) -> Size {
        Size::new(WIDTH as u32, HEIGHT as u32)
    }
}

// Implement DrawTarget trait
impl<'a> DrawTarget for KywyDisplay<'a> {
    type Color = BinaryColor;
    type Error = ();

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(coord, color) in pixels {
            if coord.x >= 0 && coord.y >= 0 {
                let x = coord.x as usize;
                let y = coord.y as usize;
                if x < WIDTH && y < HEIGHT {
                    self.set_pixel(x, y, color);
                }
            }
        }
        Ok(())
    }
}
