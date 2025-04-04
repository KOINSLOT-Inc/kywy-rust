//! examples/bricks.rs
//! Arkanoid clone for Kywy device

#![no_std]
#![no_main]

//! Bricks game for Kywy device (breakout-style)
//! Build with: cargo build --release --example bricks --target thumbv6m-none-eabi

use defmt::*;
use defmt_rtt as _;
use panic_probe as _;

use kywy::button_async::{ButtonEvent, ButtonId, ButtonState};
use kywy::display::KywyDisplay;
use kywy::{kywy_button_async_from, kywy_display_from};

use embedded_graphics::{
    Drawable,
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
use heapless::Vec;

// Display size and layout constants
const SCREEN_WIDTH: i32 = 144;
const SCREEN_HEIGHT: i32 = 168;
const SCORE_HEIGHT: i32 = 10;
const PADDLE_Y: i32 = SCREEN_HEIGHT - SCORE_HEIGHT - 6;

const PADDLE_WIDTH: i32 = 24;
const PADDLE_HEIGHT: i32 = 4;

const BALL_SIZE: i32 = 5;
const BALL_SPEED_MIN: i32 = 1;
const BALL_SPEED_MAX: i32 = 4;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Starting Bricks...");

    let p = embassy_rp::init(Default::default());

    kywy_display_from!(p => display);
    display.initialize().await;
    display.enable();

    kywy_button_async_from!(&spawner, p => buttons);

    let mut receiver = buttons.receiver();

    loop {
        run_bricks(&mut display, &mut receiver).await;
        draw_message(&mut display, "Press to Restart");
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

async fn run_bricks(
    display: &mut KywyDisplay<'_>,
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
    let mut hold_timer = 0;
    let mut last_paddle_update = Instant::now();

    loop {
        // INPUT
        while let Ok(event) = button_channel.try_receive() {
            match (event.id, event.state) {
                (ButtonId::DLeft, ButtonState::Pressed) => held_left = true,
                (ButtonId::DLeft, ButtonState::Released) => held_left = false,
                (ButtonId::DRight, ButtonState::Pressed) => held_right = true,
                (ButtonId::DRight, ButtonState::Released) => held_right = false,
                _ => {}
            }
        }

        // Paddle movement timing
        if Instant::now().duration_since(last_paddle_update) >= Duration::from_millis(30) {
            let paddle_speed = if hold_timer > 10 { 3 } else { 1 };

            if held_left {
                paddle.pos.x = (paddle.pos.x - paddle_speed).max(0);
            }
            if held_right {
                paddle.pos.x = (paddle.pos.x + paddle_speed).min(SCREEN_WIDTH - PADDLE_WIDTH);
            }

            if held_left || held_right {
                hold_timer += 1;
            } else {
                hold_timer = 0;
            }

            last_paddle_update = Instant::now();
        }

        // MOVE BALL
        ball.pos += ball.vel;

        // Fix ball getting stuck bouncing vertically
        if ball.vel.x == 0 {
            ball.vel.x = 1;
        }

        // WALL COLLISIONS
        if ball.pos.x <= 0 || ball.pos.x >= SCREEN_WIDTH - BALL_SIZE {
            ball.vel.x = -ball.vel.x;
        }
        if ball.pos.y <= 0 {
            ball.vel.y = -ball.vel.y;
        }

        // PADDLE COLLISION
        let paddle_rect = Rectangle::new(
            paddle.pos,
            Size::new(PADDLE_WIDTH as u32, PADDLE_HEIGHT as u32),
        );
        let ball_rect = Rectangle::new(ball.pos, Size::new(BALL_SIZE as u32, BALL_SIZE as u32));

        if paddle_rect.intersection(&ball_rect).size != Size::new(0, 0) {
            ball.vel.y = -ball.vel.y;

            if ball.vel.y < 0 {
                ball.vel.y = -ball.vel.y.clamp(BALL_SPEED_MIN, BALL_SPEED_MAX);
            } else {
                ball.vel.y = ball.vel.y.clamp(BALL_SPEED_MIN, BALL_SPEED_MAX);
            }

            let paddle_center = paddle.pos.x + PADDLE_WIDTH / 2;
            let ball_center = ball.pos.x + BALL_SIZE / 2;
            let diff = ball_center - paddle_center;
            ball.vel.x = diff.clamp(-BALL_SPEED_MAX, BALL_SPEED_MAX);
        }

        // BRICK COLLISIONS
        for brick in &mut bricks {
            if brick.alive && brick.rect.intersection(&ball_rect).size != Size::new(0, 0) {
                brick.alive = false;

                if ball.vel.y < 0 {
                    ball.vel.y = (-ball.vel.y + 1).clamp(-BALL_SPEED_MAX, -BALL_SPEED_MIN);
                } else {
                    ball.vel.y = (ball.vel.y + 1).clamp(BALL_SPEED_MIN, BALL_SPEED_MAX);
                }
                ball.vel.y = -ball.vel.y;

                score += 1;
                break;
            }
        }

        // WIN
        if score == total_bricks {
            draw_message(display, "YOU WIN!");
            display.write_display().await;
            Timer::after_secs(2).await;
            return;
        }

        // GAME OVER
        if ball.pos.y >= SCREEN_HEIGHT {
            draw_message(display, "GAME OVER");
            display.write_display().await;
            Timer::after_secs(2).await;
            return;
        }

        // RENDER
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

fn draw_ball(display: &mut KywyDisplay<'_>, ball: &Ball) {
    Rectangle::new(ball.pos, Size::new(BALL_SIZE as u32, BALL_SIZE as u32))
        .into_styled(PrimitiveStyle::with_fill(BinaryColor::Off))
        .draw(display)
        .ok();
}

fn draw_paddle(display: &mut KywyDisplay<'_>, paddle: &Paddle) {
    Rectangle::new(
        paddle.pos,
        Size::new(PADDLE_WIDTH as u32, PADDLE_HEIGHT as u32),
    )
    .into_styled(PrimitiveStyle::with_fill(BinaryColor::Off))
    .draw(display)
    .ok();
}

fn draw_bricks(display: &mut KywyDisplay<'_>, bricks: &[Brick]) {
    for brick in bricks.iter().filter(|b| b.alive) {
        brick
            .rect
            .into_styled(PrimitiveStyle::with_fill(BinaryColor::Off))
            .draw(display)
            .ok();
    }
}

fn draw_score(display: &mut KywyDisplay<'_>, score: u32) {
    let style = MonoTextStyle::new(&FONT_6X10, BinaryColor::Off);
    let msg = format_score(score);
    let _ = Text::new(&msg, Point::new(5, SCREEN_HEIGHT - 2), style).draw(display);
}

fn format_score(score: u32) -> heapless::String<16> {
    use core::fmt::Write as FmtWrite;
    let mut s = heapless::String::<16>::new();
    let _ = FmtWrite::write_fmt(&mut s, format_args!("Score: {}", score));
    s
}

fn draw_message(display: &mut KywyDisplay<'_>, msg: &str) {
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
