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

impl<'a> Sprite<'a> {
    pub fn draw<D>(&self, target: &mut D, pos: Point) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = BinaryColor>,
    {
        let sprite_width = self.sheet.sprite_size.width;
        let sprite_height = self.sheet.sprite_size.height;
        let mut frame_buffer = [0u8; 64 * 64 / 8];

        for pixel in self.sheet.bmp.pixels() {
            let Point { x, y } = pixel.0;
            let within_x = x >= self.offset.x && x < self.offset.x + sprite_width as i32;
            let within_y = y >= self.offset.y && y < self.offset.y + sprite_height as i32;

            if within_x && within_y {
                let local_x = (x - self.offset.x) as usize;
                let local_y = (y - self.offset.y) as usize; // remove vertical flip
                let idx = local_y * sprite_width as usize + local_x;
                if pixel.1.is_on() {
                    let byte = idx / 8;
                    let bit = 7 - (idx % 8);
                    frame_buffer[byte] |= 1 << bit;
                }
            }
        }

        let sprite_img = ImageRaw::<BinaryColor>::new(
            &frame_buffer[..(sprite_width as usize * sprite_height as usize / 8)],
            sprite_width,
        );
        let img = Image::new(&sprite_img, pos);
        img.draw(target)
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
        self.position = self.position + Point::new(dx, dy);
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
        }
    }
}
