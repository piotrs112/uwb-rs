use dwm1001::{
    dw1000::{
        mac,
        ranging::{Message, Ping, Prelude, TxMessage},
        time::Instant,
        Error, Ready, Sending, TxConfig, DW1000,
    },
    embedded_hal::blocking::spi,
    prelude::*,
};
use ieee802154::mac::{Address, PanId, ShortAddress};

// use serde::{Deserialize, Serialize};

// #[derive(Debug, Serialize, Deserialize)]
// #[repr(C)]
// pub struct Advertise {
//     //id: u16, // Short address
// }
// impl Message for Advertise {
//     const PRELUDE: Prelude = Prelude(b"ADVERTISE");
//     const PRELUDE_LEN: usize = 9;
// }

// impl Advertise {
//     pub fn new<SPI, CS>(
//         dw1000: &mut DW1000<SPI, CS, Ready>,
//     ) -> Result<TxMessage<Self>, dwm1001::dw1000::Error<SPI, CS>>
//     where
//         SPI: spi::Transfer<u8> + spi::Write<u8>,
//         CS: OutputPin,
//     {
//         dw1000.send(&[], destination, None, config)
//         // Ok(TxMessage {
//         //     recipient: mac::Address::broadcast(&mac::AddressMode::Short),
//         //     tx_time: Instant,
//         //     payload: todo!(),
//         // })
//     }
// }

pub mod messages {
    pub const ADVERTISE: &[u8] = b"ADVERTISE";
    pub const READY: &[u8] = b"READY";
}

pub fn advertise<SPI, CS>(
    dw1000: DW1000<SPI, CS, Ready>,
    tx_config: &TxConfig,
) -> Result<DW1000<SPI, CS, Sending>, dwm1001::dw1000::Error<SPI, CS>>
where
    SPI: spi::Transfer<u8> + spi::Write<u8>,
    CS: OutputPin,
{
    dw1000.send(
        messages::ADVERTISE,
        mac::Address::broadcast(&mac::AddressMode::Short),
        dwm1001::dw1000::hl::SendTime::Now,
        *tx_config,
    )
}

pub fn send_ready<SPI, CS>(
    dw1000: DW1000<SPI, CS, Ready>,
    tx_config: &TxConfig,
    pan_id: &PanId,
    destination: &ShortAddress,
) -> Result<DW1000<SPI, CS, Sending>, dwm1001::dw1000::Error<SPI, CS>>
where
    SPI: spi::Transfer<u8> + spi::Write<u8>,
    CS: OutputPin,
{
    dw1000.send(
        messages::READY,
        Some(mac::Address::Short(*pan_id, *destination)),
        dwm1001::dw1000::hl::SendTime::Now,
        *tx_config,
    )
}
