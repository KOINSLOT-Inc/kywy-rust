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

use embedded_icon::{
    NewIcon,
    iconoir::size24px::{
        Battery25, Battery50, Battery75, BatteryCharging, BatteryEmpty, BatteryFull,
    },
};

bind_interrupts!(struct Irqs {
    ADC_IRQ_FIFO => adc::InterruptHandler;
});

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum BatteryStatus {
    Charging,
    Charged,
    NotCharging,
}

pub struct BatteryMonitor<'a> {
    adc: Adc<'a, Async>,
    channel: Channel<'a>,
    charging: Input<'a>,
    standby: Input<'a>,
    position: Point,
    color: BinaryColor,
}

impl<'a> BatteryMonitor<'a> {
    pub fn new(
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

        Self {
            adc,
            channel,
            charging,
            standby,
            position,
            color,
        }
    }

    pub async fn read_voltage_mv(&mut self) -> u16 {
        let raw = self.adc.read(&mut self.channel).await.unwrap_or(0) as f32;
        let adc_mv = (raw / 4095.0) * 3.3 * 1000.0;
        (adc_mv * 2.0) as u16 // voltage divider
    }

    pub async fn battery_percentage(&mut self) -> u8 {
        let mv = self.read_voltage_mv().await;
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
        let standby = self.standby.is_high();

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
                let icon = BatteryCharging::new(color);
                Image::new(&icon, position).draw(display)
            }
            BatteryStatus::Charged => {
                let icon = BatteryFull::new(color);
                Image::new(&icon, position).draw(display)
            }
            BatteryStatus::NotCharging => match percent {
                85..=100 => {
                    let icon = BatteryFull::new(color);
                    Image::new(&icon, position).draw(display)
                }
                60..=84 => {
                    let icon = Battery75::new(color);
                    Image::new(&icon, position).draw(display)
                }
                25..=59 => {
                    let icon = Battery50::new(color);
                    Image::new(&icon, position).draw(display)
                }
                5..=24 => {
                    let icon = Battery25::new(color);
                    Image::new(&icon, position).draw(display)
                }
                _ => {
                    let icon = BatteryEmpty::new(color);
                    Image::new(&icon, position).draw(display)
                }
            },
        }
    }
}

impl OriginDimensions for BatteryMonitor<'_> {
    fn size(&self) -> Size {
        Size::new(24, 24)
    }
}
