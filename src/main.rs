pub(crate) use hex::*;
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;

pub(crate) mod hex;

/**
 * STM32G4xx default CRC32 polynomial.
 */
const CUSTOM_ALG: crc::Algorithm<u32> = crc::Algorithm {
    width: 32,
    poly: 0x04C1_1DB7,
    init: 0xFFFF_FFFF,
    refin: false,
    refout: false,
    xorout: 0x0000_0000,
    check: 0x0000_0000,   // todo ...
    residue: 0x0000_0000, // todo ...
};

const MAX_DATA_LENGTH: usize = 200;

/**
 * Packet format for sending firmware updates, via USB.
 */
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[repr(C, packed)] // Keep the field order (very important for firmware updates)
pub struct FirmwareUpdatePacket {
    boot_char: u8,   // should always be '*'
    update_char: u8, // should always be 'u'
    dummy1: u16,     // *padding*
    address: u32,    // destination address
    data_length: u8, // num of byte of data in the data region of the packet
    dummy2: u8,      // *padding*
    data_crc: u16,   // (CCIT) CRC 16 of data.
    #[serde(with = "BigArray")]
    data: [u8; MAX_DATA_LENGTH],
    end_of_packet: u8, // should always be '\n'
    dummy3: u16,       // *padding*
    dummy4: u8,        // *padding*
}

fn crc_toying_about() {
    let crc = crc::Crc::<u32>::new(&CUSTOM_ALG);
    let mut digest = crc.digest();
    digest.update(b"123456789abcdef0");
    let yummy = digest.finalize();
    println!("\nComputing CRC32");
    println!(" - CRC32: 0x{:08X}\n", yummy);
    assert_eq!(yummy, 0xa19a6e15);
}

fn main() {
    let data = std::fs::read_to_string("data/example.hex").unwrap();
    let reader = ihex::Reader::new_with_options(
        &data,
        ihex::ReaderOptions {
            stop_after_first_error: true,
            stop_after_eof: true,
        },
    );
    let mut records: Vec<ihex::Record> = reader.into_iter().filter_map(|x| x.ok()).collect();
    let regions = build_regions(&mut records);
    if !regions.is_empty() {
        println!("\nFound {} HEX regions", regions.len());
    }
    for r in regions.iter() {
        println!(" - Region: ADDR = {:08x}, SIZE = {}", r.address(), r.len());
    }

    crc_toying_about();

    println!("Hello, world!");
}
