use super::{SAVE_DATA_CRC_SEED, crc32};

/// Independent bit-at-a-time reference implementation (no table, different
/// formulation from the production code's per-byte loop).
fn reference_crc32(data: &[u8], seed: u32) -> u32 {
    let mut crc = !seed;
    for &byte in data {
        crc ^= (byte as u32) << 24;
        for _ in 0..8 {
            crc = if crc & 0x8000_0000 != 0 {
                (crc << 1) ^ 0x04C1_1DB7
            } else {
                crc << 1
            };
        }
    }
    !crc
}

#[test]
fn known_answers() {
    // Computed with an independent Python implementation of the algorithm in
    // RocketRP's Crc32.cs.
    assert_eq!(crc32(b"123456789", SAVE_DATA_CRC_SEED), 0xDBFA_7673);
    let all_bytes: Vec<u8> = (0..=255u8).collect();
    assert_eq!(crc32(&all_bytes, SAVE_DATA_CRC_SEED), 0x7BB6_1B5B);
}

#[test]
fn matches_crc32_cksum_check_value() {
    // With seed 0xFFFFFFFF this algorithm reduces to CRC-32/CKSUM (init 0,
    // xorout 0xFFFFFFFF, MSB-first, poly 0x04C11DB7), whose documented check
    // value for "123456789" is 0x765E7680 — an external known answer.
    assert_eq!(crc32(b"123456789", 0xFFFF_FFFF), 0x765E_7680);
}

#[test]
fn empty_input_returns_seed() {
    // !(!seed) == seed with no bytes processed.
    assert_eq!(crc32(&[], SAVE_DATA_CRC_SEED), SAVE_DATA_CRC_SEED);
    assert_eq!(crc32(&[], 0x1234_5678), 0x1234_5678);
}

#[test]
fn matches_independent_reference() {
    // Pseudo-random coverage against the bitwise reference.
    let mut state = 0x1357_9BDFu32;
    let mut data = Vec::new();
    for length in [1usize, 2, 7, 16, 63, 256, 1000] {
        while data.len() < length {
            state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            data.push((state >> 24) as u8);
        }
        for seed in [SAVE_DATA_CRC_SEED, 0, 0xFFFF_FFFF, state] {
            assert_eq!(
                crc32(&data[..length], seed),
                reference_crc32(&data[..length], seed),
                "length {length}, seed {seed:#x}"
            );
        }
    }
}
