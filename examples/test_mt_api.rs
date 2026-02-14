use std::io::{Write};
use std::time::Instant;

use rayon::prelude::*;
use brotli::enc::BrotliEncoderParams;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Generate Data
    let size_mb = 20;
    println!("Generating {}MB of data...", size_mb);
    let data_len = size_mb * 1024 * 1024;
    let input: Vec<u8> = (0..data_len).map(|i| (i % 251) as u8).collect();
    
    // 2. Multithreaded Compression using Rayon (Manual chunking)
    // This simulates the user's manual implementation in src/codec.rs
    println!("Starting Compression (Rayon + Brotli)...");
    let start_total = Instant::now();
    let start_comp = Instant::now();
    
    let chunk_size = 1024 * 1024; // 1MB chunks
    let chunks: Vec<&[u8]> = input.chunks(chunk_size).collect();
    
    let compressed_chunks: Vec<Vec<u8>> = chunks.par_iter().map(|chunk| {
        let mut buffer = Vec::new();
        let mut params = BrotliEncoderParams::default();
        params.quality = 4; // Balanced quality
        {
            let mut compressor = brotli::CompressorWriter::with_params(&mut buffer, 4096, &params);
            compressor.write_all(chunk).unwrap();
        }
        buffer
    }).collect();
    
    let mut compressed_output = Vec::new();
    for chunk in &compressed_chunks {
        compressed_output.extend_from_slice(chunk);
    }
    
    let duration_comp = start_comp.elapsed();
    println!("Compression completed: {} bytes -> {} bytes", input.len(), compressed_output.len());
    println!("Compression Time: {:.2?}", duration_comp);
    
    // 3. Encryption (Age) - Single Threaded
    println!("Starting Encryption (Age)...");
    let start_enc = Instant::now();
    
    let mut final_output = Vec::new();
    
    // Generate a random identity/recipient
    let key = age::x25519::Identity::generate();
    let pubkey = key.to_public();
    
    let encryptor = age::Encryptor::with_recipients(vec![Box::new(pubkey)])
        .expect("Failed to create encryptor");
        
    let mut writer = encryptor.wrap_output(&mut final_output)?;
    writer.write_all(&compressed_output)?;
    writer.finish()?;
    
    let duration_enc = start_enc.elapsed();
    println!("Encryption completed: {} bytes -> {} bytes", compressed_output.len(), final_output.len());
    println!("Encryption Time: {:.2?}", duration_enc);
    
    let duration_total = start_total.elapsed();
    println!("Total Time: {:.2?}", duration_total);
    
    // Throughput
    let mb = size_mb as f64;
    println!("Throughput: {:.2} MB/s", mb / duration_total.as_secs_f64());

    Ok(())
}
