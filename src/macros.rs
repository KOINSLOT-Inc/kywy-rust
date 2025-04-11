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
        use embassy_rp::gpio::{Level, Output};
        use embassy_rp::spi::Config;
        use embassy_rp::spi::Phase;
        use embassy_rp::spi::Polarity;
        use inverted_pin::InvertedPin;
        //initialize pins
        let cs_disp_pin = InvertedPin::new(Output::new($peripherals.PIN_17, Level::Low));
        let disp_pin = Output::new($peripherals.PIN_22, Level::Low);

        //initialize SPI device
        let mut config = Config::default();
        config.frequency = 1_000_000;
        config.polarity = Polarity::IdleLow;
        config.phase = Phase::CaptureOnFirstTransition;
        let disp_spi = SpiDeviceWithConfig::new(&$spi_bus, cs_disp_pin, config);

        //create display
        let mut $var = $crate::display::KywyDisplay::new(
            disp_spi, disp_pin, // DISP
        );

        //initialize display
        $var.initialize().await;
    };
}

#[macro_export]
macro_rules! kywy_button_async_from {
    ($spawner:expr, $peripherals:ident => $var:ident) => {
        let mut $var = $crate::button_async::init(
            $spawner,
            $peripherals.PIN_12, // Button: Right
            $peripherals.PIN_2,  // Button: Left
            $peripherals.PIN_9,  // Button: DUp
            $peripherals.PIN_3,  // Button: DDown
            $peripherals.PIN_6,  // Button: DLeft
            $peripherals.PIN_7,  // Button: DRight
            $peripherals.PIN_8,  // Button: DCenter
        );
    };
}

#[macro_export]
macro_rules! kywy_button_poll_from {
    ($peripherals:ident => $var:ident) => {
        let $var = $crate::button_poll::ButtonPoller::new(
            $peripherals.PIN_2,  // Button: Left
            $peripherals.PIN_12, // Button: Right
            $peripherals.PIN_9,  // Button: DUp
            $peripherals.PIN_3,  // Button: DDown
            $peripherals.PIN_6,  // Button: DLeft
            $peripherals.PIN_7,  // Button: DRight
            $peripherals.PIN_8,  // Button: DCenter
        );
    };
}

#[macro_export]
macro_rules! kywy_battery_from {
    ($peripherals:ident => $battery_var:ident) => {
        let mut $battery_var = $crate::battery::BatteryMonitor::new(
            $peripherals.PIN_26,
            $peripherals.PIN_10,
            $peripherals.PIN_11,
            $peripherals.ADC,
            embedded_graphics::geometry::Point::new(125, 0), // default battery location
            embedded_graphics::pixelcolor::BinaryColor::Off,
        )
        .await;
    };
}
