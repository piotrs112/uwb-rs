#![no_main]
#![no_std]

pub mod handshake;

use core::fmt::Error;

use defmt_rtt as _;

use dwm1001::{
    dw1000::{
        hl::Receiving,
        mac,
        range_bias::{get_range_bias_cm, improve_rssi_estimation},
        RxConfig, TxConfig,
    },
    nrf52832_hal::{
        pac::{ficr::deviceid::DEVICEID_SPEC, generic::Reg},
        Delay,
    },
    prelude::*,
    Led, DWM1001,
};

use panic_probe as _;

// same panicking *behavior* as `panic-probe` but doesn't print a panic message
// this prevents the panic message being printed *twice* when `defmt::panic` is invoked
#[defmt::panic_handler]
fn panic() -> ! {
    cortex_m::asm::udf()
}

/// Terminates the application and makes `probe-run` exit with exit-code = 0
pub fn exit() -> ! {
    loop {
        cortex_m::asm::bkpt();
    }
}

pub struct UWBConfig {
    pub rx_config: RxConfig,
    pub tx_config: TxConfig,
    pub pan_id: mac::PanId,
    pub short_address: mac::ShortAddress,
}

impl Default for UWBConfig {
    fn default() -> Self {
        UWBConfig {
            rx_config: RxConfig::default(),
            tx_config: TxConfig::default(),
            pan_id: mac::PanId(0x0d57),
            short_address: mac::ShortAddress(0x0123),
        }
    }
}

pub fn flash_led(led: &mut Led, delay: &mut Delay) {
    led.enable();
    delay.delay_ms(10u32);
    led.disable();
}

pub fn distance_correction(
    distance_mm: u64,
    rssi: f32,
    rx_config: &RxConfig,
) -> Result<f32, Error> {
    let rsl = improve_rssi_estimation(rssi, rx_config);
    let bias_mm = get_range_bias_cm(rsl, rx_config) * 10.0;
    Ok(distance_mm as f32 + bias_mm)
}

pub fn serial_number(dwm1001: &DWM1001) -> (u32, u32) {
    let dev_id = &dwm1001.FICR.deviceid;
    (
        read_device_id_register(&dev_id[0]),
        read_device_id_register(&dev_id[1]),
    )
}

fn read_device_id_register(reg: &Reg<DEVICEID_SPEC>) -> u32 {
    reg.read().bits()
}
