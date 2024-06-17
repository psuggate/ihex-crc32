use lazy_static::lazy_static;

use super::packet::{FirmwareUpdatePacket, MAX_DATA_LENGTH};
use super::update::FirmwareUpdate;

const HEADER_COMMENT: &str = "/**
 * This file contains the binary data of bootloader firmware image, so that the
 * Lt Sensor application firmware can update the bootloader, if needed.
 *
 * This file has been generated automatically, via:
 *  cargo run -- -f $(LTFW)/adi_boot_fw/Release/adi_boot_fw.hex \\
 *               -i $(LTFW)/adi_fw/Inc/adiBootloaderFirmware.h
 *
 * See:
 *  https://github.com/psuggate/ihex-crc32.git
 *
 * The data can also be generated with the following command:
 *   xxd -i adi_boot_fw/Release/adi_boot_fw.bin > temp.h
 *
 * Note: you need to make the BIN file manually, using:
 *   arm-none-eabi-objcopy -O binary adi_boot_fw/Release/adi_boot_fw.elf \\
 *       adi_boot_fw.bin
 */\n";

const HEADER_INCLUDE: &str = "\n#pragma once\n#include <stdint.h>\n\n";

const CRC32_COMMENT: &str = "/**
 * Note(s):
 *  - after bootloader v1.1.0, CRC32 verification is required for firmware;
 *  - there is also CRC32 generator: 'util/fwcrc32.c', within the Lt Sensors
 *    firmware Git repository, and this can be built using 'make', if a suitable
 *    GNU build environment has been set up;
 */\n";
const CRC32_DECLARE: &str = "const uint32_t kBootloaderFirmwareCrc = 0x";
const IMAGE_DECLARE: &str = "ul;\n\nconst uint8_t kBootloaderFirmwareBin[] = {\n\t";
const IMAGE_COMPLETE: &str = "\n};\n";

const MAX_COLUMNS: usize = 12;

// Global (and lazily-initialised) store for all device labels, and counters
lazy_static! {
    static ref HEX_TABLE: String = make_hex_table();
}

fn make_hex_table() -> String {
    let mut table = "".to_string();
    for i in 0..256u32 {
        let x = format!("0x{:02x}", i & 0xff);
        table.push_str(&x);
    }
    table
}

fn to_bytes(packet: &FirmwareUpdatePacket, col: &mut usize) -> String {
    let mut bytes: String = "".to_string();
    let data = packet.to_vec();
    let mut iter = data.iter();

    if let Some(x) = iter.next() {
        let s = (*x) as usize * 4;
        let e = s + 4;
        bytes.push_str(&HEX_TABLE[s..e]);
    }

    for x in iter {
        *col += 1;
        if *col == MAX_COLUMNS {
            bytes.push_str(",\n\t");
            *col = 0;
        } else {
            bytes.push_str(", ");
        }
        let s = (*x) as usize * 4;
        let e = s + 4;
        bytes.push_str(&HEX_TABLE[s..e]);
    }

    if packet.len() >= MAX_DATA_LENGTH {
        *col += 1;
        if *col == MAX_COLUMNS {
            bytes.push_str(",\n\t");
            *col = 0;
        } else {
            bytes.push_str(", ");
        }
    }

    bytes
}

pub fn to_include_file(update: &FirmwareUpdate, filename: &str) {
    let crc32 = update.crc32();
    let mut bytes: String = "".to_string();
    let mut col: usize = 0;
    for p in update.packets() {
        let bs = to_bytes(p, &mut col);
        bytes.push_str(&bs);
    }
    let mut contents: String = HEADER_COMMENT.to_string();
    contents.push_str(HEADER_INCLUDE);
    contents.push_str(CRC32_COMMENT);
    contents.push_str(&format!("{}{:08X}", CRC32_DECLARE, crc32));
    contents.push_str(IMAGE_DECLARE);
    contents.push_str(&bytes);
    contents.push_str(IMAGE_COMPLETE);

    std::fs::write(filename, contents).unwrap()
}

pub fn to_binary_file(update: &FirmwareUpdate, filename: &str, append_crc: bool) {
    let len: usize = if append_crc {
        (update.len() + 11) & !0x07
    } else {
        (update.len() + 7) & !0x07
    };
    let alg = crc::Crc::<u32>::new(&super::hexcrc::CUSTOM_ALG);
    let mut dig = alg.digest();
    let mut bytes: Vec<u8> = Vec::with_capacity(len);
    for p in update.packets() {
        let mut dat = p.to_vec();
        dig.update(&dat);
        bytes.append(&mut dat);
    }
    assert!(update.crc32() == dig.finalize());
    if append_crc {
        let crc = update.crc32();
        unsafe {
            println!(
                "Appending '0x{:08X}ul' to the Lt Sensor bootloader (length = {})",
                crc, len
            );
            let crc_bytes: [u8; 4] = std::mem::transmute::<u32, [u8; 4]>(crc);
            // Check that the host system is Little Endian
            assert!(crc_bytes[0] as u32 == crc & 0x0ff);
            bytes.extend(crc_bytes);
        };
    }
    while bytes.len() < len {
        bytes.push(0);
    }
    std::fs::write(filename, &bytes).unwrap()
}
