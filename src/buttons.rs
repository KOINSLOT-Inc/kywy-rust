//! Works kind of? Needs fixed

use embassy_executor::Spawner;
use embassy_rp::Peri;
use embassy_rp::gpio::{Input, Level, Pull};
use embassy_rp::peripherals::*;
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::signal::Signal;
use static_cell::StaticCell;

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

pub struct Buttons {
    pub left: &'static Signal<ThreadModeRawMutex, bool>,
    pub right: &'static Signal<ThreadModeRawMutex, bool>,
    pub dup: &'static Signal<ThreadModeRawMutex, bool>,
    pub ddown: &'static Signal<ThreadModeRawMutex, bool>,
    pub dleft: &'static Signal<ThreadModeRawMutex, bool>,
    pub dright: &'static Signal<ThreadModeRawMutex, bool>,
    pub dcenter: &'static Signal<ThreadModeRawMutex, bool>,
}

// Static signals
static LEFT_SIGNAL: StaticCell<Signal<ThreadModeRawMutex, bool>> = StaticCell::new();
static RIGHT_SIGNAL: StaticCell<Signal<ThreadModeRawMutex, bool>> = StaticCell::new();
static DUP_SIGNAL: StaticCell<Signal<ThreadModeRawMutex, bool>> = StaticCell::new();
static DDOWN_SIGNAL: StaticCell<Signal<ThreadModeRawMutex, bool>> = StaticCell::new();
static DLEFT_SIGNAL: StaticCell<Signal<ThreadModeRawMutex, bool>> = StaticCell::new();
static DRIGHT_SIGNAL: StaticCell<Signal<ThreadModeRawMutex, bool>> = StaticCell::new();
static DCENTER_SIGNAL: StaticCell<Signal<ThreadModeRawMutex, bool>> = StaticCell::new();

pub fn init(
    spawner: &Spawner,
    pin_left: Peri<'static, PIN_2>,
    pin_right: Peri<'static, PIN_12>,
    pin_dup: Peri<'static, PIN_9>,
    pin_ddown: Peri<'static, PIN_3>,
    pin_dleft: Peri<'static, PIN_6>,
    pin_dright: Peri<'static, PIN_7>,
    pin_dcenter: Peri<'static, PIN_8>,
) -> Buttons {
    // SAFELY initialize each signal exactly once
    let left = LEFT_SIGNAL.init(Signal::new());
    let right = RIGHT_SIGNAL.init(Signal::new());
    let dup = DUP_SIGNAL.init(Signal::new());
    let ddown = DDOWN_SIGNAL.init(Signal::new());
    let dleft = DLEFT_SIGNAL.init(Signal::new());
    let dright = DRIGHT_SIGNAL.init(Signal::new());
    let dcenter = DCENTER_SIGNAL.init(Signal::new());

    // Spawn tasks using reborrowed peripherals
    spawn_button(spawner, pin_left, left);
    spawn_button(spawner, pin_right, right);
    spawn_button(spawner, pin_dup, dup);
    spawn_button(spawner, pin_ddown, ddown);
    spawn_button(spawner, pin_dleft, dleft);
    spawn_button(spawner, pin_dright, dright);
    spawn_button(spawner, pin_dcenter, dcenter);

    Buttons {
        left,
        right,
        dup,
        ddown,
        dleft,
        dright,
        dcenter,
    }
}

// Accept borrowed Peri<'_>, convert to Input<'static>
fn spawn_button<P: embassy_rp::gpio::Pin + 'static>(
    spawner: &Spawner,
    pin: Peri<'static, P>,
    signal: &'static Signal<ThreadModeRawMutex, bool>,
) {
    let mut input = Input::new(pin, Pull::Up);
    input.set_schmitt(true);

    spawner.spawn(button_task(input, signal)).unwrap();
}

#[embassy_executor::task(pool_size = 7)]
async fn button_task(mut pin: Input<'static>, signal: &'static Signal<ThreadModeRawMutex, bool>) {
    loop {
        pin.wait_for_any_edge().await;
        let level = pin.get_level();
        signal.signal(level == Level::High); // true = released
    }
}
