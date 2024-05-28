use ihex::Record;

#[derive(Eq, PartialEq, Debug, Clone)]
pub struct Region {
    base: u32,
    data: Vec<u8>,
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

    pub fn address(&self) -> u32 {
        self.base
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
        if !region.data.is_empty() {
            regions.push(region.clone());
            // Start a new region
            region.base = segment;
            region.data = Vec::new();
        }
        pointer = 0;
    }
    regions
}
