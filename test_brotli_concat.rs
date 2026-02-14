use std::io::{Read, Write};

fn main() {
    let data1 = b"Hello ";
    let data2 = b"World!";
    
    let mut compressed1 = Vec::new();
    {
        let mut compressor = brotli::CompressorWriter::new(&mut compressed1, 4096, 6, 20);
        compressor.write_all(data1).unwrap();
    }
    
    let mut compressed2 = Vec::new();
    {
        let mut compressor = brotli::CompressorWriter::new(&mut compressed2, 4096, 6, 20);
        compressor.write_all(data2).unwrap();
    }
    
    // Concatenate
    let mut combined = compressed1;
    combined.extend(compressed2);
    
    // Decompress
    let mut decompressed = Vec::new();
    let mut decompressor = brotli::Decompressor::new(&combined[..], 4096);
    decompressor.read_to_end(&mut decompressed).unwrap();
    
    let result = String::from_utf8(decompressed).unwrap();
    println!("Result: {}", result);
    assert_eq!(result, "Hello World!");
}
