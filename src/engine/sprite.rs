// SPDX-FileCopyrightText: 2025 KOINSLOT Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! Sprite handler for Kywy game engine

use embedded_graphics::Drawable;
use embedded_graphics::geometry::{Point, Size};
use embedded_graphics::image::{Image, ImageRaw};
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::*;
use heapless::Vec;
use tinybmp::{Bmp, ParseError};

/// A sprite sheet made from a BMP image
#[derive(Debug)]
pub struct SpriteSheet<'a> {
    bmp: Bmp<'a, BinaryColor>,
    sprite_size: Size,
    sheet_size: Size,
}

#[derive(Clone, Copy)]
pub struct SpriteOptions {
    pub flip_x: bool,
    pub flip_y: bool,
    pub rotation: Rotation,
}

#[derive(Clone, Copy)]
pub enum Rotation {
    None,
    R90,
    R180,
    R270,
}

impl Default for SpriteOptions {
    fn default() -> Self {
        Self {
            flip_x: false,
            flip_y: false,
            rotation: Rotation::None,
        }
    }
}

impl<'a> SpriteSheet<'a> {
    pub fn new(bmp_data: &'a [u8], sprite_size: Size) -> Result<Self, ParseError> {
        let bmp = Bmp::from_slice(bmp_data)?;
        let bmp_size = bmp.size();
        let sheet_size = Size::new(
            bmp_size.width / sprite_size.width,
            bmp_size.height / sprite_size.height,
        );

        Ok(Self {
            bmp,
            sprite_size,
            sheet_size,
        })
    }

    pub fn sprite_count(&self) -> Size {
        self.sheet_size
    }

    pub fn sprite(&self, index_x: u32, index_y: u32) -> Option<Sprite<'_>> {
        if index_x >= self.sheet_size.width || index_y >= self.sheet_size.height {
            return None;
        }

        let offset = Point::new(
            (index_x * self.sprite_size.width) as i32,
            (index_y * self.sprite_size.height) as i32,
        );

        Some(Sprite {
            sheet: self,
            offset,
        })
    }
}

pub struct Sprite<'a> {
    sheet: &'a SpriteSheet<'a>,
    offset: Point,
}

impl Sprite<'_> {
    pub fn draw<D>(
        &self,
        target: &mut D,
        pos: Point,
        options: SpriteOptions,
    ) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = BinaryColor>,
    {
        let w = self.sheet.sprite_size.width as usize;
        let h = self.sheet.sprite_size.height as usize;
        let mut framebuffer = [0u8; 64 * 64 / 8];

        for pixel in self.sheet.bmp.pixels() {
            let Point { x, y } = pixel.0;
            if x < self.offset.x
                || y < self.offset.y
                || x >= self.offset.x + w as i32
                || y >= self.offset.y + h as i32
            {
                continue;
            }

            // Local sprite coordinates
            let mut lx = (x - self.offset.x) as usize;
            let mut ly = (y - self.offset.y) as usize;

            // Flip
            if options.flip_x {
                lx = w - 1 - lx;
            }
            if options.flip_y {
                ly = h - 1 - ly;
            }

            // Rotation
            let (fx, fy) = match options.rotation {
                Rotation::None => (lx, ly),
                Rotation::R90 => (h - 1 - ly, lx),
                Rotation::R180 => (w - 1 - lx, h - 1 - ly),
                Rotation::R270 => (ly, w - 1 - lx),
            };

            let idx = fy * w + fx;
            if pixel.1.is_on() {
                let byte = idx / 8;
                let bit = 7 - (idx % 8);
                framebuffer[byte] |= 1 << bit;
            }
        }

        let used_bytes = w * h / 8;
        let img_raw = ImageRaw::<BinaryColor>::new(&framebuffer[..used_bytes], w as u32);
        Image::new(&img_raw, pos).draw(target)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Animation<'a> {
    pub sheet: &'a SpriteSheet<'a>, // reference to the sprite sheet
    pub frames: &'a [(u32, u32)],   // list of (index_x, index_y) for frames
    pub looped: bool,
    pub current_frame: usize,
    pub finished: bool,
}

impl<'a> Animation<'a> {
    pub fn new(sheet: &'a SpriteSheet<'a>, frames: &'a [(u32, u32)], looped: bool) -> Self {
        Self {
            sheet,
            frames,
            looped,
            current_frame: 0,
            finished: false,
        }
    }

    pub fn advance(&mut self) {
        self.current_frame += 1;
        if self.current_frame >= self.frames.len() {
            if self.looped {
                self.current_frame = 0;
            } else {
                self.current_frame = self.frames.len() - 1;
                self.finished = true;
            }
        }
    }

    pub fn current_frame_sprite(&self) -> Option<Sprite<'a>> {
        let (x, y) = self.frames[self.current_frame];
        self.sheet.sprite(x, y)
    }

    pub fn current_frame_loc(&self) -> (u32, u32) {
        self.frames[self.current_frame]
    }

    pub fn is_finished(&self) -> bool {
        self.finished
    }
}

pub struct SpriteInstance<'a> {
    pub animations: Vec<Animation<'a>, 4>,
    pub active_index: usize,
    pub position: Point,
}

impl<'a> SpriteInstance<'a> {
    pub fn move_by(&mut self, dx: i32, dy: i32) {
        self.position += Point::new(dx, dy);
    }
    pub fn update(&mut self, default_index: usize) {
        self.advance();
        if !self.current().looped && self.current().is_finished() {
            self.trigger(default_index);
        }
    }
    pub fn new(animations: Vec<Animation<'a>, 4>, position: Point) -> Self {
        Self {
            animations,
            active_index: 0,
            position,
        }
    }

    pub fn current(&self) -> &Animation<'a> {
        &self.animations[self.active_index]
    }

    pub fn current_mut(&mut self) -> &mut Animation<'a> {
        &mut self.animations[self.active_index]
    }

    pub fn advance(&mut self) {
        self.current_mut().advance();
    }

    pub fn trigger(&mut self, index: usize) {
        if index < self.animations.len() {
            self.active_index = index;
            let anim = self.current_mut();
            anim.current_frame = 0;
            anim.finished = false;
        }
    }
}
