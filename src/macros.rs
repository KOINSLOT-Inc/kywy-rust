// ==========================
// Kywy Macros for Init
// ==========================

/// Initializes the `KywyDisplay` by moving the required peripherals from a `Peripherals` instance.
///
/// # Usage
/// ```rust
/// kywy_display_from!(p => display);
/// ```
/// This expands to:
/// ```rust
/// let display = KywyDisplay::new(p.SPI0, p.DMA_CH0, ..., p.PIN_22).await;
/// ```
///
/// # Inputs:
/// - `p`: the `Peripherals` instance from `embassy_rp::init(...)`
/// - `display`: the variable name to assign the created `KywyDisplay` to
///
/// # Output:
/// - Moves ownership of the SPI, DMA, and GPIO pins into `KywyDisplay::new`
/// - Returns a `KywyDisplay<'static>` bound to the given variable
#[macro_export]
macro_rules! kywy_display_from {
    ($peripherals:ident => $var:ident) => {
        let mut $var = $crate::display::KywyDisplay::new(
            $peripherals.SPI0,    // SPI bus
            $peripherals.DMA_CH0, // DMA TX
            $peripherals.DMA_CH1, // DMA RX
            $peripherals.PIN_18,  // SCK
            $peripherals.PIN_19,  // MOSI
            $peripherals.PIN_16,  // MISO
            $peripherals.PIN_17,  // CS
            $peripherals.PIN_22,  // Display DC
        )
        .await;
    };
}

/// Initializes the `Buttons` struct by moving the required GPIO pins from a `Peripherals` instance.
///
/// # Usage
/// ```rust
/// kywy_buttons_from!(spawner, p => buttons);
/// ```
/// This expands to:
/// ```rust
/// let buttons = buttons::init(spawner, p.PIN_2, p.PIN_12, ..., p.PIN_8);
/// ```
///
/// # Inputs:
/// - `spawner`: the `Spawner` from Embassy executor
/// - `p`: the `Peripherals` instance from `embassy_rp::init(...)`
/// - `buttons`: the variable name to assign the created `Buttons` to
///
/// # Output:
/// - Moves GPIO pins into `buttons::init`
/// - Returns a `Buttons` struct bound to the given variable
#[macro_export]
macro_rules! kywy_buttons_from {
    ($spawner:expr, $peripherals:ident => $var:ident) => {
        let mut $var = $crate::buttons::init(
            $spawner,
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
