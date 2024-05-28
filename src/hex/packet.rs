use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;

use crate::hexcrc::calc_ccitt_crc;

// Must be divisible by 8 (bytes), for the 'HAL_FLASH_Program(..)' routine.
pub const MAX_DATA_LENGTH: usize = 200;

/**
 * Packet format for sending firmware updates, via USB.
 */
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[repr(C, packed)] // Keep the field order & packing (very important)
pub struct FirmwareUpdatePacket {
    boot_char: u8,   // should always be '*'
    update_char: u8, // should always be 'u'
    _dummy1: u16,    // *padding*
    address: u32,    // destination address
    data_length: u8, // num of byte of data in the data region of the packet
    _dummy2: u8,     // *padding*
    data_crc: u16,   // (CCITT) CRC 16 of data.
    #[serde(with = "BigArray")]
    data: [u8; MAX_DATA_LENGTH],
    end_of_packet: u8, // should always be '\n'
    _dummy3: u16,      // *padding*
    _dummy4: u8,       // *padding*
}

impl FirmwareUpdatePacket {
    pub fn new(addr: u32, data: [u8; MAX_DATA_LENGTH], size: usize) -> Self {
        let data_crc: u16 = calc_ccitt_crc(&data, size as u32);
        Self {
            boot_char: b'*',
            update_char: b'u',
            _dummy1: 0,
            address: addr,
            data_length: size as u8,
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

    pub fn to_vec(&self) -> Vec<u8> {
        let len = self.data_length as usize;
        self.data[0..len].to_vec()
    }
}
