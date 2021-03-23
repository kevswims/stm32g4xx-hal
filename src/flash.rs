//! Flash memory

use crate::stm32::{flash, FLASH};

pub trait FlashExt {
    fn constrain(self) -> Parts;
}

impl FlashExt for FLASH {
    fn constrain(self) -> Parts {
        Parts {
            acr: ACR { _0: () },
        }
    }
}

pub struct Parts {
    pub acr: ACR,
}

pub struct ACR {
    _0: (),
}

impl ACR {
    pub(crate) fn acr(&mut self) -> &flash::ACR {
        unsafe { &(*FLASH::ptr()).acr }
    }
}