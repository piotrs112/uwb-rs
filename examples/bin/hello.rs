#![no_main]
#![no_std]

use uwb_rs_defmt as _; // global logger + panicking-behavior + memory layout

#[cortex_m_rt::entry]
fn main() -> ! {
    defmt::println!("Hello, world!");

    uwb_rs_defmt::exit()
}
