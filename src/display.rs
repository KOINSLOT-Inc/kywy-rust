// SPDX-FileCopyrightText: 2025 KOINSLOT Inc.
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Kywy Display Driver
//! Compatable with embedded-graphics DrawTarget
//! for use with LS013B7DH05

use core::ops::Not;
use embassy_rp::gpio::Output;
use embedded_graphics::{
    Pixel,
    draw_target::DrawTarget,
    pixelcolor::BinaryColor,
    prelude::{OriginDimensions, Size},
};
use embedded_hal_async::spi::SpiDevice;

const WIDTH: usize = 144;
const HEIGHT: usize = 168;
const TOTAL_BUFFER_SIZE: usize = WIDTH * HEIGHT / 8;
const BYTES_PER_LINE: usize = WIDTH / 8;
const LINE_PACKET_SIZE: usize = 2 + BYTES_PER_LINE + 2;

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

pub struct KywyDisplay<'a, SPI> {
    spi: SPI,
    disp: Output<'a>,
    buffer: [u8; TOTAL_BUFFER_SIZE],
    line_buf: [u8; LINE_PACKET_SIZE],
    vcom: Vcom,
    auto_vcom: bool,
}

impl<'a, SPI> KywyDisplay<'a, SPI>
where
    SPI: SpiDevice,
{
    pub fn new(spi: SPI, disp: Output<'a>) -> Self {
        Self {
            spi,
            disp,
            buffer: [0x00; TOTAL_BUFFER_SIZE],
            line_buf: [0x00; LINE_PACKET_SIZE],
            vcom: Vcom::Hi,
            auto_vcom: true, //defaults to toggling vcom every display update
        }
    }

    pub async fn initialize(&mut self) {
        self.disable();
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
        self.spi.transfer(&mut [], data).await.unwrap();
    }

    pub async fn write_display(&mut self) {
        if self.auto_vcom {
            self.vcom = !self.vcom;
        }

        for line in 0..HEIGHT {
            self.line_buf[0] = Command::WriteLine as u8 | self.vcom as u8 | 0x80;
            self.line_buf[1] = (line as u8 + 1).reverse_bits();

            let buf_start = line * BYTES_PER_LINE;
            for i in 0..BYTES_PER_LINE {
                self.line_buf[2 + i] = self.buffer[buf_start + i].reverse_bits();
            }

            self.line_buf[2 + BYTES_PER_LINE] = 0x00;
            self.line_buf[3 + BYTES_PER_LINE] = 0x00;

            self.spi.write(&self.line_buf).await.unwrap();
        }
    }

    pub async fn toggle_vcom(&mut self) {
        self.vcom = !self.vcom;
        self.write_spi(&[Command::Nop as u8 | self.vcom as u8, 0x00])
            .await;
    }

    pub fn set_auto_vcom(&mut self, enable: bool) {
        self.auto_vcom = enable;
    }

    pub fn is_auto_vcom(&self) -> bool {
        self.auto_vcom
    }

    pub async fn clear_display(&mut self) {
        if self.auto_vcom {
            self.vcom = !self.vcom;
        }
        self.write_spi(&[Command::ClearMemory as u8 | self.vcom as u8, 0x00])
            .await;
    }

    pub fn clear_buffer(&mut self, color: BinaryColor) {
        let fill_value = if color.is_on() { 0xFF } else { 0x00 };
        self.buffer.fill(fill_value);
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, color: BinaryColor) {
        if x >= WIDTH || y >= HEIGHT {
            return;
        }

        let index = (y * BYTES_PER_LINE) + (x / 8);
        let bit = x % 8;

        if color.is_on() {
            self.buffer[index] |= 1 << bit;
        } else {
            self.buffer[index] &= !(1 << bit);
        }
    }
    pub fn height(&self) -> usize {
        HEIGHT
    }
    pub fn width(&self) -> usize {
        WIDTH
    }
}

impl<SPI> OriginDimensions for KywyDisplay<'_, SPI> {
    fn size(&self) -> Size {
        Size::new(WIDTH as u32, HEIGHT as u32)
    }
}

impl<SPI> DrawTarget for KywyDisplay<'_, SPI>
where
    SPI: SpiDevice,
{
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
