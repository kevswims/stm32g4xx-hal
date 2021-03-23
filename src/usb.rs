//! USB Peripheral
//!
//! Requires the `stm32-usbd` feature

// use crate::{gpio::{Analog, Output}};
use crate::{stm32::{RCC, USB}};
use stm32_usbd::UsbPeripheral;

// use crate::gpio::gpioa::{PA11, PA12};
pub use stm32_usbd::UsbBus;

/// USB Peripheral
///
/// Constructs the peripheral, which
/// then gets passed to the [`UsbBus`].
pub struct Peripheral {
    /// USB Register Block
    pub usb: USB,
    // /// Data Negative Pin
    // pub pin_dm: PA11<Output<Analog>>,
    // /// Data Positiv Pin
    // pub pin_dp: PA12<Output<Analog>>,
}

unsafe impl Sync for Peripheral {}

unsafe impl UsbPeripheral for Peripheral {
    const REGISTERS: *const () = USB::ptr() as *const ();
    const DP_PULL_UP_FEATURE: bool = true;
    const EP_MEMORY: *const () = 0x4000_6000 as _;
    const EP_MEMORY_SIZE: usize = 1024;

    fn enable() {
        let rcc = unsafe { &*RCC::ptr() };

        cortex_m::interrupt::free(|_| {
            // Enable USB peripheral
            rcc.apb1enr1.modify(|_, w| w.usben().enabled());

            // Reset USB peripheral
            rcc.apb1rstr1.modify(|_, w| w.usbrst().reset());
            rcc.apb1rstr1.modify(|_, w| w.usbrst().clear_bit());
        });
    }

    fn startup_delay() {
        // There is a chip specific startup delay. It is not specified for STM32G473 but the STM32F103
        // is 1us so delay for 170 cycles minimum
        cortex_m::asm::delay(170);
    }
}

/// Type of the UsbBus
///
/// As this MCU family has only USB peripheral,
/// this is the only possible concrete type construction.
pub type UsbBusType = UsbBus<Peripheral>;