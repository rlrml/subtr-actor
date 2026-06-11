mod common;

use subtr_actor::{ReplayGameType, ReplayProcessor};

struct GameTypeFixture {
    path: &'static str,
    game_type: ReplayGameType,
    playlist_id: Option<i32>,
    match_type_class: Option<&'static str>,
}

const GAME_TYPE_FIXTURES: &[GameTypeFixture] = &[
    GameTypeFixture {
        path: "assets/air-dribble-goal-mouth-2026-05-24.replay",
        game_type: ReplayGameType::Ranked,
        playlist_id: Some(11),
        match_type_class: Some("TAGame.MatchType_PublicRanked_TA"),
    },
    GameTypeFixture {
        path: "assets/ballchasing-d0a7cdd4-5c9d-42b2-81aa-24ef85da3f8a.replay",
        game_type: ReplayGameType::Private,
        playlist_id: Some(6),
        match_type_class: Some("TAGame.MatchType_Private_TA"),
    },
    GameTypeFixture {
        path: "assets/colonelpanic8-double-tap-third-goal-2026-05-24.replay",
        game_type: ReplayGameType::Offline,
        playlist_id: Some(8),
        match_type_class: Some("TAGame.MatchType_Offline_TA"),
    },
    GameTypeFixture {
        path: "assets/nuttrback-double-tap-goal-7-2026-06-01.replay",
        game_type: ReplayGameType::Ranked,
        playlist_id: Some(11),
        match_type_class: Some("TAGame.MatchType_PublicRanked_TA"),
    },
    GameTypeFixture {
        path: "assets/old-ballchasing-midfield-car.replay",
        game_type: ReplayGameType::Casual,
        playlist_id: Some(3),
        match_type_class: None,
    },
    GameTypeFixture {
        path: "assets/post-eac-private-2026-04-28.replay",
        game_type: ReplayGameType::Private,
        playlist_id: Some(6),
        match_type_class: Some("TAGame.MatchType_Private_TA"),
    },
    GameTypeFixture {
        path: "assets/post-eac-ranked-doubles-2026-04-28.replay",
        game_type: ReplayGameType::Ranked,
        playlist_id: Some(11),
        match_type_class: Some("TAGame.MatchType_PublicRanked_TA"),
    },
    GameTypeFixture {
        path: "assets/post-eac-ranked-duel-2026-04-28-a.replay",
        game_type: ReplayGameType::Ranked,
        playlist_id: Some(10),
        match_type_class: Some("TAGame.MatchType_PublicRanked_TA"),
    },
    GameTypeFixture {
        path: "assets/post-eac-ranked-duel-2026-04-28-b.replay",
        game_type: ReplayGameType::Ranked,
        playlist_id: Some(10),
        match_type_class: Some("TAGame.MatchType_PublicRanked_TA"),
    },
    GameTypeFixture {
        path: "assets/post-eac-ranked-standard-2026-04-28.replay",
        game_type: ReplayGameType::Ranked,
        playlist_id: Some(13),
        match_type_class: Some("TAGame.MatchType_PublicRanked_TA"),
    },
    GameTypeFixture {
        path: "assets/problematic-private-duel-2026-03-20.replay",
        game_type: ReplayGameType::Private,
        playlist_id: Some(6),
        match_type_class: Some("TAGame.MatchType_Private_TA"),
    },
    GameTypeFixture {
        path: "assets/recent-ranked-doubles-2026-03-10.replay",
        game_type: ReplayGameType::Ranked,
        playlist_id: Some(11),
        match_type_class: Some("TAGame.MatchType_PublicRanked_TA"),
    },
    GameTypeFixture {
        path: "assets/recent-ranked-standard-2026-03-10-a.replay",
        game_type: ReplayGameType::Ranked,
        playlist_id: Some(13),
        match_type_class: Some("TAGame.MatchType_PublicRanked_TA"),
    },
    GameTypeFixture {
        path: "assets/recent-ranked-standard-2026-03-10-b.replay",
        game_type: ReplayGameType::Ranked,
        playlist_id: Some(13),
        match_type_class: Some("TAGame.MatchType_PublicRanked_TA"),
    },
    GameTypeFixture {
        path: "assets/replay-format-2016-07-21-v868-12-net-none-lan.replay",
        game_type: ReplayGameType::Lan,
        playlist_id: Some(6),
        match_type_class: None,
    },
    GameTypeFixture {
        path: "assets/replay-format-2016-11-09-v868-14-net-none-rlcs-lan.replay",
        game_type: ReplayGameType::Lan,
        playlist_id: Some(6),
        match_type_class: Some("TAGame.MatchType_Lan_TA"),
    },
    GameTypeFixture {
        path: "assets/replay-format-2017-03-16-v868-17-net-none-online.replay",
        game_type: ReplayGameType::Casual,
        playlist_id: Some(23),
        match_type_class: Some("TAGame.MatchType_Public_TA"),
    },
    GameTypeFixture {
        path: "assets/replay-format-2017-11-22-v868-20-net2-legacy-vectors.replay",
        game_type: ReplayGameType::Casual,
        playlist_id: Some(3),
        match_type_class: Some("TAGame.MatchType_Public_TA"),
    },
    GameTypeFixture {
        path: "assets/replay-format-2018-03-15-v868-20-net5-modern-vectors-legacy-rotation.replay",
        game_type: ReplayGameType::Tournament,
        playlist_id: Some(22),
        match_type_class: Some("TAGame.MatchType_Tournament_TA"),
    },
    GameTypeFixture {
        path: "assets/replay-format-2018-05-17-v868-22-net7-modern-rigidbody.replay",
        game_type: ReplayGameType::Casual,
        playlist_id: Some(3),
        match_type_class: Some("TAGame.MatchType_Public_TA"),
    },
    GameTypeFixture {
        path: "assets/replay-format-2019-04-19-v868-24-net10-modern-rigidbody.replay",
        game_type: ReplayGameType::Private,
        playlist_id: Some(6),
        match_type_class: Some("TAGame.MatchType_Private_TA"),
    },
    GameTypeFixture {
        path: "assets/replay-format-2020-09-25-v868-29-net10-tournament.replay",
        game_type: ReplayGameType::Tournament,
        playlist_id: Some(34),
        match_type_class: Some("TAGame.MatchType_AutoTournament_TA"),
    },
    GameTypeFixture {
        path: "assets/replay-format-2022-09-29-v868-32-net10-legacy-boost.replay",
        game_type: ReplayGameType::Private,
        playlist_id: Some(6),
        match_type_class: Some("TAGame.MatchType_Private_TA"),
    },
    GameTypeFixture {
        path: "assets/replay-format-2025-06-10-v868-32-net10-replicated-boost.replay",
        game_type: ReplayGameType::Ranked,
        playlist_id: Some(11),
        match_type_class: Some("TAGame.MatchType_PublicRanked_TA"),
    },
    GameTypeFixture {
        path: "assets/replay-format-2026-01-14-v868-32-net10-demolish-extended.replay",
        game_type: ReplayGameType::Private,
        playlist_id: Some(6),
        match_type_class: Some("TAGame.MatchType_Private_TA"),
    },
    GameTypeFixture {
        path: "assets/replay-format-2026-03-03-v868-32-net11-dodge-refresh-counter.replay",
        game_type: ReplayGameType::Ranked,
        playlist_id: Some(11),
        match_type_class: Some("TAGame.MatchType_PublicRanked_TA"),
    },
    GameTypeFixture {
        path: "assets/rlcs-2025-worlds-grand-final-flcn-nrg-g5.replay",
        game_type: ReplayGameType::Private,
        playlist_id: Some(6),
        match_type_class: Some("TAGame.MatchType_Private_TA"),
    },
];

#[test]
fn replay_game_type_metadata_is_consistent_across_fixture_corpus() {
    for fixture in GAME_TYPE_FIXTURES {
        let replay = common::parse_replay(fixture.path);
        let mut processor = ReplayProcessor::new(&replay).unwrap_or_else(|error| {
            panic!("failed to build processor for {}: {error:?}", fixture.path)
        });
        let replay_meta = processor
            .process_and_get_replay_meta()
            .unwrap_or_else(|error| {
                panic!(
                    "failed to build replay meta for {}: {error:?}",
                    fixture.path
                )
            });
        let game_type = replay_meta.game_type;

        assert_eq!(
            game_type.game_type, fixture.game_type,
            "{} game type",
            fixture.path
        );
        assert_eq!(
            game_type.playlist_id, fixture.playlist_id,
            "{} playlist id",
            fixture.path
        );
        assert_eq!(
            game_type.match_type_class.as_deref(),
            fixture.match_type_class,
            "{} match type class",
            fixture.path
        );
        assert_eq!(
            processor.get_replay_game_type_details(),
            game_type,
            "{} processor game type details",
            fixture.path
        );
    }
}

#[test]
fn season_is_resolved_from_fixture_replay_date() {
    let replay = common::parse_replay("assets/post-eac-ranked-doubles-2026-04-28.replay");
    let mut processor = ReplayProcessor::new(&replay).expect("failed to build processor");
    let replay_meta = processor
        .process_and_get_replay_meta()
        .expect("failed to build replay meta");
    // Recorded 2026-04-28, which falls in free-to-play season 22
    // (S22 went live 2026-03-11, S23 on 2026-06-10).
    let season = replay_meta.season.expect("season should be resolved");
    assert_eq!(season.era, subtr_actor::SeasonEra::FreeToPlay);
    assert_eq!(season.number, 22);
    assert_eq!(season.code(), "f22");
}
