# ihex-crc32

Reads Intel HEX files and generates 32-bit CRCs

Can generate C include files, and binary files:
```bash
$ cargo run -- -f FILE.HEX [-i FILE.H] [-b FILE.BIN] [-a] [-v] [-v]
```

Output of the BIN file can be displayed using `hexdump`:
```bash
$ hexdump -C FILE.BIN
```

The '`--append-crc`'/'`-a`' option appends a 32-bit CRC value to the end of the ROM, prior to padding for 8-byte alignment.
