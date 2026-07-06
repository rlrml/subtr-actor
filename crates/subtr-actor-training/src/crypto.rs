//! AES-256-ECB encryption of the save data envelope.
//!
//! Rocket League encrypts the decrypted payload with a hardcoded AES-256 key
//! in ECB mode (block by block, no chaining, no IV). "Padding" is zero fill
//! up to the next 16-byte boundary, and the *padded* length is what gets
//! stored in the envelope, so decryption never has to strip padding.

use aes::Aes256;
use aes::cipher::generic_array::GenericArray;
use aes::cipher::{BlockDecrypt, BlockEncrypt, KeyInit};

use crate::error::{Error, Result};

/// The hardcoded save data AES-256 key (from RocketRP's `AES.cs`).
pub const SAVE_DATA_AES_KEY: [u8; 32] = [
    0xD7, 0x8C, 0x32, 0x4A, 0x94, 0x42, 0x94, 0x3C, 0x6D, 0x65, 0xCE, 0x98, 0x81, 0x85, 0x4C, 0x41,
    0x68, 0x99, 0x22, 0x0C, 0xC7, 0xA1, 0x46, 0x40, 0x93, 0x9B, 0x96, 0x3C, 0x93, 0x2A, 0x6F, 0xAF,
];

const BLOCK_SIZE: usize = 16;

/// Encrypt `data` with the save data key, zero-padding to a block boundary.
///
/// The returned ciphertext length is `data.len()` rounded up to a multiple
/// of 16; that padded length is what the envelope stores.
pub fn encrypt(data: &[u8]) -> Vec<u8> {
    let padded_len = data.len().div_ceil(BLOCK_SIZE) * BLOCK_SIZE;
    let mut out = vec![0u8; padded_len];
    out[..data.len()].copy_from_slice(data);
    let cipher = Aes256::new(GenericArray::from_slice(&SAVE_DATA_AES_KEY));
    for block in out.chunks_exact_mut(BLOCK_SIZE) {
        cipher.encrypt_block(GenericArray::from_mut_slice(block));
    }
    out
}

/// Decrypt `data` (which must be a whole number of AES blocks) in ECB mode.
pub fn decrypt(data: &[u8]) -> Result<Vec<u8>> {
    if !data.len().is_multiple_of(BLOCK_SIZE) {
        return Err(Error::EncryptedLengthNotBlockAligned(data.len()));
    }
    let mut out = data.to_vec();
    let cipher = Aes256::new(GenericArray::from_slice(&SAVE_DATA_AES_KEY));
    for block in out.chunks_exact_mut(BLOCK_SIZE) {
        cipher.decrypt_block(GenericArray::from_mut_slice(block));
    }
    Ok(out)
}

#[cfg(test)]
#[path = "crypto_tests.rs"]
mod tests;
