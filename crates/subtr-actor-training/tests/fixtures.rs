//! Fixture tests: a committed synthetic `.tem` exercised in CI, plus
//! byte-fidelity round-trips over any real save data files supplied via the
//! `SUBTR_ACTOR_TEM_FIXTURE_DIR` environment variable (real `.tem`/`.save`
//! files are personal data and are not committed to the repository; those
//! tests skip silently when the variable is unset).

use subtr_actor_training::{Difficulty, TrainingFile, TrainingType, crypto};

const SYNTHETIC: &[u8] = include_bytes!("../assets/synthetic-pack.tem");

#[test]
fn synthetic_fixture_decodes() {
    let file = TrainingFile::from_bytes(SYNTHETIC).unwrap();
    let pack = file.pack().unwrap();
    assert_eq!(pack.name.as_deref(), Some("Synthetic Fixture Päck"));
    assert_eq!(pack.code.as_deref(), Some("AAAA-BBBB-CCCC-DDDD"));
    assert_eq!(pack.creator_name.as_deref(), Some("subtr-actor"));
    assert_eq!(pack.description.as_deref(), Some("Тест 🚀 fixture"));
    assert_eq!(pack.training_type, TrainingType::Striker);
    assert_eq!(pack.difficulty, Difficulty::Medium);
    assert_eq!(pack.map_name.as_deref(), Some("Park_P"));
    assert_eq!(pack.rounds.len(), 3);
    assert_eq!(pack.rounds[1].time_limit, 12.5);
    assert!(pack.rounds[0].serialized_archetypes[0].contains("ObjectArchetype"));
}

#[test]
fn synthetic_fixture_roundtrips_byte_for_byte() {
    let file = TrainingFile::from_bytes(SYNTHETIC).unwrap();
    assert_eq!(file.to_bytes().unwrap(), SYNTHETIC);
}

/// Byte-fidelity over real files when a fixture directory is provided.
#[test]
fn real_fixtures_roundtrip_byte_for_byte() {
    let Ok(dir) = std::env::var("SUBTR_ACTOR_TEM_FIXTURE_DIR") else {
        eprintln!("SUBTR_ACTOR_TEM_FIXTURE_DIR unset; skipping");
        return;
    };
    let mut checked = 0;
    for entry in std::fs::read_dir(&dir).unwrap() {
        let path = entry.unwrap().path();
        let extension = path
            .extension()
            .and_then(|extension| extension.to_str())
            .map(str::to_ascii_lowercase);
        if !matches!(extension.as_deref(), Some("tem") | Some("save")) {
            continue;
        }
        let original = std::fs::read(&path).unwrap();
        let file = TrainingFile::from_bytes(&original)
            .unwrap_or_else(|error| panic!("parsing {path:?}: {error}"));

        // Decrypted payload fidelity (including AES zero padding).
        let decrypted = crypto::decrypt(&original[8..]).unwrap();
        let mut payload = file.to_decrypted_payload().unwrap();
        payload.resize(decrypted.len(), 0);
        assert_eq!(payload, decrypted, "decrypted payload of {path:?}");

        // Full-file fidelity (envelope, CRC, ciphertext).
        assert_eq!(file.to_bytes().unwrap(), original, "full file {path:?}");

        // Lossless JSON round trip on real data.
        let json = file.to_json(false).unwrap();
        assert_eq!(TrainingFile::from_json(&json).unwrap(), file);

        checked += 1;
    }
    assert!(checked > 0, "no .tem/.save files found in {dir}");
}
