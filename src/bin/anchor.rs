#![no_main]
#![no_std]

use uwb_rs as _; // memory layout + panic handler

#[cortex_m_rt::entry]
fn main() -> ! {
    defmt::info!("Anchor");
    loop {
        todo!();
    }
}
