pub use append::{to_binary_file, to_include_file, to_include_text};
pub mod append;
pub mod hexcrc;
pub use region::*;
pub mod region;
pub use packet::*;
pub mod packet;
pub use update::*;
pub mod update;

// OBSOLETE
pub fn make_packets(regions: &mut [Region]) -> Vec<FirmwareUpdatePacket> {
    let mut packets = Vec::new();

    for r in regions.iter_mut() {
        let mut fwups = r.to_packets(true);
        packets.append(&mut fwups);
    }
    packets
}
