use super::{Difficulty, Guid, PlayerId, Round, TrainingPack, TrainingType};
use crate::container::TrainingFile;
use crate::property::{PropertyValue, StructBody, StructValue};

fn round(time_limit: f32, archetype: &str) -> Round {
    Round {
        time_limit,
        serialized_archetypes: vec![archetype.to_string()],
    }
}

fn sample_pack() -> TrainingPack {
    TrainingPack {
        guid: Guid {
            a: 1,
            b: 2,
            c: 3,
            d: 4,
        },
        code: Some("1111-2222-3333-4444".to_string()),
        name: Some("Pack name".to_string()),
        training_type: TrainingType::Aerial,
        difficulty: Difficulty::Hard,
        creator_name: Some("creator".to_string()),
        description: Some("description".to_string()),
        tags: vec![1, 2, 3],
        map_name: Some("cs_p".to_string()),
        created_at: 111,
        updated_at: 222,
        creator_player_id: PlayerId {
            uid: 76_561_198_027_786_501,
            epic_account_id: None,
            platform: Some("OnlinePlatform_Steam".to_string()),
            splitscreen_id: 0,
        },
        rounds: vec![round(10.0, "{\"a\":1}"), round(0.0, "{\"b\":2}")],
        player_team_number: 1,
        unowned: true,
        perfect_completed: false,
        shots_completed: 5,
    }
}

#[test]
fn pack_view_roundtrips_through_the_tree() {
    let pack = sample_pack();
    let file = TrainingFile::from_pack(&pack).unwrap();
    assert_eq!(file.pack().unwrap(), pack);
}

#[test]
fn pack_serde_json_roundtrip() {
    let pack = sample_pack();
    let json = serde_json::to_string_pretty(&pack).unwrap();
    let back: TrainingPack = serde_json::from_str(&json).unwrap();
    assert_eq!(back, pack);
    // Enums serialize as their in-file names.
    assert!(json.contains("\"Training_Aerial\""));
    assert!(json.contains("\"D_Hard\""));
}

#[test]
fn enum_names_roundtrip_including_unknown_values() {
    for (training_type, name) in [
        (TrainingType::None, "Training_None"),
        (TrainingType::Aerial, "Training_Aerial"),
        (TrainingType::Goalie, "Training_Goalie"),
        (TrainingType::Striker, "Training_Striker"),
        (TrainingType::End, "Training_END"),
        (
            TrainingType::Other("Training_Future".to_string()),
            "Training_Future",
        ),
    ] {
        assert_eq!(training_type.as_name(), name);
        assert_eq!(TrainingType::from_name(name), training_type);
    }
    for (difficulty, name) in [
        (Difficulty::Easy, "D_Easy"),
        (Difficulty::Medium, "D_Medium"),
        (Difficulty::Hard, "D_Hard"),
        (Difficulty::End, "D_END"),
        (Difficulty::Other("D_Custom".to_string()), "D_Custom"),
    ] {
        assert_eq!(difficulty.as_name(), name);
        assert_eq!(Difficulty::from_name(name), difficulty);
    }
}

#[test]
fn scalar_setters_write_into_the_tree() {
    let mut file = TrainingFile::from_pack(&sample_pack()).unwrap();
    file.set_name(Some("Renamed")).unwrap();
    file.set_code(Some("9999-8888-7777-6666")).unwrap();
    file.set_description(None).unwrap();
    file.set_creator_name(Some("new creator")).unwrap();
    file.set_difficulty(&Difficulty::Easy).unwrap();
    file.set_training_type(&TrainingType::Goalie).unwrap();
    file.set_map_name("Park_P").unwrap();
    file.set_tags(vec![9]).unwrap();
    file.set_guid(Guid {
        a: 9,
        b: 8,
        c: 7,
        d: 6,
    })
    .unwrap();
    file.set_created_at(1).unwrap();
    file.set_updated_at(2).unwrap();
    file.set_player_team_number(0);
    file.set_unowned(false);
    file.set_perfect_completed(true);
    file.set_shots_completed(10);

    let pack = file.pack().unwrap();
    assert_eq!(pack.name.as_deref(), Some("Renamed"));
    assert_eq!(pack.code.as_deref(), Some("9999-8888-7777-6666"));
    assert_eq!(pack.description, None);
    assert_eq!(pack.creator_name.as_deref(), Some("new creator"));
    assert_eq!(pack.difficulty, Difficulty::Easy);
    assert_eq!(pack.training_type, TrainingType::Goalie);
    assert_eq!(pack.map_name.as_deref(), Some("Park_P"));
    assert_eq!(pack.tags, vec![9]);
    assert_eq!(pack.guid.a, 9);
    assert_eq!((pack.created_at, pack.updated_at), (1, 2));
    assert_eq!(pack.player_team_number, 0);
    assert!(!pack.unowned);
    assert!(pack.perfect_completed);
    assert_eq!(pack.shots_completed, 10);

    // Edits keep the file encodable and stable.
    let bytes = file.to_bytes().unwrap();
    assert_eq!(
        TrainingFile::from_bytes(&bytes).unwrap().pack().unwrap(),
        pack
    );
}

#[test]
fn round_editing_operations() {
    let mut file = TrainingFile::from_pack(&sample_pack()).unwrap();

    file.add_round(&round(3.0, "{\"c\":3}")).unwrap();
    assert_eq!(file.rounds().unwrap().len(), 3);

    file.insert_round(0, &round(1.0, "{\"first\":true}"))
        .unwrap();
    assert_eq!(file.rounds().unwrap()[0].time_limit, 1.0);

    file.duplicate_round(0).unwrap();
    let rounds = file.rounds().unwrap();
    assert_eq!(rounds.len(), 5);
    assert_eq!(rounds[0], rounds[1]);

    file.move_round(0, 4).unwrap();
    assert_eq!(file.rounds().unwrap()[4].time_limit, 1.0);

    let removed = file.remove_round(4).unwrap();
    assert_eq!(removed.time_limit, 1.0);
    assert_eq!(file.rounds().unwrap().len(), 4);

    assert!(file.remove_round(99).is_err());
    assert!(file.move_round(0, 99).is_err());
}

#[test]
fn append_rounds_from_another_pack() {
    let mut destination = TrainingFile::from_pack(&sample_pack()).unwrap();
    let source = TrainingFile::from_pack(&TrainingPack {
        rounds: vec![round(7.0, "{\"src\":1}"), round(8.0, "{\"src\":2}")],
        ..TrainingPack::default()
    })
    .unwrap();

    let appended = destination.append_rounds_from(&source).unwrap();
    assert_eq!(appended, 2);
    let rounds = destination.rounds().unwrap();
    assert_eq!(rounds.len(), 4);
    assert_eq!(rounds[3].serialized_archetypes, vec!["{\"src\":2}"]);
}

#[test]
fn rounds_can_be_added_when_absent() {
    let mut file = TrainingFile::from_pack(&TrainingPack::default()).unwrap();
    assert!(file.rounds().unwrap().is_empty());
    file.add_round(&round(4.0, "{}")).unwrap();
    assert_eq!(file.rounds().unwrap().len(), 1);
}

#[test]
fn unknown_properties_survive_typed_edits() {
    let mut file = TrainingFile::from_pack(&sample_pack()).unwrap();
    // Simulate a future-game property between known fields.
    let data = file.training_data_mut().unwrap();
    data.set("FutureField", PropertyValue::Int(1234));
    data.set(
        "FutureStruct",
        PropertyValue::Struct(StructValue {
            struct_type: crate::io::UeString::new("Mystery"),
            body: StructBody::Raw(vec![1, 2, 3]),
        }),
    );

    file.set_name(Some("edited")).unwrap();
    file.add_round(&round(2.0, "{}")).unwrap();

    let bytes = file.to_bytes().unwrap();
    let reparsed = TrainingFile::from_bytes(&bytes).unwrap();
    let data = reparsed.training_data().unwrap();
    assert_eq!(
        data.get("FutureField").map(|property| &property.value),
        Some(&PropertyValue::Int(1234))
    );
    assert!(data.get("FutureStruct").is_some());
    assert_eq!(reparsed.pack().unwrap().name.as_deref(), Some("edited"));
}
