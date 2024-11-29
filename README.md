# Disk Testing Utility

A high-performance disk testing utility written in Rust that fills drives with pseudorandom patterns and verifies them. This tool was created as an alternative to `badblocks` for testing large drives (10TB+) where traditional tools may be too slow or resource-intensive.

## Features

- Fast pseudorandom pattern generation using xoshiro256** algorithm
- Progress monitoring with real-time speed measurements
- Configurable block sizes for optimal performance
- Separate write and verification modes
- Low CPU usage compared to Python alternatives
- Support for large drives (tested on 18TB drives)

## Installation

1. Ensure you have Rust installed on your system. If not, install it from [https://rustup.rs/](https://rustup.rs/)
2. Clone this repository:
```bash
git clone [repository-url]
cd disk-test-utility
```
3. Build the project:
```bash
cargo build --release
```

The compiled binary will be available in `target/release/`

## Usage

The utility requires root privileges to access disk devices directly.

### Writing Random Patterns

```bash
sudo ./disk-test-utility --write --seed 1 --blocksize 1M /dev/sdX
```

### Verifying Written Patterns

```bash
sudo ./disk-test-utility --read --seed 1 --blocksize 1M /dev/sdX
```

### Parameters

- `--write`: Write mode
- `--read`: Verification mode
- `--seed`: Seed for the random pattern generator (must be the same for write and verify)
- `--blocksize`: Block size for I/O operations (supports K, M, G suffixes)
- Last parameter: Target device path

## Alternative: Writing Zeros

If you need to write zeros to a disk instead of random patterns, you can use these standard Linux commands:

1. Write zeros to the entire disk:
```bash
dd if=/dev/zero of=/dev/sdX bs=1M status=progress
```

2. Verify zeros (any of these methods):
```bash
# Quick check of first few megabytes
dd if=/dev/sdX bs=1M count=10 | hexdump -C

# Complete verification
cmp -b /dev/zero /dev/sdX

# Alternative verification (shows only non-zero bytes)
dd if=/dev/sdX bs=1M status=progress | od -A x -t x1z | grep -v "0000 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00"
```

## Safety Warning

⚠️ **This tool will overwrite all data on the target device. Double-check the device path before running!**

The tool requires root privileges and can potentially destroy data if used incorrectly. Always verify the target device path carefully.

## Technical Details

- Uses the xoshiro256** algorithm for high-quality random number generation
- Implements efficient buffer filling using 64-bit operations
- Provides real-time progress monitoring with MB/s measurements
- Written in Rust for high performance and memory safety


## License

MIT License

## Author

bbitmaster@gmail.com with AI assistance

## Acknowledgments

- Thanks to the Rust community for providing excellent tools and documentation
- Inspired by limitations encountered with badblocks on large modern drives

