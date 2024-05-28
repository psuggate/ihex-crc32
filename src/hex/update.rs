use crate::hexcrc::CUSTOM_ALG;
use crate::packet::FirmwareUpdatePacket;

/**
 * Complete update for an Lt Sensor.
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
        let crc = crc::Crc::<u32>::new(&CUSTOM_ALG);
        let mut digest = crc.digest();
        for p in packets.iter() {
            digest.update(&p.to_vec());
        }
        let crc32 = digest.finalize();
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
}
