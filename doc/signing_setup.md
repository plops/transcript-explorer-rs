# Package Signing Setup

This document describes how package signing is configured for the transcript-explorer project using Zipsign with ed25519 keys.

## Overview

All release packages are cryptographically signed using Zipsign (ed25519-based). The signature is embedded directly in the archive, and the binary verifies it using the embedded public key before installation.

## Files

- `zipsign.pub` - Public key (committed to repo, embedded in binary)
- `secrets/zipsign.priv` - Private key (gitignored, stored in GitHub Secrets)
- `secrets/README.md` - Setup instructions

## GitHub Secrets Configuration

The following secrets must be added to your GitHub repository:

### ZIPSIGN_PRIV_KEY

Base64-encoded private key. This is used by CI to sign packages during release.

**Current value (Base64):**
```
KynEV/DrEfBtaGiY7sTb3vPUz2TOXUSw
crv1JyMUeZ8RcFGVUsDpmFS04VLpaO3owoaTiQE6iSA/d12q4XLbeQ==
```

**To add to GitHub:**
1. Go to your GitHub Repo → Settings → Secrets and variables → Actions
2. Click "New repository secret"
3. Name: `ZIPSIGN_PRIV_KEY`
4. Value: Paste the Base64 string above
5. Click "Add secret"

### ZIPSIGN_PASSWORD

The password used when generating the keypair (if any). If no password was used, this can be left empty or set to an empty string.

**To add to GitHub:**
1. Go to your GitHub Repo → Settings → Secrets and variables → Actions
2. Click "New repository secret"
3. Name: `ZIPSIGN_PASSWORD`
4. Value: (leave empty if no password was used)
5. Click "Add secret"

## CI/CD Integration

The `.github/workflows/release.yml` workflow:

1. Builds the binary for each platform
2. Creates archives (.tar.gz for Unix, .zip for Windows)
3. Signs each archive using the private key from GitHub Secrets
4. Uploads signed archives to the GitHub release

## Verification in the Application

The `BinaryVerifier::verify_binary()` function in `src/update/mod.rs`:

1. Checks file existence and readability
2. Verifies file size matches expected size
3. **Verifies the signature using the embedded public key**
4. Deletes corrupted files automatically

The public key is embedded at compile time using:
```rust
let public_key_bytes: [u8; 32] = *include_bytes!("../../zipsign.pub");
let public_key = VerifyingKey::from_bytes(&public_key_bytes)?;
zipsign_api::verify::verify_tar(&mut reader, &[public_key], None)?;
// or
zipsign_api::verify::verify_zip(&mut reader, &[public_key], None)?;
```

## Security Benefits

- **Private key never leaves GitHub Secrets** - Only CI has access
- **Signature embedded in archive** - No separate .sig files needed
- **Public key embedded in binary** - No external key distribution
- **Prevents GitHub account compromise** - Even if GitHub account is hacked, attacker cannot sign malware without the private key
- **Standard HTTPS + signature verification** - No need for certificate pinning
- **Ed25519 cryptography** - Modern, fast, and secure elliptic curve signing

## Regenerating Keys

If you need to regenerate the keypair:

```bash
# Generate new keys
zipsign gen-key secrets/zipsign.priv secrets/zipsign.pub

# Copy public key to repo root
cp secrets/zipsign.pub zipsign.pub

# Get new Base64 encoded private key
base64 -w0 secrets/zipsign.priv && echo ""

# Update GitHub Secrets with new ZIPSIGN_PRIV_KEY value
```

**Important:** After regenerating keys, you must:
1. Update the `ZIPSIGN_PRIV_KEY` secret in GitHub
2. Commit the new `zipsign.pub` to the repository
3. Rebuild and release a new version with the new public key embedded
