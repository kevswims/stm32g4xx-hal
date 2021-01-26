use crate::flash::ACR;

pub fn set_wait_states(wait_states: u8, acr: &mut ACR) -> () {
    unsafe {
        // flash.acr.modify(|_, w| w.latency().bits(wait_states)); // Four wait states

        acr.acr().modify(|_, w| w.latency().bits(wait_states));
    }
}
