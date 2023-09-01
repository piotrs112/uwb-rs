#![no_main]
#![no_std]

use dwm1001::{
    block_timeout,
    dw1000::{
        mac,
        ranging::{self, Message as _RangingMessage},
    },
    nrf52832_hal::{
        gpio::{p0::P0_17, Output, PushPull},
        pac::SPIM2,
        Delay, Spim, Timer,
    },
    prelude::*,
};
use uwb_rs::{
    self as _, flash_led,
    handshake::{self, advertise},
    serial_number, UWBConfig,
};

#[cortex_m_rt::entry]
fn main() -> ! {
    defmt::info!("Launching anchor");

    let mut dwm1001 = dwm1001::DWM1001::take().unwrap();
    let sn = serial_number(&dwm1001);

    let mut delay = Delay::new(dwm1001.SYST);

    let uwb_config = UWBConfig {
        short_address: mac::ShortAddress(sn.1 as u16),
        ..UWBConfig::default()
    };
    dwm1001.DW_RST.reset_dw1000(&mut delay);

    let mut dw1000 = dwm1001
        .DW1000
        .init(&mut delay)
        .expect("Failed to initialize DW1000");

    dw1000
        .enable_tx_interrupts()
        .expect("Failed to enable TX interrupts");

    dw1000
        .enable_rx_interrupts()
        .expect("Failed to enable RX interrupts");

    dw1000
        .set_antenna_delay(16456, 16300)
        .expect("Failed to set antenna delay");

    dw1000
        .set_address(uwb_config.pan_id, uwb_config.short_address)
        .expect("Failed to set address");

    let mut timer = Timer::new(dwm1001.TIMER0);

    let mut buffer = [0; 1024];

    // Advertise the anchor

    let mut sending =
        advertise(dw1000, &uwb_config.tx_config).expect("Failed to advertise address");

    nb::block!(sending.wait_transmit()).expect("Failed to send data");
    dw1000 = sending.finish_sending().expect("Failed to finish sending");

    loop {
        /*
        Strategy for mobile tag ranging:
        - 0. Wait for Ready
        - 1. Send a ping
        - 2. Wait for a ranging request
        - 3. Send a ranging response
        */

        /*
        0. Waiting for Ready packet
        */

        let mut receiving = dw1000
            .receive(uwb_config.rx_config)
            .expect("Failed to receive");

        timer.start(5_000_000u32);
        let result = block_timeout!(&mut timer, receiving.wait_receive(&mut buffer));

        dw1000 = receiving
            .finish_receiving()
            .expect("Failed to finish receiving");

        let message = match result {
            Ok(msg) => msg,
            Err(_) => continue,
        };

        if message.frame.payload != handshake::messages::READY {
            continue;
        }

        defmt::info!("Sending ping");

        flash_led(&mut dwm1001.leds.D10, &mut delay);

        /*
        1. Send a ping
        */

        let mut sending = ranging::Ping::new(&mut dw1000)
            .expect("Failed to initiate ping")
            .send(dw1000)
            .expect("Failed to initiate ping transmission");

        nb::block!(sending.wait_transmit()).expect("Failed to send data");
        dw1000 = sending.finish_sending().expect("Failed to finish sending");

        defmt::info!("Ping sent, waiting for base station response");

        /*
        2. Wait for the anchor to respond with a ranging request.
        */
        let mut receiving = dw1000
            .receive(uwb_config.rx_config)
            .expect("Failed to receive message");

        timer.start(5_000_000u32);
        let result = block_timeout!(&mut timer, receiving.wait_receive(&mut buffer));

        dw1000 = receiving
            .finish_receiving()
            .expect("Failed to finish receiving");

        let message = match result {
            Ok(message) => message,
            Err(error) => {
                use embedded_timeout_macros::TimeoutError;
                match error {
                    TimeoutError::Timeout => {
                        defmt::info!("Waiting for base station timed out. Trying again.")
                    }
                    TimeoutError::Other(o) => {
                        defmt::error!("Other error: {:?}", defmt::Debug2Format(&o));
                    }
                }
                continue;
            }
        };

        /*
        3. Decode the ranging request and respond with a ranging response
        */
        let request =
            match ranging::Request::decode::<Spim<SPIM2>, P0_17<Output<PushPull>>>(&message) {
                Ok(Some(request)) => request,
                Ok(None) | Err(_) => {
                    defmt::info!("Ignoring message that is not a request\n");
                    continue;
                }
            };

        defmt::info!("Ranging request received. Preparing to send ranging response.");

        flash_led(&mut dwm1001.leds.D12, &mut delay);

        // Wait for a moment, to give the tag a chance to start listening for
        // the reply.
        delay.delay_ms(10u32);

        // Send ranging response
        let mut sending = ranging::Response::new(&mut dw1000, &request)
            .expect("Failed to initiate response")
            .send(dw1000)
            .expect("Failed to initiate response transmission");

        nb::block!(sending.wait_transmit()).expect("Failed to send data");
        dw1000 = sending.finish_sending().expect("Failed to finish sending");

        defmt::info!("Ranging response sent");

        flash_led(&mut dwm1001.leds.D9, &mut delay);
    }
}
