//! Range measurement tag node
//!
//! This is a tag node used for range measurement. Tags use anchor nodes to
//! measure their distance from those anchors.
//!
//! Currently, distance measurements have a highly inaccurate result. One reason
//! that could account for this is the lack of antenna delay calibration, but
//! it's possible that there are various hidden bugs that contribute to this.

#![no_main]
#![no_std]

use defmt_rtt as _;
use panic_probe as _;

use heapless::String as HString;

use dwm1001::{
    block_timeout,
    dw1000::{
        mac,
        ranging::{self, Message as _RangingMessage},
        RxConfig,
    },
    nrf52832_hal::{
        gpio::{p0::P0_17, Output, PushPull},
        pac::{SPIM2, UARTE0},
        rng::Rng,
        Delay, Spim, Timer, Uarte,
    },
    prelude::*,
};

#[cortex_m_rt::entry]
fn main() -> ! {
    defmt::info!("Launching tag");

    let mut dwm1001 = dwm1001::DWM1001::take().unwrap();

    let mut delay = Delay::new(dwm1001.SYST);
    let mut rng = Rng::new(dwm1001.RNG);
    let mut uart_buf = [0u8; 32];

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

    let mut dw_irq = dwm1001.DW_IRQ;
    let mut gpiote = dwm1001.GPIOTE;

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
            mac::PanId(0x0d57),                  // hardcoded network id
            mac::ShortAddress(rng.random_u16()), // random device address
        )
        .expect("Failed to set address");

    let mut timeout_timer = Timer::new(dwm1001.TIMER1);

    let mut buf = [0; 128];

    loop {
        defmt::info!("waiting for base station ping");
        let mut receiving = dw1000
            .receive(RxConfig::default())
            .expect("Failed to receive message");
        dwm1001.uart.write(base_10_bytes(1, &mut uart_buf)).unwrap();
        timeout_timer.start(5_500_000u32);
        let message = block_timeout!(&mut timeout_timer, {
            dw_irq.wait_for_interrupts(&mut gpiote, &mut timeout_timer);
            receiving.wait_receive(&mut buf)
        });
        dwm1001.uart.write(base_10_bytes(2, &mut uart_buf)).unwrap();

        dw1000 = receiving
            .finish_receiving()
            .expect("Failed to finish receiving");
        dwm1001.uart.write(base_10_bytes(3, &mut uart_buf)).unwrap();

        let message = match message {
            Ok(message) => message,
            Err(_) => {
                write_uart(&mut dwm1001.uart, "Timeout error occured");
                continue;
            }
        };

        defmt::info!("msg from base station: received");
        dwm1001.uart.write(base_10_bytes(4, &mut uart_buf)).unwrap();

        let ping = ranging::Ping::decode::<Spim<SPIM2>, P0_17<Output<PushPull>>>(&message)
            .expect("Failed to decode ping");
        if let Some(ping) = ping {
            // Received ping from an anchor. Reply with a ranging
            // request.
            dwm1001.uart.write(base_10_bytes(5, &mut uart_buf)).unwrap();

            dwm1001.leds.D10.enable();
            delay.delay_ms(10u32);
            dwm1001.leds.D10.disable();

            // Wait for a moment, to give the anchor a chance to start listening
            // for the reply.
            delay.delay_ms(10u32);

            let mut sending = ranging::Request::new(&mut dw1000, &ping)
                .expect("Failed to initiate request")
                .send(dw1000)
                .expect("Failed to initiate request transmission");

            timeout_timer.start(500_000u32);
            block_timeout!(&mut timeout_timer, {
                dw_irq.wait_for_interrupts(&mut gpiote, &mut timeout_timer);
                sending.wait_transmit()
            })
            .expect("Failed to send ranging request");

            dw1000 = sending.finish_sending().expect("Failed to finish sending");

            continue;
        }

        let response = ranging::Response::decode::<Spim<SPIM2>, P0_17<Output<PushPull>>>(&message)
            .expect("Failed to decode response");
        if let Some(response) = response {
            // Received ranging response from anchor. Now we can compute the
            // distance.

            dwm1001.leds.D11.enable();
            delay.delay_ms(10u32);
            dwm1001.leds.D11.disable();

            // If this is not a PAN ID and short address, it doesn't
            // come from a compatible node. Ignore it.
            let (pan_id, addr) = match response.source {
                Some(mac::Address::Short(pan_id, addr)) => (pan_id, addr),
                _ => continue,
            };

            // Ranging response received. Compute distance.
            let distance_mm = ranging::compute_distance_mm(&response).unwrap();

            dwm1001.leds.D9.enable();
            delay.delay_ms(10u32);
            dwm1001.leds.D9.disable();

            // write_uart(&mut dwm1001, distance_mm.to_be_bytes());
            write_uart(&mut dwm1001.uart, "measurement done!");

            // defmt::info!("{:04x}:{:04x} - {} mm\n", pan_id.0, addr.0, distance_mm,);

            continue;
        }

        dwm1001.leds.D11.enable();
        delay.delay_ms(10u32);
        dwm1001.leds.D11.disable();
        // defmt::info!("Ignored message that was neither ping nor response\n");
    }
}

fn write_uart(uart: &mut Uarte<UARTE0>, msg: &str) {
    let mut s: HString<64> = HString::new();
    s.push_str(msg).expect("Failed to push to string");
    s.push_str("\n").expect("Failed to push to string");
    uart.write(s.as_bytes()).unwrap();
}

fn base_10_bytes(mut n: u64, buf: &mut [u8]) -> &[u8] {
    if n == 0 {
        return b"0";
    }
    let mut i = 0;
    while n > 0 {
        buf[i] = (n % 10) as u8 + b'0';
        n /= 10;
        i += 1;
    }
    let slice = &mut buf[..i];
    slice.reverse();
    &*slice
}
