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

pub fn calc_stm32_crc(data: &[u8]) -> u32 {
    let crc = crc::Crc::<u32>::new(&CUSTOM_ALG);
    let mut digest = crc.digest();
    digest.update(data);
    digest.finalize()
}

pub fn crc_toying_about() {
    let crc = crc::Crc::<u32>::new(&CUSTOM_ALG);
    let mut digest = crc.digest();
    digest.update(b"123456789abcdef0");
    let yummy = digest.finalize();
    println!("\nComputing CRC32");
    println!(" - CRC32: 0x{:08X}\n", yummy);
    assert_eq!(yummy, 0xa19a6e15);
}
