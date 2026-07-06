use super::TrainingFile;
use crate::error::Error;
use crate::pack::{Difficulty, Guid, Round, TrainingPack, TrainingType};

fn sample_pack() -> TrainingPack {
    TrainingPack {
        guid: Guid {
            a: 0x0102_0304,
            b: 0x1112_1314,
            c: 0x2122_2324,
            d: 0x3132_3334,
        },
        code: Some("ABCD-1234-EF56-7890".to_string()),
        name: Some("Synthetic Pack".to_string()),
        training_type: TrainingType::Striker,
        difficulty: Difficulty::Medium,
        creator_name: Some("subtr-actor".to_string()),
        description: Some("Round-trip fixture".to_string()),
        tags: vec![3, 17],
        map_name: Some("Park_P".to_string()),
        created_at: 1_700_000_000,
        updated_at: 1_700_000_100,
        rounds: vec![
            Round {
                time_limit: 8.0,
                serialized_archetypes: vec![
                    "{\"ObjectArchetype\":\"Archetypes.Ball.Ball_GameEditor\",\"StartLocationX\":0}".to_string(),
                ],
            },
            Round {
                time_limit: 0.0,
                serialized_archetypes: vec!["{\"IsPC\":true}".to_string()],
            },
        ],
        ..TrainingPack::default()
    }
}

#[test]
fn encrypted_file_roundtrip() {
    let file = TrainingFile::from_pack(&sample_pack()).unwrap();
    let bytes = file.to_bytes().unwrap();
    let reparsed = TrainingFile::from_bytes(&bytes).unwrap();
    assert_eq!(reparsed, file);
    // Deterministic serialization: encode of the reparse is identical.
    assert_eq!(reparsed.to_bytes().unwrap(), bytes);
}

#[test]
fn decrypted_payload_roundtrip_including_padding() {
    let file = TrainingFile::from_pack(&sample_pack()).unwrap();
    let payload = file.to_decrypted_payload().unwrap();
    let reparsed = TrainingFile::from_decrypted_payload(&payload).unwrap();
    assert_eq!(reparsed, file);

    // With zero padding appended (as after decryption) parsing still works.
    let mut padded = payload.clone();
    padded.resize(payload.len().div_ceil(16) * 16, 0);
    assert_eq!(TrainingFile::from_decrypted_payload(&padded).unwrap(), file);
}

#[test]
fn crc_tampering_is_detected() {
    let file = TrainingFile::from_pack(&sample_pack()).unwrap();
    let mut bytes = file.to_bytes().unwrap();
    let last = bytes.len() - 1;
    bytes[last] ^= 0xFF;
    assert!(matches!(
        TrainingFile::from_bytes(&bytes),
        Err(Error::CrcMismatch { .. })
    ));
}

#[test]
fn bad_magic_is_rejected() {
    let file = TrainingFile::from_pack(&sample_pack()).unwrap();
    let mut payload = file.to_decrypted_payload().unwrap();
    payload[0] ^= 0xFF;
    assert!(matches!(
        TrainingFile::from_decrypted_payload(&payload),
        Err(Error::BadMagic { .. })
    ));
}

#[test]
fn trailing_garbage_is_rejected() {
    let file = TrainingFile::from_pack(&sample_pack()).unwrap();
    let mut payload = file.to_decrypted_payload().unwrap();
    payload.push(0x5A);
    assert!(matches!(
        TrainingFile::from_decrypted_payload(&payload),
        Err(Error::TrailingGarbage(_))
    ));
}

#[test]
fn json_roundtrip_is_lossless() {
    let file = TrainingFile::from_pack(&sample_pack()).unwrap();
    let json = file.to_json(true).unwrap();
    let back = TrainingFile::from_json(&json).unwrap();
    assert_eq!(back, file);
    assert_eq!(
        back.to_decrypted_payload().unwrap(),
        file.to_decrypted_payload().unwrap()
    );
}

#[test]
fn object_table_positions_are_validated() {
    let file = TrainingFile::from_pack(&sample_pack()).unwrap();
    let mut payload = file.to_decrypted_payload().unwrap();
    // Corrupt the object table's file position (last 8 bytes are position +
    // object index).
    let position_offset = payload.len() - 8;
    payload[position_offset] ^= 0x01;
    assert!(matches!(
        TrainingFile::from_decrypted_payload(&payload),
        Err(Error::NonContiguousObjects { .. })
    ));
}
