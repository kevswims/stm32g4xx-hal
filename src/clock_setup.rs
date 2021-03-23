use crate::flash::ACR;
use crate::rcc::Rcc;

pub fn set_wait_states(wait_states: u8, acr: &mut ACR, rcc: &mut Rcc) -> () {
    unsafe {
        // flash.acr.modify(|_, w| w.latency().bits(wait_states)); // Four wait states

        rcc.ahb2.enr().modify(|_, w| w.gpioaen().set_bit());
        acr.acr().modify(|_, w| w.latency().bits(wait_states));
    }
}
