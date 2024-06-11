use crate::packet::FirmwareUpdatePacket;
use std::num::Wrapping;

/**
 * STM32G4xx default CRC32 polynomial.
 */
pub const CUSTOM_ALG: crc::Algorithm<u32> = crc::Algorithm {
    width: 32,
    poly: 0x04C1_1DB7,
    init: 0xFFFF_FFFF,
    refin: false,
    refout: false,
    xorout: 0x0000_0000,
    check: 0x0000_0000,   // todo ...
    residue: 0x0000_0000, // todo ...
};

pub fn calc_stm32_crc(packets: &[FirmwareUpdatePacket]) -> u32 {
    let crc = crc::Crc::<u32>::new(&CUSTOM_ALG);
    let mut digest = crc.digest();
    for p in packets.iter() {
        digest.update(&p.to_vec());
    }
    digest.finalize()
}

pub fn calc_ccitt_crc(data: &[u8], size: u32) -> u16 {
    let mut crc: u16 = 0xffff;

    let u8_memory_size = std::mem::size_of::<u8>();
    let total_bytes: usize = (size as usize) / u8_memory_size;
    let mut current_byte: usize = 0;

    while current_byte < total_bytes {
        let value = data[current_byte];

        crc = (crc >> 8) | (crc << 8);
        crc ^= value as u16; // Cast to u16 for bitwise xor

        crc ^= (crc & 0xff) >> 4;

        // Shift 12 left and xor
        crc ^= (crc << 8) << 4;

        // Shift lsb 5 and xor
        crc ^= ((crc & 0xff) << 4) << 1;

        current_byte += 1;
    }

    crc
}

#[allow(unused)]
pub fn ihex_checksum(data: &[u8]) -> u8 {
    let mut cs: Wrapping<u8> = Wrapping(0);
    for x in data.iter() {
        cs += Wrapping(*x);
    }
    cs = (!cs) + Wrapping(1);
    cs.0
}

//----------------------------------------------------------------------------
// Tests
//----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::ihex_checksum;

    const TEST_LINE: &str = ":1007F80004F03CFA69461A48FFF70AFE694604F114";
    const TEST_DATA: [u8; 21] = [
        0x10, 0x07, 0xf8, 0x00, 0x04, 0xf0, 0x3c, 0xfa, 0x69, 0x46, 0x1a, 0x48, 0xff, 0xf7, 0x0a,
        0xfe, 0x69, 0x46, 0x04, 0xf1, 0x14,
    ];

    #[allow(unused)]
    fn make_test_data() -> Vec<u8> {
        return TEST_LINE.as_bytes().to_vec();
    }

    #[test]
    fn checksum_implementation_matches_spec() {
        let tdata: Vec<u8> = TEST_DATA.to_vec();
        let end = tdata.len() - 1;
        let start = &tdata[0..end];
        let last: u8 = tdata[end];
        assert!(ihex_checksum(start) == last);
    }
}
