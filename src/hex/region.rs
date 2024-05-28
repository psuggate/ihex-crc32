use ihex::Record;
use std::cmp::Ordering;

use crate::packet::{FirmwareUpdatePacket, MAX_DATA_LENGTH};

/**
 * Represents a single contiguous region of 'u8' values, read from a HEX file.
 */
#[derive(Eq, PartialEq, Debug, Clone)]
pub struct Region {
    base: u32,
    data: Vec<u8>,
}

impl Ord for Region {
    fn cmp(&self, other: &Region) -> Ordering {
        self.base.cmp(&other.base)
    }
}

impl PartialOrd for Region {
    fn partial_cmp(&self, other: &Region) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Region {
    pub fn new(base: u32) -> Self {
        Self {
            base,
            data: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn address(&self) -> u32 {
        self.base
    }

    pub fn make_packets(&mut self) -> Vec<FirmwareUpdatePacket> {
        let mut packets = Vec::new();
        let mut addr = self.base;
        let mut iter = self.data.chunks_exact(MAX_DATA_LENGTH);

        loop {
            if let Some(c) = iter.next() {
                let data: [u8; MAX_DATA_LENGTH] = c.try_into().unwrap();
                let fwup = FirmwareUpdatePacket::new(addr, data, MAX_DATA_LENGTH);
                packets.push(fwup);
                addr += MAX_DATA_LENGTH as u32;
            } else {
                let mut last = Vec::with_capacity(MAX_DATA_LENGTH);
                last.extend(iter.remainder());
                let size = last.len();
                assert!(size & 0x7 == 0);

                if size > 0 {
                    last.resize(MAX_DATA_LENGTH, 0);
                    let data: [u8; MAX_DATA_LENGTH] = last.try_into().unwrap();
                    let fwup = FirmwareUpdatePacket::new(addr, data, size);
                    packets.push(fwup);
                }
                break;
            }
        }
        packets
    }
}

/**
 * Build an array of (upto) 64 kB "regions" of firmware (binary-)data.
 */
pub fn build_regions(records: &mut [Record]) -> Vec<Region> {
    let mut regions: Vec<Region> = Vec::new();
    let mut segment: u32 = 0;
    let mut pointer: u32 = 0;
    let mut region = Region::new(0);

    for r in records.iter_mut() {
        match r {
            Record::Data {
                offset,
                ref mut value,
            } => {
                // Type: 0x00
                // Append data to the current region, if contiguous
                let offset = *offset as u32;
                let length = value.len() as u32;

                if length > 0 {
                    if region.data.is_empty() {
                        // First data, so set full 32-bit address
                        region.base += offset;
                    } else if offset != pointer {
                        // Data isn't contiguous, so store the current region,
                        // and then start a new region
                        regions.push(region.clone());
                        region.base = segment + offset;
                        region.data = Vec::new();
                    }
                    region.data.append(value);
                    pointer = offset + length;
                }
                continue;
            }
            Record::EndOfFile => {
                // Type: 0x01
                segment = 0;
            }
            Record::ExtendedSegmentAddress(base) => {
                // Type: 0x02
                segment = (*base as u32) << 4;
            }
            Record::StartSegmentAddress { cs: _, ip: _ } => {
                // Type: 0x03 (ignored, for firmware updates)
                segment = 0;
            }
            Record::ExtendedLinearAddress(base) => {
                // Type: 0x04
                // Start a new region (even if contiguous)
                segment = (*base as u32) << 16;
            }
            Record::StartLinearAddress(_) => {
                // Type: 0x05 (ignored, for firmware updates)
                segment = 0;
            }
        }

        // Found a non-data record, so store the current 'Region', if non-zero.
        if !region.is_empty() {
            regions.push(region.clone());
            // Start a new region
            region.data = Vec::new();
        }
        region.base = segment;
        pointer = 0;
    }
    regions.sort();
    regions
}

//
// Todo:
//  - merge regions if they are contiguous after aligning and padding to eight
//    bytes, as this alignment is required for the STM32G4xx FLASH writer;
//  - compute the image-lengths & CRC32 values using the padded/aligned image
//    data ??
//
pub fn merge_regions(regions: &[Region]) -> Vec<Region> {
    let mut result: Vec<Region> = Vec::new();
    let mut iter = regions.iter();

    let mut prev = if let Some(prev) = iter.next() {
        prev.clone()
    } else {
        return result;
    };

    loop {
        let mut curr = if let Some(curr) = iter.next() {
            curr.clone()
        } else {
            // No more 'Region's
            result.push(prev);
            break;
        };

        // Compute the index of the last byte of the previous 'Region'
        let last = prev.base as usize + prev.data.len() - 1;

        // Compute the index of the start of the next 'u64'-aligned chunk, if
        // 'Region's are contiguous
        let next = (last + 8) & 0xfffffff8;
        // let next = (last + 16) & 0xfffffff0;
        let base = curr.base as usize;
        println!(
            "prev: 0x{:08X}, last: 0x{:08X}, next: 0x{:08X}, base: 0x{:08X}",
            prev.base, last, next, base
        );

        if base <= next {
            // Start of 'Region' is contiguous with the previous 'Region'
            // once aligned and padded (if required)
            let npad = base - last - 1;
            let mut pads = vec![0; npad];
            prev.data.append(&mut pads);
            prev.data.append(&mut curr.data);
        } else {
            // We are done with 'prev: Region', so
            result.push(prev.clone());
            prev = curr;
        }
    }
    result
}

pub fn make_packets(regions: &mut [Region]) -> Vec<FirmwareUpdatePacket> {
    let mut packets = Vec::new();

    for r in regions.iter_mut() {
        let mut fwups = r.make_packets();
        packets.append(&mut fwups);
    }
    packets
}
