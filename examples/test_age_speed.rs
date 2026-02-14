use std::io::Write;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let size_mb = 50;
    println!("Generating {}MB of random data...", size_mb);
    // Simple LCG to generate "random" data that isn't trivially compressible/optimizable
    let mut input = Vec::with_capacity(size_mb * 1024 * 1024);
    let mut state: u32 = 123456789;
    for _ in 0..(size_mb * 1024 * 1024) {
        state = state.wrapping_mul(1664525).wrapping_add(1013904223);
        input.push((state >> 24) as u8);
    }
    
    println!("Starting Encryption (Age, x25519)...");
    let start = Instant::now();
    
    let chunk_size = 64 * 1024;
    let mut output = Vec::with_capacity(input.len() + 1024);
    
    // Use x25519 to skip Scrypt delay and measure stream throughput
    let key = age::x25519::Identity::generate();
    let pubkey = key.to_public();
    
    let encryptor = age::Encryptor::with_recipients(vec![Box::new(pubkey)])
        .expect("Failed to create encryptor");
        
    let mut writer = encryptor.wrap_output(&mut output)?;
    writer.write_all(&input)?;
    writer.finish()?;
    
    let duration = start.elapsed();
    let bytes = input.len();
    let throughput = (bytes as f64 / 1024.0 / 1024.0) / duration.as_secs_f64();
    
    println!("Encryption completed: {} bytes -> {} bytes", bytes, output.len());
    println!("Time: {:.2?}", duration);
    println!("Throughput: {:.2} MB/s", throughput);
    
    Ok(())
}
