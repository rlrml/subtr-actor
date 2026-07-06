//! The CRC-32 variant used by Rocket League save data.
//!
//! This is an MSB-first (non-reflected) CRC-32 with polynomial `0x04C11DB7`.
//! The seed is complemented before processing and the result is complemented
//! again at the end, mirroring `Crc32.cs` in the RocketRP reference
//! implementation. Note that this is *not* the common reflected CRC-32
//! (`crc32fast` computes the wrong value for these files).

/// Seed used for the save data envelope CRC.
pub const SAVE_DATA_CRC_SEED: u32 = 0xEFCB_F201;

const POLY: u32 = 0x04C1_1DB7;

/// Compute the Rocket League save data CRC-32 of `data` with the given seed.
pub fn crc32(data: &[u8], seed: u32) -> u32 {
    let mut crc = !seed;
    for &byte in data {
        let mut c = (((crc >> 24) as u8) ^ byte) as u32;
        c <<= 24;
        for _ in 0..8 {
            c = if c & 0x8000_0000 != 0 {
                (c << 1) ^ POLY
            } else {
                c << 1
            };
        }
        crc = (crc << 8) ^ c;
    }
    !crc
}

#[cfg(test)]
#[path = "crc32_tests.rs"]
mod tests;
