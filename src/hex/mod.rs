pub use append::*;
pub mod append;
// pub use hexcrc::*;
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
        let mut fwups = r.to_packets();
        packets.append(&mut fwups);
    }
    packets
}
