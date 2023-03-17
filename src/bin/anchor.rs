#![no_main]
#![no_std]

use defmt_rtt as _;
use panic_probe as _;

#[cortex_m_rt::entry]
fn main() -> ! {
    defmt::info!("Anchor");
    loop {
        todo!();
    }
}
