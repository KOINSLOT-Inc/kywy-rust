//! src/buttons.rs
//! Button event system using a shared Channel for press/release detection.

use embassy_executor::Spawner;
use embassy_rp::Peri;
use embassy_rp::gpio::{Input, Level, Pull};
use embassy_rp::peripherals::*;
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::Channel;

#[derive(Clone, Copy, Debug)]
pub enum ButtonId {
    Left,
    Right,
    DUp,
    DDown,
    DLeft,
    DRight,
    DCenter,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ButtonState {
    Pressed,
    Released,
}

#[derive(Clone, Copy, Debug)]
pub struct ButtonEvent {
    pub id: ButtonId,
    pub state: ButtonState,
}

const BUTTON_CHANNEL_CAPACITY: usize = 16;

static BUTTON_CHANNEL: Channel<ThreadModeRawMutex, ButtonEvent, BUTTON_CHANNEL_CAPACITY> =
    Channel::new();

pub fn init(
    spawner: &Spawner,
    pin_left: Peri<'static, PIN_12>,
    pin_right: Peri<'static, PIN_2>,
    pin_dup: Peri<'static, PIN_9>,
    pin_ddown: Peri<'static, PIN_3>,
    pin_dleft: Peri<'static, PIN_6>,
    pin_dright: Peri<'static, PIN_7>,
    pin_dcenter: Peri<'static, PIN_8>,
) -> &'static Channel<ThreadModeRawMutex, ButtonEvent, BUTTON_CHANNEL_CAPACITY> {
    spawn_button(spawner, pin_left, ButtonId::Left);
    spawn_button(spawner, pin_right, ButtonId::Right);
    spawn_button(spawner, pin_dup, ButtonId::DUp);
    spawn_button(spawner, pin_ddown, ButtonId::DDown);
    spawn_button(spawner, pin_dleft, ButtonId::DLeft);
    spawn_button(spawner, pin_dright, ButtonId::DRight);
    spawn_button(spawner, pin_dcenter, ButtonId::DCenter);

    &BUTTON_CHANNEL
}

fn spawn_button<P: embassy_rp::gpio::Pin + 'static>(
    spawner: &Spawner,
    pin: Peri<'static, P>,
    id: ButtonId,
) {
    let mut input = Input::new(pin, Pull::Up);
    input.set_schmitt(true);
    spawner.spawn(button_task(input, id)).unwrap();
}

#[embassy_executor::task(pool_size = 7)] // spawns 7 tasks, one for each button
async fn button_task(mut pin: Input<'static>, id: ButtonId) {
    loop {
        pin.wait_for_any_edge().await;
        let level = pin.get_level();

        let state = if level == Level::Low {
            ButtonState::Pressed
        } else {
            ButtonState::Released
        };

        let _ = BUTTON_CHANNEL.try_send(ButtonEvent { id, state });
    }
}
