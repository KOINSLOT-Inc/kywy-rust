// SPDX-FileCopyrightText: 2025 KOINSLOT Inc.
//
// SPDX-License-Identifier: GPL-3.0-or-later

// ==========================
// Kywy Macros for Init
// ==========================

#[macro_export]
macro_rules! kywy_spi_from {
    ($peripherals:ident => $valname:ident) => {
        let $valname = {
            use embassy_rp::spi::{Async, Config, Spi};
            use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
            use embassy_sync::mutex::Mutex;
            use static_cell::StaticCell;

            // NOTE: we declare a STATIC cell with a different name than the return value
            static SPI_BUS_STATIC: StaticCell<
                Mutex<CriticalSectionRawMutex, Spi<'static, embassy_rp::peripherals::SPI0, Async>>,
            > = StaticCell::new();

            let spi = Spi::new(
                $peripherals.SPI0,
                $peripherals.PIN_18,
                $peripherals.PIN_19,
                $peripherals.PIN_16,
                $peripherals.DMA_CH0,
                $peripherals.DMA_CH1,
                Config::default(),
            );

            let mutex = Mutex::new(spi);
            let $valname = SPI_BUS_STATIC.init(mutex);
            $valname
        };
    };
}

#[macro_export]
macro_rules! kywy_display_from {
    ($spi_bus:ident, $peripherals:ident => $var:ident) => {
        use embassy_embedded_hal::shared_bus::asynch::spi::SpiDeviceWithConfig;
        use embassy_rp::Peripheral;
        use embassy_rp::gpio::{Level, Output};
        use embassy_rp::spi::Config;
        use embassy_rp::spi::Phase;
        use embassy_rp::spi::Polarity;
        use inverted_pin::InvertedPin;

        //initialize pins
        let cs_disp_pin = InvertedPin::new(Output::new($peripherals.PIN_17.into_ref(), Level::Low));
        let disp_pin = Output::new($peripherals.PIN_22.into_ref(), Level::Low);

        //initialize SPI device
        let mut config = Config::default();
        config.frequency = 1_000_000; // Try overclocking me! some displays can handle higher frequencies (~8_000_000), 2-4 MHz seems mostly stable but the spec says 1 MHz so leaving it there for now.
        config.polarity = Polarity::IdleLow;
        config.phase = Phase::CaptureOnFirstTransition;
        let mut disp_spi = SpiDeviceWithConfig::new($spi_bus, cs_disp_pin, config);

        //create display
        let mut $var = $crate::display::KywyDisplay::new(
            disp_spi, disp_pin, // DISP
        );

        //initialize display
        $var.initialize().await;
    };
}

// to do: actually write a library that will handle USB tasks to listen
#[macro_export]
macro_rules! kywy_serial_usb_from {
    ($peripherals:ident, $irqs:ident => $usb_var:ident, $serial_var:ident, $usb_fut:ident, $serial_fut:ident) => {{
        use embassy_rp::peripherals::USB;
        use embassy_rp::usb::Driver;
        use embassy_usb::class::cdc_acm::CdcAcmClass;
        use embassy_usb::class::cdc_acm::State;
        use embassy_usb::{Builder, Config as USBConfig};
        use log::info;

        let mut config_descriptor = [0; 256];
        let mut bos_descriptor = [0; 256];
        let mut control_buf = [0; 64];
        let mut state = State::new();

        let driver = Driver::new($peripherals.USB, $irqs);

        let mut config = USBConfig::new(0xc0de, 0xcafe);
        config.manufacturer = Some("Kywy Devices");
        config.product = Some("Kywy Serial");
        config.serial_number = Some("KYWY-0001");
        config.max_power = 100;
        config.max_packet_size_0 = 64;

        let mut builder = Builder::new(
            driver,
            config,
            &mut config_descriptor,
            &mut bos_descriptor,
            &mut [],
            &mut control_buf,
        );

        let mut $serial_var = CdcAcmClass::new(&mut builder, &mut state, 64);
        let mut $usb_var = builder.build();

        let mut buf = [0; 64];

        let $usb_fut = $usb_var.run();

        let $serial_fut = async {
            loop {
                $serial_var.wait_connection().await;
                info!("Connected");
                loop {
                    match $serial_var.read_packet(&mut buf).await {
                        Ok(n) if n > 0 => {
                            let _ = $serial_var.write_packet(&buf[..n]).await;
                        }
                        _ => {
                            info!("Disconnected");
                            break;
                        }
                    }
                }
            }
        };
    }};
}

#[macro_export]
macro_rules! kywy_button_async_from {
    ($spawner:expr, $peripherals:ident => $var:ident) => {
        let mut $var = $crate::button_async::init(
            $spawner,
            $peripherals.PIN_12.into_ref(), // Button: Left
            $peripherals.PIN_2.into_ref(),  // Button: Right
            $peripherals.PIN_9.into_ref(),  // Button: DUp
            $peripherals.PIN_3.into_ref(),  // Button: DDown
            $peripherals.PIN_6.into_ref(),  // Button: DLeft
            $peripherals.PIN_7.into_ref(),  // Button: DRight
            $peripherals.PIN_8.into_ref(),  // Button: DCenter
        );
    };
}

#[macro_export]
macro_rules! kywy_button_poll_from {
    ($peripherals:ident => $var:ident) => {
        let $var = $crate::button_poll::ButtonPoller::new(
            $peripherals.PIN_12.into_ref(), // Button: Left
            $peripherals.PIN_2.into_ref(),  // Button: Right
            $peripherals.PIN_9.into_ref(),  // Button: DUp
            $peripherals.PIN_3.into_ref(),  // Button: DDown
            $peripherals.PIN_6.into_ref(),  // Button: DLeft
            $peripherals.PIN_7.into_ref(),  // Button: DRight
            $peripherals.PIN_8.into_ref(),  // Button: DCenter
        );
    };
}

#[macro_export]
macro_rules! kywy_battery_from {
    ($peripherals:ident => $battery_var:ident) => {
        use embedded_graphics::geometry::Point;
        let mut $battery_var = $crate::battery::BatteryMonitor::new(
            $peripherals.PIN_26.into_ref(),
            $peripherals.PIN_10.into_ref(),
            $peripherals.PIN_11.into_ref(),
            $peripherals.ADC.into_ref(),
            Point::new(125, 0), // default battery location
            embedded_graphics::pixelcolor::BinaryColor::Off,
        )
        .await;
    };
}
