use crate::stm32::{
    rcc::{self, cfgr},
    RCC,
};

use crate::flash::ACR;
use crate::time::Hertz;
use crate::time::U32Ext;

pub trait RccExt {
    fn constrain(self) -> Rcc;
}

impl RccExt for RCC {
    fn constrain(self) -> Rcc {
        Rcc {
            ahb1: AHB1 { _0: () },
            ahb2: AHB2 { _0: () },
            ahb3: AHB3 { _0: () },
            // apb1: APB1 { _0: () },
            // apb2: APB2 { _0: () },
            // bdcr: BDCR { _0: () },
            cfgr: CFGR::default(),
        }
    }
}

pub struct Rcc {
    pub ahb1: AHB1,
    pub ahb2: AHB2,
    pub ahb3: AHB3,
    // pub apb1: APB1,
    // pub apb2: APB2,
    // pub bdcr: BDCR,
    pub cfgr: CFGR,
}

pub struct AHB1 {
    _0: (),
}

impl AHB1 {
    pub(crate) fn enr(&mut self) -> &rcc::AHB1ENR {
        unsafe { &(*RCC::ptr()).ahb1enr }
    }

    pub(crate) fn rstr(&mut self) -> &rcc::AHB1RSTR {
        unsafe { &(*RCC::ptr()).ahb1rstr }
    }
}

pub struct AHB2 {
    _0: (),
}

impl AHB2 {
    pub(crate) fn enr(&mut self) -> &rcc::AHB2ENR {
        unsafe { &(*RCC::ptr()).ahb2enr }
    }

    pub(crate) fn rstr(&mut self) -> &rcc::AHB2RSTR {
        unsafe { &(*RCC::ptr()).ahb2rstr }
    }
}

pub struct AHB3 {
    _0: (),
}

impl AHB3 {
    pub(crate) fn enr(&mut self) -> &rcc::AHB3ENR {
        unsafe { &(*RCC::ptr()).ahb3enr }
    }

    pub(crate) fn rstr(&mut self) -> &rcc::AHB3RSTR {
        unsafe { &(*RCC::ptr()).ahb3rstr }
    }
}

#[derive(Default)]
pub struct CFGR {
    hse: Option<u32>,
    hse_bypass: bool,
    css: bool,
    hclk: Option<u32>,
    pclk1: Option<u32>,
    pclk2: Option<u32>,
    sysclk: Option<u32>,
}

impl CFGR {
    /// Uses `HSE` (high speed external oscillator) instead of `HSI` (high speed internal oscillator) as the clock source
    ///
    /// Will result in a hang if an external oscillator is not connected or it fails to start,
    /// unless enable_css is enabled
    pub fn use_hse<F>(mut self, freq: F) -> Self
    where
        F: Into<Hertz>,
    {
        self.hse = Some(freq.into().0);
        self
    }

    pub fn bypass_hse(mut self) -> Self {
        self.hse_bypass = true;
        self
    }

    pub fn enable_css(mut self) -> Self {
        self.css = true;
        self
    }

    /// Sets a frequency for the HCLK AHB bus
    ///
    /// Maximum of 170 MHz
    pub fn hclk<F>(mut self, freq: F) -> Self
    where
        F: Into<Hertz>,
    {
        self.hclk = Some(freq.into().0);
        self
    }

    /// Sets a frequency for the APB1 bus
    ///
    /// Maximum of 170 MHz
    pub fn pclk1<F>(mut self, freq: F) -> Self
    where
        F: Into<Hertz>,
    {
        self.pclk1 = Some(freq.into().0);
        self
    }

    /// Sets a frequency for the APB2 bus
    ///
    /// Maximum of 170 MHz
    pub fn pclk2<F>(mut self, freq: F) -> Self
    where
        F: Into<Hertz>,
    {
        self.pclk2 = Some(freq.into().0);
        self
    }

    /// Sets a frequency for the system core
    ///
    /// Maximum of 170 MHz
    pub fn sysclk<F>(mut self, freq: F) -> Self
    where
        F: Into<Hertz>,
    {
        self.sysclk = Some(freq.into().0);
        self
    }


    pub fn freeze(self) -> Clocks {
        let rcc = unsafe { &*RCC::ptr() };

        // Turn on the external oscillator. The HSE is 8 MHz
        rcc.cr.modify(|_, w| w.hseon().set_bit());

        // Wait for the HSE to turn on and stabilize
        while rcc.cr.read().hseon() != true {}

        // Setup the PLL
        rcc.cr.modify(|_, w| w.pllon().clear_bit());

        while rcc.cr.read().pllrdy() == true {}

        // PLLPCLK -> ADC
        // 8 -> 24 MHz = 3x multiplier
        // 8 / M * N / P = 24 MHz
        // M = 2
        // N = 72, PLL = 288
        // P = 12

        // PLLQCLK -> 48 MHz for USB and CAN
        // 8 -> 48 = 6x multiplier
        // 8 / M * N / Q = 48
        // Q = 6

        // PLLRCLK => SYSCLK at high freq
        // 8 -> 144 MHz = 20x multiplier
        // 8 / M * N / R = 144
        // R = 2

        unsafe {
            rcc.pllcfgr.modify(|_, w| {
                w.pllsrc().bits(0b11); // Set PLL src to HSE
                w.pllm().bits(1);
                w.plln().bits(72);
                w.pllpdiv().bits(12);
                w.pllq().bits(0b10); // Corresponds to a divider of 6
                w.pllr().bits(0b00) // Corresponds to a divider of 2
            });
        }

        // Turn on the PLL
        rcc.cr.modify(|_, w| w.pllon().set_bit());

        while rcc.cr.read().pllrdy() != true {}

        rcc.pllcfgr.modify(|_, w| {
            w.pllpen().set_bit();
            w.pllqen().set_bit();
            w.pllren().set_bit()
        });

        // It is recommended to go through an intermediate frequency when switching clocks.
        // To accomplish this scale the AHB clock (HPRE[3:0]) by two, change to the PLL as
        // source, wait 1 us, then set the AHB prescaler back.
        unsafe {
            rcc.cfgr.modify(|_, w| {
                w.ppre2().bits(0b101); // Divide the APB2 domains by 4
                w.ppre1().bits(0b100); // Divide the APB1 domains by 2
                w.hpre().bits(0b1000);
                w.sw().bits(0b11) // PLL as system clock
            });
        }

        for _ in 0..10000 {}

        unsafe { rcc.cfgr.modify(|_, w| w.hpre().bits(0b0000)) }

        unsafe {
            // Prescale MCO by 16
            rcc.cfgr.modify(|_, w| w.mcopre().bits(0b100));

            // Set MCO source to PLL
            rcc.cfgr.modify(|_, w| w.mcosel().bits(0b0001));
        }

        // Turn on apb clock to CAN FD module
        rcc.apb1enr1.modify(|_, w| w.fdcanen().set_bit());

        unsafe {
            // Set FD can clock to use PCLK1 as clock source. PCLK1 is 72 MHz
            rcc.ccipr.modify(|_, w| w.fdcansel().bits(0b10));
        }

        // Turn on clocks to GPIO A
        rcc.ahb2enr.modify(|_, w| w.gpioaen().set_bit());
        rcc.ahb2enr.modify(|_, w| w.gpioben().set_bit());
        rcc.ahb2enr.modify(|_, w| w.gpiocen().set_bit());

        // Configure clocks for USB
        unsafe {
            // Set 48 MHz clock sourced from PLL Q clock
            rcc.ccipr.modify(|_, w| w.clk48sel().bits(0b10));

            // Turn on USB clock
            rcc.apb1enr1.modify(|_, w| w.usben().set_bit());
        }

        Clocks {
            sysclk: 144.mhz(),
            hclk: 144.mhz(),
            pclk1: 72.mhz(),
            pclk2: 72.mhz(),
            pll_clk: PLLClocks {
                r: 144.mhz(),
                q: Some(48.mhz()),
                p: Some(24.mhz()),
            }
        }
    }
}

/// HSI speed
pub const HSI_FREQ: u32 = 16_000_000;

/// Clock frequencies
#[derive(Clone, Copy)]
pub struct Clocks {
    /// System frequency - SYSCLK
    pub sysclk: Hertz,
    /// Core frequency - HCLK
    pub hclk: Hertz,
    /// APB1 frequency - PCLK1
    pub pclk1: Hertz,
    /// APB1 frequency - PCLK2
    pub pclk2: Hertz,
    /// PLL frequency
    pub pll_clk: PLLClocks,
}

/// PLL Clock frequencies
#[derive(Clone, Copy)]
pub struct PLLClocks {
    /// R frequency
    pub r: Hertz,
    /// Q frequency
    pub q: Option<Hertz>,
    /// P frequency
    pub p: Option<Hertz>,
}

// impl Default for Clocks {
//     fn default() -> Clocks {
//         let freq = HSI_FREQ.hz();
//         Clocks {
//             sys_clk: freq,
//             ahb_clk: freq,
//             core_clk: freq,
//             apb_clk: freq,
//             apb_tim_clk: freq,
//             pll_clk: PLLClocks {
//                 r: 32_u32.mhz(),
//                 q: None,
//                 p: None,
//             },
//         }
//     }
// }
