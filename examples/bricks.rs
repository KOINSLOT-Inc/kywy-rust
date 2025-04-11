// SPDX-FileCopyrightText: 2025 KOINSLOT Inc.
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! examples/bricks.rs

#![no_std]
#![no_main]

//! Arkanoid clone for kywy

use defmt::*;
use defmt_rtt as _;
use panic_probe as _;

use kywy::button_async::{ButtonEvent, ButtonId, ButtonState};
use kywy::display::KywyDisplay;
use kywy::{kywy_button_async_from, kywy_display_from, kywy_spi_from};

use embedded_graphics::{
    Drawable,
    image::Image,
    mono_font::{MonoTextStyle, ascii::FONT_6X10},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
    text::Text,
};

use embassy_executor::Spawner;
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::Receiver;
use embassy_time::{Duration, Instant, Timer};
use embedded_graphics::prelude::DrawTarget;
use embedded_hal_async::spi::SpiDevice;
use heapless::Vec;
use micromath::F32Ext;
use tinybmp::Bmp;

// Screen and layout
const SCREEN_WIDTH: i32 = 144;
const SCREEN_HEIGHT: i32 = 168;
const SCORE_HEIGHT: i32 = 10;
const PADDLE_Y: i32 = SCREEN_HEIGHT - SCORE_HEIGHT - 6;

// Game parameters
const PADDLE_WIDTH: i32 = 24;
const PADDLE_HEIGHT: i32 = 4;
const BALL_SIZE: i32 = 5;
const BALL_SPEED_MIN: f32 = 3.0;
const BALL_SPEED_MAX: f32 = 6.0;
const BALL_VEL_Y_MIN: f32 = 0.5;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Starting Bricks...");

    let p = embassy_rp::init(Default::default());
    kywy_spi_from!(p => spi_bus);
    kywy_display_from!(spi_bus, p => display);

    display.enable();

    kywy_button_async_from!(&spawner, p => buttons);
    let mut receiver = buttons.receiver();

    static IMAGE_DATA: &[u8] = include_bytes!("../examples/Art Assets/Bricks.bmp");
    let bmp = Bmp::from_slice(IMAGE_DATA).unwrap();
    let width = bmp.size().width as i32;
    let height = bmp.size().height as i32;
    let pos = Point::new((SCREEN_WIDTH - width) / 2, (SCREEN_HEIGHT - height) / 2);
    let image = Image::new(&bmp, pos);
    image.draw(&mut display).unwrap();
    display.write_display().await;
    Timer::after(Duration::from_millis(200)).await;
    buttons.clear();
    let _: ButtonEvent = buttons.receive().await;

    loop {
        run_bricks(&mut display, &mut receiver).await;
        display.write_display().await;
        wait_for_button(&mut receiver).await;
    }
}

struct Ball {
    pos: Point,
    vel: Point,
}

struct Paddle {
    pos: Point,
}

struct Brick {
    rect: Rectangle,
    alive: bool,
}

async fn run_bricks<SPI: SpiDevice>(
    display: &mut KywyDisplay<'_, SPI>,
    button_channel: &mut Receiver<'static, ThreadModeRawMutex, ButtonEvent, 16>,
) {
    let mut paddle = Paddle {
        pos: Point::new((SCREEN_WIDTH - PADDLE_WIDTH) / 2, PADDLE_Y),
    };

    let mut ball = Ball {
        pos: Point::new(SCREEN_WIDTH / 2, SCREEN_HEIGHT / 2),
        vel: Point::new(1, -1),
    };

    let mut bricks = create_bricks();
    let mut score = 0;
    let total_bricks = bricks.len() as u32;

    let mut tick_delay_ms = 40u64;
    let mut held_left = false;
    let mut held_right = false;
    let mut last_paddle_update = Instant::now();

    loop {
        while let Ok(event) = button_channel.try_receive() {
            match (event.id, event.state) {
                (ButtonId::DLeft, ButtonState::Pressed) => held_left = true,
                (ButtonId::DLeft, ButtonState::Released) => held_left = false,
                (ButtonId::DRight, ButtonState::Pressed) => held_right = true,
                (ButtonId::DRight, ButtonState::Released) => held_right = false,
                _ => {}
            }
        }

        if Instant::now().duration_since(last_paddle_update) >= Duration::from_millis(30) {
            let paddle_speed = 3;
            if held_left {
                paddle.pos.x = (paddle.pos.x - paddle_speed).max(0);
            }
            if held_right {
                paddle.pos.x = (paddle.pos.x + paddle_speed).min(SCREEN_WIDTH - PADDLE_WIDTH);
            }
            last_paddle_update = Instant::now();
        }

        ball.pos += ball.vel;
        if ball.vel.x == 0 {
            ball.vel.x = 1;
        }

        if ball.pos.x <= 0 || ball.pos.x >= SCREEN_WIDTH - BALL_SIZE {
            ball.vel.x = -ball.vel.x;
        }
        if ball.pos.y <= 0 {
            ball.vel.y = -ball.vel.y;
        }

        let paddle_rect = Rectangle::new(
            paddle.pos,
            Size::new(PADDLE_WIDTH as u32, PADDLE_HEIGHT as u32),
        );
        let ball_rect = Rectangle::new(ball.pos, Size::new(BALL_SIZE as u32, BALL_SIZE as u32));

        if paddle_rect.intersection(&ball_rect).size != Size::new(0, 0) {
            ball.vel.y = -ball.vel.y;
            let paddle_center = paddle.pos.x + PADDLE_WIDTH / 2;
            let ball_center = ball.pos.x + BALL_SIZE / 2;
            let diff = ball_center - paddle_center;
            ball.vel.x = diff.clamp(-4, 4);
            ball.vel = normalize_velocity(ball.vel, ball_speed_from_score(score, total_bricks));
        }

        for brick in &mut bricks {
            if brick.alive && brick.rect.intersection(&ball_rect).size != Size::new(0, 0) {
                brick.alive = false;
                ball.vel.y = -ball.vel.y;
                ball.vel = normalize_velocity(ball.vel, ball_speed_from_score(score, total_bricks));
                score += 1;
                break;
            }
        }

        if score == total_bricks {
            draw_message(display, "YOU WIN!");
            display.write_display().await;
            Timer::after_secs(2).await;
            return;
        }

        if ball.pos.y >= SCREEN_HEIGHT {
            draw_message(display, "GAME OVER");
            display.write_display().await;
            Timer::after_secs(2).await;
            return;
        }

        display.clear_buffer(BinaryColor::On);
        draw_ball(display, &ball);
        draw_paddle(display, &paddle);
        draw_bricks(display, &bricks);
        draw_score(display, score);
        display.write_display().await;

        if tick_delay_ms > 10 {
            tick_delay_ms -= 1;
        }

        Timer::after_millis(tick_delay_ms).await;
    }
}

fn ball_speed_from_score(score: u32, total: u32) -> f32 {
    let t = score as f32 / total.max(1) as f32;
    BALL_SPEED_MIN + (BALL_SPEED_MAX - BALL_SPEED_MIN) * t
}

fn normalize_velocity(vel: Point, target_speed: f32) -> Point {
    let dx = vel.x as f32;
    let dy = vel.y as f32;
    let mag = (dx * dx + dy * dy).sqrt();
    if mag == 0.0 {
        return Point::new(1, -1);
    }

    let mut vx = dx / mag;
    let mut vy = dy / mag;

    if vy.abs() < BALL_VEL_Y_MIN {
        vy = BALL_VEL_Y_MIN.copysign(vy);
        vx = (1.0 - vy * vy).sqrt().copysign(vx);
    }

    let scale = target_speed;
    Point::new((vx * scale).round() as i32, (vy * scale).round() as i32)
}

fn format_score(score: u32) -> heapless::String<16> {
    use core::fmt::Write as FmtWrite;
    let mut s = heapless::String::<16>::new();
    let _ = FmtWrite::write_fmt(&mut s, format_args!("Score: {}", score));
    s
}

fn draw_ball<D: DrawTarget<Color = BinaryColor>>(display: &mut D, ball: &Ball) {
    Rectangle::new(ball.pos, Size::new(BALL_SIZE as u32, BALL_SIZE as u32))
        .into_styled(PrimitiveStyle::with_fill(BinaryColor::Off))
        .draw(display)
        .ok();
}

fn draw_paddle<D: DrawTarget<Color = BinaryColor>>(display: &mut D, paddle: &Paddle) {
    Rectangle::new(
        paddle.pos,
        Size::new(PADDLE_WIDTH as u32, PADDLE_HEIGHT as u32),
    )
    .into_styled(PrimitiveStyle::with_fill(BinaryColor::Off))
    .draw(display)
    .ok();
}

fn draw_bricks<D: DrawTarget<Color = BinaryColor>>(display: &mut D, bricks: &[Brick]) {
    for brick in bricks.iter().filter(|b| b.alive) {
        brick
            .rect
            .into_styled(PrimitiveStyle::with_fill(BinaryColor::Off))
            .draw(display)
            .ok();
    }
}

fn draw_score<D: DrawTarget<Color = BinaryColor>>(display: &mut D, score: u32) {
    let style = MonoTextStyle::new(&FONT_6X10, BinaryColor::Off);
    let msg = format_score(score);
    let _ = Text::new(&msg, Point::new(5, SCREEN_HEIGHT - 2), style).draw(display);
}

fn draw_message<D: DrawTarget<Color = BinaryColor>>(display: &mut D, msg: &str) {
    let style = MonoTextStyle::new(&FONT_6X10, BinaryColor::Off);
    let _ = Text::new(msg, Point::new(20, SCREEN_HEIGHT / 2), style).draw(display);
}

async fn wait_for_button(
    button_channel: &mut Receiver<'static, ThreadModeRawMutex, ButtonEvent, 16>,
) {
    loop {
        if let Ok(event) = button_channel.try_receive() {
            if matches!(event.state, ButtonState::Pressed) {
                break;
            }
        }
        Timer::after_millis(50).await;
    }
}

fn create_bricks() -> Vec<Brick, 32> {
    let mut bricks = Vec::new();
    let brick_width = 14;
    let brick_height = 8;
    let cols = 8;
    let rows = 4;
    let padding = 2;
    let total_width = cols * (brick_width + padding);
    let offset_x = (SCREEN_WIDTH - total_width as i32) / 2;

    for row in 0..rows {
        for col in 0..cols {
            let x = offset_x + col as i32 * (brick_width + padding);
            let y = 4 + row as i32 * (brick_height + padding);
            let rect = Rectangle::new(
                Point::new(x, y),
                Size::new(brick_width as u32, brick_height as u32),
            );
            bricks.push(Brick { rect, alive: true }).ok();
        }
    }
    bricks
}
