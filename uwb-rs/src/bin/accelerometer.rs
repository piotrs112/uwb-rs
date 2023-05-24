#![no_main]
#![no_std]

use core::fmt::Write;

use dwm1001;
use lis2dh12::{self, Accelerometer, RawAccelerometer};
use uwb_rs as _; // memory layout + panic handler

#[cortex_m_rt::entry]
fn main() -> ! {
    defmt::info!("Anchor");
    let mut dwm1001 = dwm1001::DWM1001::take().unwrap();
    let address = lis2dh12::SlaveAddr::Alternative(true);
    let mut acc = lis2dh12::Lis2dh12::new(dwm1001.LIS2DH12, address).unwrap();
    acc.enable_axis((true, true, true))
        .expect("Failed to enable axis");
    acc.set_mode(lis2dh12::Mode::HighResolution)
        .expect("Failed to set mode");
    acc.set_odr(lis2dh12::Odr::Hz50)
        .expect("Failed to set data rate");

    // acc.enable_temp(true)
    //     .expect("Failed to enable temperature sensor");

    // Read data
    loop {
        // let accel = acc.accel_raw().unwrap();
        let accel = acc.accel_norm().unwrap();
        // let temp = acc.get_temp_outf().unwrap();

        defmt::info!(
            "Acc: x={:04} y={:04} z={:04}",
            // temp,
            accel.x,
            accel.y,
            accel.z,
        );
        dwm1001
            .uart
            .write_str(&"Hello world!")
            .expect("Failed to write");
    }
}
