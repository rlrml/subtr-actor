use aes::Aes256;
use aes::cipher::generic_array::GenericArray;
use aes::cipher::{BlockEncrypt, KeyInit};

use super::{SAVE_DATA_AES_KEY, decrypt, encrypt};
use crate::error::Error;

#[test]
fn fips_197_aes256_vector() {
    // FIPS-197 appendix C.3: AES-256 of 00112233..ff under key 000102..1f.
    let key: Vec<u8> = (0..32u8).collect();
    let plaintext: [u8; 16] = [
        0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xAA, 0xBB, 0xCC, 0xDD, 0xEE,
        0xFF,
    ];
    let expected: [u8; 16] = [
        0x8E, 0xA2, 0xB7, 0xCA, 0x51, 0x67, 0x45, 0xBF, 0xEA, 0xFC, 0x49, 0x90, 0x4B, 0x49, 0x60,
        0x89,
    ];
    let cipher = Aes256::new(GenericArray::from_slice(&key));
    let mut block = GenericArray::from(plaintext);
    cipher.encrypt_block(&mut block);
    assert_eq!(block.as_slice(), &expected);
}

#[test]
fn roundtrip_identity_with_padding() {
    // 20 bytes: padded to 32, and decrypt recovers data + zero padding.
    let data: Vec<u8> = (1..=20u8).collect();
    let ciphertext = encrypt(&data);
    assert_eq!(ciphertext.len(), 32, "padded length is the stored length");
    let decrypted = decrypt(&ciphertext).unwrap();
    assert_eq!(&decrypted[..20], data.as_slice());
    assert!(decrypted[20..].iter().all(|&byte| byte == 0));
}

#[test]
fn block_aligned_input_gets_no_padding() {
    let data = vec![0xABu8; 48];
    let ciphertext = encrypt(&data);
    assert_eq!(ciphertext.len(), 48);
    assert_eq!(decrypt(&ciphertext).unwrap(), data);
}

#[test]
fn empty_input() {
    let ciphertext = encrypt(&[]);
    assert!(ciphertext.is_empty());
    assert!(decrypt(&ciphertext).unwrap().is_empty());
}

#[test]
fn ecb_mode_encrypts_identical_blocks_identically() {
    let data = vec![0x42u8; 32];
    let ciphertext = encrypt(&data);
    assert_eq!(ciphertext[..16], ciphertext[16..32]);
}

#[test]
fn unaligned_ciphertext_is_rejected() {
    assert!(matches!(
        decrypt(&[0u8; 17]),
        Err(Error::EncryptedLengthNotBlockAligned(17))
    ));
}

#[test]
fn hardcoded_key_is_used() {
    // Encrypting one zero block with the save data key must differ from a
    // zero-key encryption of the same block.
    let with_save_key = encrypt(&[0u8; 16]);
    let zero_key_cipher = Aes256::new(GenericArray::from_slice(&[0u8; 32]));
    let mut block = GenericArray::from([0u8; 16]);
    zero_key_cipher.encrypt_block(&mut block);
    assert_ne!(with_save_key.as_slice(), block.as_slice());
    assert_eq!(SAVE_DATA_AES_KEY[0], 0xD7);
}
