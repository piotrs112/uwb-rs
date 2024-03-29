#![no_main]
#![no_std]

use defmt_rtt as _;
use panic_probe as _;

use core::fmt::Write;
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
use lis2dh12::{self, Accelerometer};
use uwb_rs::{
    self as _, distance_correction, flash_led,
    handshake::{self, send_ready},
    serial_number, UWBConfig,
}; // memory layout + panic handler

#[cortex_m_rt::entry]
fn main() -> ! {
    defmt::debug!("Launching tag.");

    const N_ANCHORS: usize = 4;

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

    // These are the hardcoded calibration values from the dwm1001-examples
    // repository[1]. Ideally, the calibration values would be determined using
    // the proper calibration procedure, but hopefully those are good enough for
    // now.
    //
    // [1] https://github.com/Decawave/dwm1001-examples
    dw1000
        .set_antenna_delay(16456, 16300)
        .expect("Failed to set antenna delay");

    // Set network address
    dw1000
        .set_address(
            uwb_config.pan_id,        // hardcoded network id
            uwb_config.short_address, // random device address
        )
        .expect("Failed to set address");

    let mut timer = Timer::new(dwm1001.TIMER0);

    let mut buffer1 = [0; 1024];
    let mut buffer2 = [0; 1024];

    let mut known_anchors = [None; N_ANCHORS];

    let mut i = 0;
    while i < N_ANCHORS {
        let mut receiving = dw1000
            .receive(uwb_config.rx_config)
            .expect("Failed to receive message");

        timer.start(15_000_000u32);
        let message = block_timeout!(&mut timer, receiving.wait_receive(&mut buffer1));

        dw1000 = receiving
            .finish_receiving()
            .expect("Failed to finish receiving");

        let message = match message {
            Ok(message) => message,
            Err(_) => {
                defmt::error!("Timeout error occured");
                continue;
            }
        };

        if message.frame.payload != handshake::messages::ADVERTISE {
            continue;
        }

        let source = match message.frame.header.source.unwrap() {
            mac::Address::Short(_, add) => add,
            mac::Address::Extended(_, _) => continue,
        };

        defmt::debug!("Discovered anchor {:?}", source);
        known_anchors[i] = Some(source);

        i += 1;
    }

    // Init accelerometer
    let address = lis2dh12::SlaveAddr::Alternative(true);
    let mut acc = lis2dh12::Lis2dh12::new(dwm1001.LIS2DH12, address).unwrap();
    acc.enable_axis((true, true, true))
        .expect("Failed to enable axis");
    acc.set_mode(lis2dh12::Mode::HighResolution)
        .expect("Failed to set mode");
    acc.set_odr(lis2dh12::Odr::Hz50)
        .expect("Failed to set data rate");

    'main_loop: loop {
        /*
        Strategy for basestation:
        - 0. Send Ready packet
        - 1. Wait for ping
        - 2. Initiate ranging request
        - 3. Wait for response
        - 4. Calculate and log distance to tag
        */

        for anchor in known_anchors.iter().flatten() {
            /*
            0. Send Ready packet
            */
            defmt::debug!("Sending READY to {:?}", &anchor);
            let mut sending =
                send_ready(dw1000, &uwb_config.tx_config, &uwb_config.pan_id, anchor).unwrap();

            timer.start(5_000_000u32);
            block_timeout!(&mut timer, sending.wait_transmit()).expect("Failed to transmit");

            dw1000 = sending.finish_sending().expect("Failed to finish sending");

            /*
            1. Wait for ping
            */
            timer.start(5_000_000u32);
            defmt::debug!("Waiting for PING from anchor {:?}", &anchor);
            let mut receiving = dw1000
                .receive(uwb_config.rx_config)
                .expect("Failed to receive message");

            let message = block_timeout!(&mut timer, receiving.wait_receive(&mut buffer1));

            dw1000 = receiving
                .finish_receiving()
                .expect("Failed to finish receiving");

            let message = match message {
                Ok(message) => message,
                Err(_) => {
                    defmt::error!("Timeout error occured");
                    continue 'main_loop; // Results from other anchors after a big timeout are useless
                }
            };

            let ping = match ranging::Ping::decode::<Spim<SPIM2>, P0_17<Output<PushPull>>>(&message)
            {
                Ok(Some(ping)) => ping,
                Ok(None) => {
                    defmt::error!("Failed to decode ping");
                    continue 'main_loop;
                }
                Err(e) => {
                    defmt::error!("Ping decode error: {:?}", defmt::Debug2Format(&e));
                    continue 'main_loop;
                }
            };

            defmt::debug!(
                "Received ping from {:?}.\nResponding with ranging request.",
                ping.source
            );

            flash_led(&mut dwm1001.leds.D10, &mut delay);

            // Wait for a moment, to give the anchor a chance to start listening
            // for the reply.
            delay.delay_ms(10u32);

            /*
            2. Initiate ranging request
            */
            let mut sending = ranging::Request::new(&mut dw1000, &ping)
                .expect("Failed to initiate request")
                .send(dw1000)
                .expect("Failed to initiate request transmission");

            nb::block!(sending.wait_transmit()).expect("Failed to send data");
            dw1000 = sending.finish_sending().expect("Failed to finish sending");

            defmt::debug!("Request sent Transmission sent. Waiting for response.");

            /*
            3. Wait for response
            */
            let mut receiving = dw1000
                .receive(uwb_config.rx_config)
                .expect("Failed to receive message");

            timer.start(500_000u32);
            let result = block_timeout!(&mut timer, receiving.wait_receive(&mut buffer2));

            // Wait for a moment so RSSI and LOS confidence data become available
            delay.delay_ms(5u32);

            let (rssi, los_confidence) = match result {
                Ok(_) => {
                    let metrics = receiving.read_rx_quality().unwrap();
                    (metrics.rssi, metrics.los_confidence_level)
                }
                Err(_) => (0.0, -1.0),
            };

            dw1000 = receiving
                .finish_receiving()
                .expect("Failed to finish receiving");

            let message = match result {
                Ok(message) => message,
                Err(error) => match error {
                    embedded_timeout_macros::TimeoutError::Timeout => {
                        defmt::debug!("Waiting for mobile tag respond timed out.");
                        continue 'main_loop;
                    }
                    embedded_timeout_macros::TimeoutError::Other(other) => {
                        defmt::error!("Other timeout error: {:?}", defmt::Debug2Format(&other));
                        continue 'main_loop;
                    }
                },
            };

            let response =
                match ranging::Response::decode::<Spim<SPIM2>, P0_17<Output<PushPull>>>(&message) {
                    Ok(Some(response)) => response,
                    Ok(None) => {
                        defmt::error!(
                            "Failed to decode ranging response. Frame is {:?}",
                            defmt::Debug2Format(&message.frame)
                        );
                        continue;
                    }
                    Err(e) => {
                        defmt::error!(
                            "Ranging response decode error: {:?}",
                            defmt::Debug2Format(&e)
                        );
                        continue;
                    }
                };

            /*
            4. Calculate distance
            */

            // If this is not a PAN ID and short address, it doesn't
            // come from a compatible node. Ignore it.
            let (pan_id, addr) = match response.source {
                Some(mac::Address::Short(pan_id, addr)) => (pan_id, addr),
                _ => continue,
            };

            // Ranging response received. Compute distance.
            match ranging::compute_distance_mm(&response) {
                Ok(distance_mm) => {
                    flash_led(&mut dwm1001.leds.D9, &mut delay);

                    let computed_distance =
                        distance_correction(distance_mm, rssi, &uwb_config.rx_config).unwrap();
                    let timestamp = response.rx_time.value();
                    let accel = acc.accel_norm().unwrap();
                    write!(
                        dwm1001.uart,
                        "{} {} {} {} {} {} {}\r\n",
                        addr.0,
                        computed_distance,
                        timestamp,
                        accel.x,
                        accel.y,
                        accel.z,
                        los_confidence
                    )
                    .expect("Failed to write result to UART");
                    defmt::info!(
                        "[{}] {}:{} - {}mm, LOS: {}| {} {} {}\n",
                        timestamp,
                        pan_id.0,
                        addr.0,
                        computed_distance,
                        los_confidence,
                        accel.x,
                        accel.y,
                        accel.z
                    );
                }
                Err(e) => {
                    defmt::error!("Ranging response error: {:?}", defmt::Debug2Format(&e));
                }
            }

            flash_led(&mut dwm1001.leds.D11, &mut delay);
        }
    }
}
