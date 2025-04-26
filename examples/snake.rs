// SPDX-FileCopyrightText: 2025 KOINSLOT Inc.
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! examples/snake.rs
//! Snake game for Kywy

#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use panic_probe as _;

use core::cell::Cell;
use itoa::Buffer;

use kywy::button_async::{ButtonEvent, ButtonId, ButtonState};
use kywy::{kywy_button_async_from, kywy_display_from, kywy_spi_from};

use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::peripherals::USB;
use embassy_rp::usb::InterruptHandler;
use embassy_time::{Duration, Instant, Timer};

use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::FONT_6X10},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
    text::Text,
};

use embedded_graphics::image::Image;
use tinybmp::Bmp;

use heapless::Vec;
use rand::{Rng, SeedableRng, rngs::SmallRng};

const GRID_WIDTH: i32 = 144 / 4; // display size / 4 gives 4 pixel block
const GRID_HEIGHT: i32 = 168 / 4;

#[derive(Default)]
struct ButtonHeldState {
    left: Cell<bool>,
    right: Cell<bool>,
    up: Cell<bool>,
    down: Cell<bool>,

    left_just_pressed: Cell<bool>,
    right_just_pressed: Cell<bool>,
    up_just_pressed: Cell<bool>,
    down_just_pressed: Cell<bool>,
}

impl ButtonHeldState {
    fn set(&self, id: ButtonId, pressed: bool) {
        match id {
            ButtonId::DLeft => {
                self.left.set(pressed);
                if pressed {
                    self.left_just_pressed.set(true);
                }
            }
            ButtonId::DRight => {
                self.right.set(pressed);
                if pressed {
                    self.right_just_pressed.set(true);
                }
            }
            ButtonId::DUp => {
                self.up.set(pressed);
                if pressed {
                    self.up_just_pressed.set(true);
                }
            }
            ButtonId::DDown => {
                self.down.set(pressed);
                if pressed {
                    self.down_just_pressed.set(true);
                }
            }
            _ => {}
        }
    }

    /// Returns Some(Direction) if any button is held or was just pressed this tick.
    fn current_direction(&self) -> Option<Direction> {
        if self.up.get() || self.up_just_pressed.get() {
            Some(Direction::Up)
        } else if self.down.get() || self.down_just_pressed.get() {
            Some(Direction::Down)
        } else if self.left.get() || self.left_just_pressed.get() {
            Some(Direction::Left)
        } else if self.right.get() || self.right_just_pressed.get() {
            Some(Direction::Right)
        } else {
            None
        }
    }

    /// Clear "just pressed" flags at the end of the tick.
    fn clear_just_pressed(&self) {
        self.left_just_pressed.set(false);
        self.right_just_pressed.set(false);
        self.up_just_pressed.set(false);
        self.down_just_pressed.set(false);
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
struct Position {
    x: i32,
    y: i32,
}

#[derive(Copy, Clone, PartialEq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

struct Snake {
    body: Vec<Position, 64>,
    dir: Direction,
}

struct GameState {
    snake: Snake,
    food: Position,
    rng: SmallRng,
    score: u32,
}

impl GameState {
    fn new(seed: u64) -> Self {
        let mut body = Vec::new();
        body.push(Position { x: 5, y: 5 }).unwrap();
        body.push(Position { x: 4, y: 5 }).unwrap();

        let mut rng = SmallRng::seed_from_u64(seed);
        let food = Position {
            x: rng.random_range(0..GRID_WIDTH),
            y: rng.random_range(0..GRID_HEIGHT),
        };

        Self {
            snake: Snake {
                body,
                dir: Direction::Right,
            },
            food,
            rng,
            score: 0,
        }
    }

    fn update(&mut self) -> bool {
        let mut new_head = self.snake.body[0];

        match self.snake.dir {
            Direction::Up => new_head.y -= 1,
            Direction::Down => new_head.y += 1,
            Direction::Left => new_head.x -= 1,
            Direction::Right => new_head.x += 1,
        }

        // Wrap around edges
        new_head.x = (new_head.x + GRID_WIDTH) % GRID_WIDTH;
        new_head.y = (new_head.y + GRID_HEIGHT) % GRID_HEIGHT;

        let growing = new_head == self.food;

        // Check for collision with body, excluding last segment if not growing
        let body_to_check = if growing {
            &self.snake.body[..]
        } else {
            &self.snake.body[..self.snake.body.len().saturating_sub(1)]
        };

        if body_to_check.iter().any(|&seg| seg == new_head) {
            return false; // Collision with self
        }

        // Move or grow
        self.snake.body.insert(0, new_head).ok();

        if growing {
            self.spawn_food();
            self.score += 1;
        } else {
            self.snake.body.pop(); // remove tail
        }

        true
    }

    fn spawn_food(&mut self) {
        loop {
            let pos = Position {
                x: self.rng.random_range(0..GRID_WIDTH),
                y: self.rng.random_range(0..GRID_HEIGHT),
            };
            if !self.snake.body.contains(&pos) {
                self.food = pos;
                break;
            }
        }
    }

    fn change_direction(&mut self, new_dir: Direction) {
        use Direction::*;
        let opposite = matches!(
            (self.snake.dir, new_dir),
            (Up, Down) | (Down, Up) | (Left, Right) | (Right, Left)
        );

        if !opposite {
            self.snake.dir = new_dir;
        }
    }

    fn render<D: DrawTarget<Color = BinaryColor>>(&self, display: &mut D) {
        display.clear(BinaryColor::On).ok();

        let block_size = Size::new(4, 4);

        for seg in self.snake.body.iter() {
            let _ = Rectangle::new(Point::new(seg.x * 4, seg.y * 4), block_size)
                .into_styled(PrimitiveStyle::with_fill(BinaryColor::Off))
                .draw(display);
        }
        let _ = Rectangle::new(Point::new(self.food.x * 4, self.food.y * 4), block_size)
            .into_styled(PrimitiveStyle::with_fill(BinaryColor::Off))
            .draw(display);
    }

    fn render_game_over<D: DrawTarget<Color = BinaryColor>>(
        &self,
        display: &mut D,
        style: MonoTextStyle<BinaryColor>,
    ) {
        display.clear(BinaryColor::On).ok();
        let _ = Text::new("Game Over", Point::new(20, 20), style).draw(display);
        let _ = Text::new("Press Any Button", Point::new(5, 40), style).draw(display);
        let _ = Text::new("to Restart", Point::new(20, 50), style).draw(display);

        let mut buf = Buffer::new();
        let score_str = buf.format(self.score);
        let _ = Text::new("Score: ", Point::new(5, 100), style).draw(display);
        let _ = Text::new(score_str, Point::new(47, 100), style).draw(display);
    }
}

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => InterruptHandler<USB>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Snake game starting");

    let p = embassy_rp::init(Default::default());

    kywy_spi_from!(p => spi_bus);
    kywy_display_from!(spi_bus, p => display);
    kywy_button_async_from!(&spawner, p => button_channel);

    let style = MonoTextStyle::new(&FONT_6X10, BinaryColor::Off);
    display.initialize().await;
    display.enable();

    static IMAGE_DATA: &[u8] = include_bytes!("../examples/Art Assets/SnakeSplash.bmp"); // Change this path to use a different image
    let bmp = Bmp::from_slice(IMAGE_DATA).unwrap();
    let image = Image::new(&bmp, Point::zero());
    image.draw(&mut display).unwrap();
    display.write_display().await;
    Timer::after(Duration::from_millis(200)).await;

    button_channel.clear();
    let _: ButtonEvent = button_channel.receive().await; // Wait for any button press

    let held = ButtonHeldState::default();

    loop {
        let seed = Instant::now().as_ticks() as u64;
        let mut game = GameState::new(seed);

        loop {
            // Drain button events to update held state and just-pressed flags
            while let Ok(event) = button_channel.try_receive() {
                let is_pressed = event.state == ButtonState::Pressed;
                held.set(event.id, is_pressed);
            }

            // Update direction based on current state and just-pressed flags
            if let Some(new_dir) = held.current_direction() {
                game.change_direction(new_dir);
            }

            let alive = game.update();
            game.render(&mut display);
            display.write_display().await;

            // Reset "just pressed" flags so they only last one frame
            held.clear_just_pressed();

            if !alive {
                break;
            }

            Timer::after(Duration::from_millis(100)).await;
        }

        game.render_game_over(&mut display, style);
        display.write_display().await;
        Timer::after(Duration::from_millis(500)).await;

        loop {
            let ev = button_channel.receive().await;
            if ev.state == ButtonState::Pressed {
                break;
            }
        }
    }
}
