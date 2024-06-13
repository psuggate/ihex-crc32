use crate::hexcrc::calc_stm32_crc;
use crate::packet::FirmwareUpdatePacket;

/**
 * Complete firmware update.
 */
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct FirmwareUpdate {
    packets: Vec<FirmwareUpdatePacket>,
    length: usize,
    crc32: u32,
}

impl FirmwareUpdate {
    pub fn new(packets: Vec<FirmwareUpdatePacket>) -> Self {
        let length = packets.iter().fold(0, |s, x| s + x.len());
        let crc32 = calc_stm32_crc(&packets);
        Self {
            packets,
            length,
            crc32,
        }
    }
    pub fn len(&self) -> usize {
        self.length
    }
    pub fn crc32(&self) -> u32 {
        self.crc32
    }
    pub fn packets(&self) -> &[FirmwareUpdatePacket] {
        &self.packets
    }
}
