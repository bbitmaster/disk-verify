# Disk Testing Utility

High-performance disk testing utility in Rust for validating large drives (10TB+) using pseudorandom patterns

## Installation

1. Install Rust:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

2. Clone and build:
```bash
git clone [repository-url]
cd disk-verify
cargo build --release
```

The compiled binary will be in `target/release/disk-verify`

## Usage

The utility requires root privileges to access disk devices directly.

### Writing Random Patterns

```bash
sudo ./disk-verify --write --seed 1 --blocksize 1M /dev/sdX
```

### Verifying Written Patterns

```bash
sudo ./disk-verify --read --seed 1 --blocksize 1M /dev/sdX
```

### Parameters

- `--write`: Write mode
- `--read`: Verification mode
- `--seed`: Seed for the random pattern generator (must be same for write and verify)
- `--blocksize`: Block size for I/O operations (supports K, M, G suffixes)
- Last parameter: Target device path

## Alternative: Writing Zeros

To write zeros instead of random patterns:

1. Write zeros:
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

## Safety Warning ⚠️

**This tool will overwrite all data on the target device. Double-check the device path before running!**

The tool requires root privileges and can permanently destroy data if used incorrectly. Always verify the target device path.

## Technical Details

- Uses xoshiro256** algorithm for high-quality random number generation
- Efficient buffer filling using 64-bit operations
- Real-time progress monitoring with MB/s measurements
- Written in Rust for high performance and memory safety
- Low CPU usage compared to Python alternatives

## Author

bbitmaster@gmail.com with AI assistance

## License

MIT License

