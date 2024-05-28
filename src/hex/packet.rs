use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;

use crate::hexcrc::calc_ccitt_crc;

// Must be divisible by 8 (bytes), for the 'HAL_FLASH_Program(..)' routine.
pub const MAX_DATA_LENGTH: usize = 200;

/**
 * Packet format for sending firmware updates, via USB.
 */
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[repr(C, packed)] // Keep the field order & packing (very important)
pub struct FirmwareUpdatePacket {
    pub boot_char: u8,   // should always be '*'
    pub update_char: u8, // should always be 'u'
    pub _dummy1: u16,    // *padding*
    pub address: u32,    // destination address
    pub data_length: u8, // num of byte of data in the data region of the packet
    pub _dummy2: u8,     // *padding*
    pub data_crc: u16,   // (CCITT) CRC 16 of data.
    #[serde(with = "BigArray")]
    pub data: [u8; MAX_DATA_LENGTH],
    pub end_of_packet: u8, // should always be '\n'
    pub _dummy3: u16,      // *padding*
    pub _dummy4: u8,       // *padding*
}

impl FirmwareUpdatePacket {
    pub fn new(addr: u32, data: [u8; MAX_DATA_LENGTH], size: usize) -> Self {
        let data_crc: u16 = calc_ccitt_crc(&data, size as u32);
        Self {
            boot_char: b'*',
            update_char: b'u',
            _dummy1: 0,
            address: addr,
            data_length: MAX_DATA_LENGTH as u8,
            _dummy2: 0,
            data_crc,
            data,
            end_of_packet: b'\n',
            _dummy3: 0,
            _dummy4: 0,
        }
    }

    pub fn address(&self) -> u32 {
        self.address
    }

    pub fn crc16(&self) -> u16 {
        self.data_crc
    }

    pub fn len(&self) -> usize {
        self.data_length as usize
    }
}
