use std::fs::File;
use std::io::{self, Read, Write};
use std::path::Path;
use age::secrecy::{Secret, ExposeSecret};
use brotli::enc::BrotliEncoderParams;
use brotli::{BrotliDecompressStream, BrotliResult, BrotliState};
use brotli::enc::StandardAlloc;
use rayon::prelude::*;

pub fn encrypt_stream(
    input_path: &Path,
    output_path: &Path,
    password: Secret<String>,
    quality: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    let start_total = std::time::Instant::now();
    
    // Read entire input into memory for parallel compression
    // Assuming DB size is manageable (e.g. < 1GB)
    let start_read = std::time::Instant::now();
    let input_data = std::fs::read(input_path)?;
    println!("Read input: {:?}", start_read.elapsed());
    
    // Chunk size for parallel processing (e.g. 1MB)
    const CHUNK_SIZE: usize = 1024 * 1024;
    let chunks: Vec<&[u8]> = input_data.chunks(CHUNK_SIZE).collect();

    let start_comp = std::time::Instant::now();
    let compressed_chunks: Result<Vec<Vec<u8>>, std::io::Error> = chunks
        .par_iter()
        .map(|chunk| {
            let mut compressed = Vec::new();
            let mut params = BrotliEncoderParams::default();
            params.quality = quality as i32;
            // We use default params which create independent streams.
            // Concatenating them forms a valid multi-stream Brotli file.
            
            {
                let mut compressor = brotli::CompressorWriter::with_params(&mut compressed, 4096, &params);
                compressor.write_all(chunk)?;
            } // Compressor is dropped here, flushing data
            
            Ok(compressed)
        })
        .collect();

    let compressed_chunks = compressed_chunks?;
    println!("Compression (Rayon): {:?}", start_comp.elapsed());

    // Create the encrypted output file
    let output_file = File::create(output_path)?;
    // Wrap output file in a BufWriter for better performance
    let output_writer = io::BufWriter::with_capacity(65536, output_file);
    
    let start_enc = std::time::Instant::now();
    let encryptor = age::Encryptor::with_user_passphrase(Secret::new(password.expose_secret().clone()));
    let mut writer = encryptor.wrap_output(output_writer)?;

    // Write all compressed chunks sequentially
    for chunk in compressed_chunks {
        writer.write_all(&chunk)?;
    }
    
    writer.finish()?;
    println!("Encryption: {:?}", start_enc.elapsed());
    println!("Total Encrypt Time: {:?}", start_total.elapsed());

    Ok(())
}

pub fn decrypt_stream(
    input_path: &Path,
    output_path: &Path,
    password: Secret<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let start_total = std::time::Instant::now();
    const BUFFER_SIZE: usize = 65536;
    let input_file = File::open(input_path)?;
    // Use BufReader for input to reduce syscalls during decryption
    let input_reader = io::BufReader::with_capacity(BUFFER_SIZE, input_file);
    
    let output_file = File::create(output_path)?;
    // Wrap output in BufWriter
    let mut output_writer = io::BufWriter::with_capacity(BUFFER_SIZE, output_file);

    let start_dec = std::time::Instant::now();
    let decryptor = match age::Decryptor::new(input_reader)? {
        age::Decryptor::Passphrase(d) => d,
        _ => return Err("Input file is not encrypted with a passphrase".into()),
    };

    let mut reader = decryptor.decrypt(&password, None)?;
    println!("Decryption Init: {:?}", start_dec.elapsed());
    
    let start_decomp = std::time::Instant::now();
    // Manual multi-stream decompression loop
    let mut buffer = [0u8; BUFFER_SIZE]; 
    let mut output_buffer = [0u8; 65536];
    let mut state = BrotliState::new(StandardAlloc::default(), StandardAlloc::default(), StandardAlloc::default());
    
    let mut available_in = 0;
    let mut input_offset = 0;
    
    loop {
        // Refill buffer if needed
        if available_in == 0 {
            input_offset = 0;
            match reader.read(&mut buffer) {
                Ok(0) => break, // EOF
                Ok(n) => {
                    available_in = n;
                },
                Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
                Err(e) => return Err(e.into()),
            }
        }

        let mut available_out = output_buffer.len();
        let mut output_offset = 0;
        let mut total_out = 0; 

        let result = BrotliDecompressStream(
            &mut available_in,
            &mut input_offset,
            &buffer,
            &mut available_out,
            &mut output_offset,
            &mut output_buffer,
            &mut total_out,
            &mut state,
        );
        
        // Write output
        if output_offset > 0 {
            output_writer.write_all(&output_buffer[..output_offset])?;
        }

        match result {
            BrotliResult::ResultSuccess => {
                // Determine if we are truly done or just one stream done.
                // Reset state for potential next stream.
                state = BrotliState::new(StandardAlloc::default(), StandardAlloc::default(), StandardAlloc::default());
            },
            BrotliResult::NeedsMoreInput => {
                // Loop will refill. If EOF was hit (reader returned 0), we break outer loop?
                // Wait, if available_in == 0 after read returns 0, loop breaks.
                // So if NeedsMoreInput and available_in is 0, we are done (or error if incomplete stream).
                // But loop condition handles EOF.
            },
            BrotliResult::NeedsMoreOutput => {
                // Loop will continue with same input (available_in > 0)
            },
            BrotliResult::ResultFailure => {
                return Err("Decompression failed".into());
            }
        }
    }
    
    output_writer.flush()?;
    println!("Decompression (Streaming): {:?}", start_decomp.elapsed());
    println!("Total Decrypt Time: {:?}", start_total.elapsed());

    Ok(())
}
