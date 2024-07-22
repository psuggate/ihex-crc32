pub(crate) use hex::*;
pub(crate) mod hex;
use clap::Parser;

// -- Data types for command-line options -- //
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, value_name = "IN.HEX", default_value = "data/example.hex")]
    file: String,

    #[arg(short, long, value_name = "OUT.H")]
    include: Option<String>,

    #[arg(short, long, value_name = "OUT.BIN")]
    binary: Option<String>,

    #[arg(short, long)]
    append_crc: bool,

    /// Verbosity of generated output?
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
}

#[allow(unused)]
pub fn test_checksum() {
    const BYTES: [u8; 12] = [
        0x08, 0x07, 0xf8, 0x00, 0x04, 0xf0, 0x3c, 0xfa, 0x69, 0x46, 0x1a, 0x48,
    ];
    let cs = hex::hexcrc::ihex_checksum(&BYTES);
    println!("Checksum: {:02x}", cs);
}

#[allow(unused)]
pub fn test_stm32crc() {
    const STR: &str = "A test-string for CRC32 checking\n";
    let crc = crc::Crc::<u32>::new(&hex::hexcrc::CUSTOM_ALG);
    let val = crc.checksum(STR.as_bytes());
    println!("STM32CRC: {:08x} (len = {})", val, STR.len());
}

fn main() {
    let args = Args::parse();
    let path = args.file;
    let data = std::fs::read_to_string(path).unwrap();
    let reader = ihex::Reader::new_with_options(
        &data,
        ihex::ReaderOptions {
            stop_after_first_error: true,
            stop_after_eof: true,
        },
    );
    let mut records: Vec<ihex::Record> = reader.into_iter().filter_map(|x| x.ok()).collect();
    let regions = Region::build_regions(&mut records);
    if !regions.is_empty() {
        println!("\nFound {} HEX regions", regions.len());
    }
    if args.verbose > 0 {
        for r in regions.iter() {
            println!(" - Region: ADDR = {:08x}, SIZE = {}", r.address(), r.len());
        }
    }

    let mut regions = merge_regions(&regions);
    if args.verbose > 0 {
        if !regions.is_empty() {
            println!("\nFound {} HEX regions", regions.len());
        }
        for r in regions.iter() {
            println!(" - Region: ADDR = {:08x}, SIZE = {}", r.address(), r.len());
        }
    }

    let packets = if let Some(mut r) = Region::single_region(&regions) {
        //
        //  M O N O  !!
        //
        println!("\nBuild HEX mono-region");
        println!(" - Region: ADDR = {:08x}, SIZE = {}", r.address(), r.len());

        let packets = r.to_packets(!args.append_crc);
        if args.verbose > 0 && !packets.is_empty() {
            println!("\nFound {} HEX packets", packets.len());
        }
        if args.verbose > 1 {
            for p in packets.iter() {
                println!(
                    " - Packet: ADDR = {:08x}, SIZE = {}, CRC16 = 0x{:04X}",
                    p.address(),
                    p.len(),
                    p.crc16()
                );
            }
        }
        packets
    } else {
        let packets = make_packets(&mut regions);
        if args.verbose > 0 && !packets.is_empty() {
            println!("\nFound {} HEX packets", packets.len());
        }
        if args.verbose > 1 {
            for p in packets.iter() {
                println!(
                    " - Packet: ADDR = {:08x}, SIZE = {}, CRC16 = 0x{:04X}",
                    p.address(),
                    p.len(),
                    p.crc16()
                );
            }
        }
        packets
    };

    let update = FirmwareUpdate::new(packets);
    println!("\nFirmware update:");
    println!(" - Length: {}", update.len());
    println!(" - CRC32:  0x{:08X}", update.crc32());
    println!();

    if args.verbose > 1 {
        println!("{}", to_include_text(&update, args.append_crc));
    }

    if let Some(filename) = args.include {
        if args.verbose > 0 {
            println!("\nWriting '{}'", &filename);
        }
        to_include_file(&filename, &update, args.append_crc);
    }
    if let Some(filename) = args.binary {
        if args.verbose > 0 {
            println!("\nWriting '{}'", &filename);
        }
        to_binary_file(&filename, &update, args.append_crc);
    }
}
