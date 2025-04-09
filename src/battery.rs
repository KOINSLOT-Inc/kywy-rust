use embassy_rp::{
    Peri,
    adc::{self, Adc, Async, Channel, Config as AdcConfig},
    bind_interrupts,
    gpio::{Input, Pull},
    peripherals::{ADC, PIN_10, PIN_11, PIN_26},
};
use embedded_graphics::{
    draw_target::DrawTarget,
    geometry::{OriginDimensions, Point, Size},
    image::Image,
    pixelcolor::BinaryColor,
    prelude::*,
};
use embedded_iconoir::prelude::*;
use heapless::Vec;

bind_interrupts!(struct Irqs {
    ADC_IRQ_FIFO => adc::InterruptHandler;
});

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum BatteryStatus {
    Charging,
    Charged,
    NotCharging,
}

const VOLTAGE_AVG_SAMPLES: usize = 8;

pub struct BatteryMonitor<'a> {
    adc: Adc<'a, Async>,
    channel: Channel<'a>,
    charging: Input<'a>,
    standby: Input<'a>,
    position: Point,
    color: BinaryColor,
    voltage_buffer: Vec<u16, VOLTAGE_AVG_SAMPLES>,
    last_percent: u8,
}

impl<'a> BatteryMonitor<'a> {
    pub async fn new(
        adc_pin: Peri<'a, PIN_26>,
        charging_pin: Peri<'a, PIN_10>,
        standby_pin: Peri<'a, PIN_11>,
        adc_periph: Peri<'a, ADC>,
        position: Point,
        color: BinaryColor,
    ) -> Self {
        let adc = Adc::new(adc_periph, Irqs, AdcConfig::default());
        let channel = Channel::new_pin(adc_pin, Pull::None);
        let charging = Input::new(charging_pin, Pull::Up);
        let standby = Input::new(standby_pin, Pull::Up);
        let mut voltage_buffer: Vec<u16, VOLTAGE_AVG_SAMPLES> = Vec::new();

        let mut temp_monitor = BatteryMonitor {
            adc,
            channel,
            charging,
            standby,
            position,
            color,
            voltage_buffer: Vec::new(),
            last_percent: 100, // temporary
        };

        for _ in 0..VOLTAGE_AVG_SAMPLES {
            let raw = temp_monitor
                .adc
                .read(&mut temp_monitor.channel)
                .await
                .unwrap_or(0);
            let adc_mv = raw as u32 * 3300 / 4095;
            let mv = (adc_mv * 2) as u16;
            voltage_buffer.push(mv).ok();
        }

        let initial_mv = *voltage_buffer.last().unwrap_or(&4200);
        let initial_percent = Self::voltage_to_percent(initial_mv);

        BatteryMonitor {
            adc: temp_monitor.adc,
            channel: temp_monitor.channel,
            charging: temp_monitor.charging,
            standby: temp_monitor.standby,
            position,
            color,
            voltage_buffer,
            last_percent: initial_percent,
        }
    }

    pub fn move_to(&mut self, new_position: Point) {
        self.position = new_position;
    }

    fn update_voltage_buffer(&mut self, mv: u16) -> u16 {
        if self.voltage_buffer.len() == VOLTAGE_AVG_SAMPLES {
            self.voltage_buffer.remove(0);
        }
        self.voltage_buffer.push(mv).ok();
        let sum: u32 = self.voltage_buffer.iter().copied().map(|v| v as u32).sum();
        (sum / self.voltage_buffer.len() as u32) as u16
    }

    pub async fn read_voltage_mv(&mut self) -> u16 {
        let raw = self.adc.read(&mut self.channel).await.unwrap_or(0);
        let adc_mv = raw as u32 * 3300 / 4095;
        let mv = (adc_mv * 2) as u16;
        self.update_voltage_buffer(mv)
    }

    pub async fn battery_percentage(&mut self) -> u8 {
        let mv = self.read_voltage_mv().await;
        let raw_percent = Self::voltage_to_percent(mv);

        let hysteresis = 2;
        if (raw_percent as i16 - self.last_percent as i16).abs() > hysteresis {
            self.last_percent = raw_percent;
        }

        self.last_percent
    }

    fn voltage_to_percent(mv: u16) -> u8 {
        match mv {
            v if v >= 4200 => 100,
            v if v >= 3900 => 85 + ((v - 3900) * 15 / 300) as u8,
            v if v >= 3600 => 60 + ((v - 3600) * 25 / 300) as u8,
            v if v >= 3300 => 25 + ((v - 3300) * 35 / 300) as u8,
            v if v >= 3100 => 5 + ((v - 3100) * 20 / 200) as u8,
            v if v >= 3000 => ((v - 3000) * 5 / 100) as u8,
            _ => 0,
        }
    }

    pub fn status(&self) -> BatteryStatus {
        let charging = self.charging.is_low();
        let standby = self.standby.is_low();

        match (charging, standby) {
            (true, _) => BatteryStatus::Charging,
            (false, true) => BatteryStatus::Charged,
            _ => BatteryStatus::NotCharging,
        }
    }

    pub async fn draw_async<D>(&mut self, display: &mut D) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = BinaryColor>,
    {
        let percent = self.battery_percentage().await;
        let status = self.status();
        let position = self.position;
        let color = self.color;

        match status {
            BatteryStatus::Charging => {
                let icon = icons::size16px::system::BatteryCharging::new(color);
                Image::new(&icon, position).draw(display)
            }
            BatteryStatus::Charged => {
                let icon = icons::size16px::system::BatteryFull::new(color);
                Image::new(&icon, position).draw(display)
            }
            BatteryStatus::NotCharging => match percent {
                85..=100 => Image::new(&icons::size16px::system::BatteryFull::new(color), position)
                    .draw(display),
                60..=84 => Image::new(
                    &icons::size16px::system::BatterySevenFive::new(color),
                    position,
                )
                .draw(display),
                25..=59 => Image::new(
                    &icons::size16px::system::BatteryFiveZero::new(color),
                    position,
                )
                .draw(display),
                5..=24 => Image::new(
                    &icons::size16px::system::BatteryTwoFive::new(color),
                    position,
                )
                .draw(display),
                _ => Image::new(&icons::size16px::system::BatteryEmpty::new(color), position)
                    .draw(display),
            },
        }
    }
}

impl OriginDimensions for BatteryMonitor<'_> {
    fn size(&self) -> Size {
        Size::new(16, 16)
    }
}
