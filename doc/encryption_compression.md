# Encryption and Compression

Transcript Explorer supports securing your database files using `age` encryption and reducing their size using `brotli` compression.

## Features

- **Compression**: Uses `brotli` to significantly reduce the size of the SQLite database (typically ~50% reduction).
- **Encryption**: Uses `age` (Actually Good Encryption) to secure the database with a passphrase.
- **Transparent Loading**: The application can directly load encrypted/compressed files by decrypting them to a temporary file in the background.

## Usage

### Encrypting a Database

To compress and encrypt an existing SQLite database:

```bash
transcript-explorer encrypt -i data/summaries.db -o data/summaries.db.age
```

**Optimization Options:**
- `--fast`: Use faster compression (Brotli quality 1). Larger file size, but much quicker.
  ```bash
  transcript-explorer encrypt --fast --input data/summaries.db --output data/summaries.db.age
  ```
- `--best`: Use best compression (Brotli quality 11). Smallest file size, but very slow.

You will be prompted to enter a passphrase.

### Decrypting a Database

To decrypt and decompress a file back to a standard SQLite database:

```bash
transcript-explorer decrypt -i data/summaries.db.age -o data/summaries_restored.db
```

You will be prompted for the passphrase.

### Running with an Encrypted Database

You can run the application directly against an encrypted file. The application detects encryption automatically (by checking for the `age-encryption.org` header) or you can just pass the file path.

```bash
transcript-explorer run --db data/summaries.db.age
# OR simply
transcript-explorer --db data/summaries.db.age
```

1. The application will prompt for the passphrase.
2. It decrypts the database to a secure temporary file.
3. The TUI launches using the temporary database.
4. When you quit the application, the temporary file is automatically deleted.

> [!NOTE]
> The temporary file is creating using `tempfile`, which ensures it is removed even if the application crashes (OS dependent, but generally reliable). It is created with restricted permissions (0600) on Unix systems.

## Technical Details

- **Streamed Processing**: Encryption and decryption use a single-stream approach for maximum reliability and lower memory overhead.
- **Robustness**: Uses a single Brotli stream per file to avoid data loss issues sometimes associated with multi-stream concatenation in custom decompressors.
- **Critical Build Configuration**: Encryption performance relies heavily on compiler optimizations. The `release` profile must use `opt-level = 3` (speed) rather than `z` (size) to achieve >1GB/s throughput.
- **Performance Metrics**: The CLI outputs detailed timing logs to stdout, measuring:
    - Input read time
    - Compression & Encryption time
    - Total elapsed time
- **Algorithm**: 
    - **Compression**: Brotli (Default Quality 6, Window 20). Configurable via `--fast` (Quality 1) or `--best` (Quality 11).
    - **Encryption**: Age (Passphrase-based, Scrypt work factor 18)
