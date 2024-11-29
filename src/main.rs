use std::fs::{File, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::time::Instant;

// Fast xoshiro256** PRNG
struct Xoshiro256StarStar {
    s: [u64; 4]
}

impl Xoshiro256StarStar {
    fn new(seed: u64) -> Self {
        // Initialize state using SplitMix64
        let mut state = [0u64; 4];
        let mut x = seed;
        for i in 0..4 {
            x = x.wrapping_add(0x9e3779b97f4a7c15);
            let mut z = x;
            z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
            state[i] = z ^ (z >> 31);
        }
        Self { s: state }
    }

    fn next_u64(&mut self) -> u64 {
        let result = self.s[1].wrapping_mul(5).rotate_left(7).wrapping_mul(9);
        let t = self.s[1] << 17;
        self.s[2] ^= self.s[0];
        self.s[3] ^= self.s[1];
        self.s[1] ^= self.s[2];
        self.s[0] ^= self.s[3];
        self.s[2] ^= t;
        self.s[3] = self.s[3].rotate_left(45);
        result
    }
}

fn parse_size(size_str: &str) -> Result<usize, String> {
    let len = size_str.len();
    if len < 2 {
        return Err("Invalid size format".to_string());
    }
    
    let (num_str, unit) = size_str.split_at(len - 1);
    let num = num_str.parse::<usize>().map_err(|_| "Invalid number")?;
    
    match unit.to_uppercase().as_str() {
        "K" => Ok(num * 1024),
        "M" => Ok(num * 1024 * 1024),
        "G" => Ok(num * 1024 * 1024 * 1024),
        _ => Err("Invalid unit (use K, M, or G)".to_string()),
    }
}


fn fill_buffer(rng: &mut Xoshiro256StarStar, buf: &mut [u8]) {
    let mut i = 0;
    
    // Fill 8 bytes at a time
    while i + 8 <= buf.len() {
        let val = rng.next_u64().to_le_bytes();
        buf[i..i+8].copy_from_slice(&val);
        i += 8;
    }
    
    // Handle remaining bytes
    if i < buf.len() {
        let val = rng.next_u64().to_le_bytes();
        let remaining = buf.len() - i;
        buf[i..].copy_from_slice(&val[..remaining]);
    }
}

fn write_to_disk(device: &str, seed: u64, block_size: usize) -> io::Result<()> {
    let mut file = OpenOptions::new().read(true).write(true).open(device)?;
    let total_size = file.seek(SeekFrom::End(0))?;
    file.seek(SeekFrom::Start(0))?;

    let mut written = 0u64;
    let start_time = Instant::now();
    let mut last_update = start_time;
    let mut rng = Xoshiro256StarStar::new(seed);
    let mut buffer = vec![0u8; block_size];

    while written < total_size {
        let remaining = total_size - written;
        let chunk_size = std::cmp::min(block_size as u64, remaining) as usize;
        
        fill_buffer(&mut rng, &mut buffer[..chunk_size]);
        file.write_all(&buffer[..chunk_size])?;
        written += chunk_size as u64;

        let now = Instant::now();
        if now.duration_since(last_update).as_secs() >= 1 {
            let progress = (written as f64 / total_size as f64) * 100.0;
            let elapsed = now.duration_since(start_time).as_secs_f64();
            let speed = written as f64 / (1024.0 * 1024.0 * elapsed);

            print!("\rProgress: {:.2}% | Speed: {:.2} MB/s | Written: {} MB", 
                  progress, speed, written / (1024 * 1024));
            io::stdout().flush()?;
            last_update = now;
        }
    }
    println!("\nWrite operation completed.");
    Ok(())
}

fn read_and_verify(device: &str, seed: u64, block_size: usize) -> io::Result<()> {
    let mut file = File::open(device)?;
    let total_size = file.seek(SeekFrom::End(0))?;
    file.seek(SeekFrom::Start(0))?;

    let mut read = 0u64;
    let mut mismatches = 0u64;
    let start_time = Instant::now();
    let mut last_update = start_time;
    let mut rng = Xoshiro256StarStar::new(seed);
    let mut read_buffer = vec![0u8; block_size];
    let mut expected_buffer = vec![0u8; block_size];

    while read < total_size {
        let remaining = total_size - read;
        let chunk_size = std::cmp::min(block_size as u64, remaining) as usize;
        
        // Read actual data
        file.read_exact(&mut read_buffer[..chunk_size])?;
        
        // Generate expected pattern
        fill_buffer(&mut rng, &mut expected_buffer[..chunk_size]);
        
        // Compare buffers
        if read_buffer[..chunk_size] != expected_buffer[..chunk_size] {
            mismatches += 1;
        }
        
        read += chunk_size as u64;

        let now = Instant::now();
        if now.duration_since(last_update).as_secs() >= 1 {
            let progress = (read as f64 / total_size as f64) * 100.0;
            let elapsed = now.duration_since(start_time).as_secs_f64();
            let speed = read as f64 / (1024.0 * 1024.0 * elapsed);

            print!("\rProgress: {:.2}% | Speed: {:.2} MB/s | Read: {} MB | Mismatches: {}", 
                  progress, speed, read / (1024 * 1024), mismatches);
            io::stdout().flush()?;
            last_update = now;
        }
    }
    println!("\nVerification completed. Total mismatches: {}", mismatches);
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    
    // Basic argument parsing
    let mut write_mode = false;
    let mut read_mode = false;
    let mut seed = None;
    let mut blocksize = None;
    let mut device = None;
    
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--write" => write_mode = true,
            "--read" => read_mode = true,
            "--seed" => {
                i += 1;
                if i < args.len() {
                    seed = Some(args[i].parse::<u64>().map_err(|_| "Invalid seed")?);
                }
            },
            "--blocksize" => {
                i += 1;
                if i < args.len() {
                    blocksize = Some(args[i].clone());
                }
            },
            arg if !arg.starts_with("--") => device = Some(arg.to_string()),
            _ => {}
        }
        i += 1;
    }

    // Validate arguments
    if write_mode == read_mode {
        eprintln!("Error: Please specify either --write or --read mode, but not both.");
        std::process::exit(1);
    }

    let device = device.ok_or("No device specified")?;
    let seed = seed.ok_or("No seed specified")?;
    let blocksize = blocksize.ok_or("No blocksize specified")?;

    // Check for root privileges on Unix
    #[cfg(unix)]
    if unsafe { libc::geteuid() } != 0 {
        eprintln!("This script must be run with sudo privileges.");
        std::process::exit(1);
    }

    let block_size = parse_size(&blocksize)?;

    if write_mode {
        println!("Writing to {} with seed {} and block size {}", device, seed, blocksize);
        write_to_disk(&device, seed, block_size)?;
    } else {
        println!("Verifying {} with seed {} and block size {}", device, seed, blocksize);
        read_and_verify(&device, seed, block_size)?;
    }

    Ok(())
}
