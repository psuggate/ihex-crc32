# ihex-crc32

Reads Intel HEX files and generates 32-bit CRCs

Can generate C include files, and binary files:
```bash
$ cargo run -- -f FILE.HEX -i FILE.H -b FILE.BIN [-v] [-v]
```

Output of the BIN file can be displayed using `hexdump`:
```bash
$ hexdump -C FILE.BIN
```
