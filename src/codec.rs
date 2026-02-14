use std::fs::File;
use std::io::{self, Write};
use std::path::Path;
use age::secrecy::{Secret, ExposeSecret};
use brotli::enc::BrotliEncoderParams;

pub fn encrypt_stream(
    input_path: &Path,
    output_path: &Path,
    password: Secret<String>,
    quality: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    let start_total = std::time::Instant::now();
    
    let input_file = File::open(input_path)?;
    let mut input_reader = io::BufReader::new(input_file);
    
    let output_file = File::create(output_path)?;
    let output_writer = io::BufWriter::with_capacity(65536, output_file);
    
    let start_enc = std::time::Instant::now();
    let encryptor = age::Encryptor::with_user_passphrase(Secret::new(password.expose_secret().clone()));
    let mut age_writer = encryptor.wrap_output(output_writer)?;
    println!("Encryption Header (Scrypt): {:?}", start_enc.elapsed());

    let _start_comp = std::time::Instant::now();
    
    let mut params = BrotliEncoderParams::default();
    params.quality = quality as i32;
    
    {
        let mut compressor = brotli::CompressorWriter::with_params(&mut age_writer, 65536, &params);
        io::copy(&mut input_reader, &mut compressor)?;
        compressor.flush()?;
    } // Compressor dropped here
    
    age_writer.finish()?;
    
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

    let max_work_factor = if cfg!(debug_assertions) { Some(22) } else { None };
    let reader = decryptor.decrypt(&password, max_work_factor)?;
    println!("Decryption Init: {:?}", start_dec.elapsed());
    
    let _start_decomp = std::time::Instant::now();
    
    // Use brotli::Decompressor which handles concatenated streams automatically
    let mut decompressor = brotli::Decompressor::new(reader, BUFFER_SIZE);
    io::copy(&mut decompressor, &mut output_writer)?;
    
    output_writer.flush()?;
    println!("Total Decrypt Time: {:?}", start_total.elapsed());

    Ok(())
}
