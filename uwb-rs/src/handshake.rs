use dwm1001::{
    dw1000::{mac, Ready, Sending, TxConfig, DW1000},
    embedded_hal::blocking::spi,
    prelude::*,
};
use ieee802154::mac::{PanId, ShortAddress};

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
