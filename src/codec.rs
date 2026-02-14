use std::fs::File;
use std::io::{self, Read, Write};
use std::path::Path;
use age::secrecy::Secret;
use age::secrecy::ExposeSecret;

pub fn encrypt_stream(
    input_path: &Path,
    output_path: &Path,
    password: Secret<String>,
    quality: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    const BUFFER_SIZE: usize = 65536;
    let mut input_file = File::open(input_path)?;
    let output_file = File::create(output_path)?;

    // Wrap output file in a BufWriter for better performance
    let output_writer = io::BufWriter::with_capacity(BUFFER_SIZE, output_file);

    let encryptor = age::Encryptor::with_user_passphrase(password);
    let mut armor = encryptor.wrap_output(output_writer)?;
    
    {
        let mut compressor = brotli::CompressorWriter::new(&mut armor, BUFFER_SIZE, quality, 20);
        io::copy(&mut input_file, &mut compressor)?;
    }
    
    armor.finish()?;

    Ok(())
}

pub fn decrypt_stream(
    input_path: &Path,
    output_path: &Path,
    password: Secret<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    const BUFFER_SIZE: usize = 65536;
    let input_file = File::open(input_path)?;
    // Use BufReader for input to reduce syscalls during decryption
    let input_reader = io::BufReader::with_capacity(BUFFER_SIZE, input_file);
    
    let output_file = File::create(output_path)?;

    let decryptor = match age::Decryptor::new(input_reader)? {
        age::Decryptor::Passphrase(d) => d,
        _ => return Err("Input file is not encrypted with a passphrase".into()),
    };

    let mut reader = decryptor.decrypt(&password, None)?;
    
    // Decompress
    let mut decompressor = brotli::Decompressor::new(&mut reader, BUFFER_SIZE);
    
    let mut writer = io::BufWriter::with_capacity(BUFFER_SIZE, output_file);
    io::copy(&mut decompressor, &mut writer)?;

    Ok(())
}
