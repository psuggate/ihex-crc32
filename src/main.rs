pub(crate) use hex::*;
pub(crate) mod hex;

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
    let mut regions = build_regions(&mut records);
    if !regions.is_empty() {
        println!("\nFound {} HEX regions", regions.len());
    }
    for r in regions.iter() {
        println!(" - Region: ADDR = {:08x}, SIZE = {}", r.address(), r.len());
    }

    let mut regions = merge_regions(&mut regions);
    if !regions.is_empty() {
        println!("\nFound {} HEX regions", regions.len());
    }
    for r in regions.iter() {
        println!(" - Region: ADDR = {:08x}, SIZE = {}", r.address(), r.len());
    }

    let packets = make_packets(&mut regions);
    if !packets.is_empty() {
        println!("\nFound {} HEX packets", packets.len());
    }
    for r in packets.iter() {
        println!(
            " - Packet: ADDR = {:08x}, SIZE = {}, CRC16 = 0x{:04X}",
            r.address(),
            r.len(),
            r.crc16()
        );
    }

    crc_toying_about();

    println!("Hello, world!");
}
