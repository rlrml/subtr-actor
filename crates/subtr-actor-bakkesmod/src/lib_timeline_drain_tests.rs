use super::*;

#[test]
fn emits_late_inserted_sorted_timeline_mechanics() {
    let mut pending_events = Vec::new();
    let mut emitted_mechanic_ids = HashSet::new();

    push_mechanic_events_from_timeline(
        &mut pending_events,
        &mut emitted_mechanic_ids,
        &[
            normalized_mechanic("speed_flip:10:0", "speed_flip", 10, 1.0),
            normalized_mechanic("wavedash:20:25:0", "wavedash", 20, 2.0),
        ],
    );
    assert_eq!(pending_events.len(), 2);

    pending_events.clear();
    push_mechanic_events_from_timeline(
        &mut pending_events,
        &mut emitted_mechanic_ids,
        &[
            normalized_mechanic("speed_flip:10:0", "speed_flip", 10, 1.0),
            normalized_mechanic("center:15:30:0", "center", 15, 1.5),
            normalized_mechanic("wavedash:20:25:0", "wavedash", 20, 2.0),
        ],
    );

    assert_eq!(pending_events.len(), 1);
    assert_eq!(pending_events[0].kind, SaMechanicKind::Center);
    assert_eq!(pending_events[0].frame_number, 15);
    assert_eq!(pending_events[0].time, 1.5);
}

#[test]
fn drains_player_owned_events_from_timeline_events() {
    let mut pending_events = Vec::new();
    let mut emitted_mechanic_ids = HashSet::new();
    let mut pending_team_events = Vec::new();
    let mut emitted_team_event_ids = HashSet::new();
    let mut pending_goal_context_events = Vec::new();
    let mut emitted_goal_context_ids = HashSet::new();
    let timeline_events = ReplayStatsTimelineEvents {
        timeline: vec![
            TimelineEvent {
                time: 1.05,
                frame: Some(10),
                kind: TimelineEventKind::Goal,
                player_id: Some(RemoteId::SplitScreen(0)),
                is_team_0: Some(true),
            },
            TimelineEvent {
                time: 1.06,
                frame: Some(10),
                kind: TimelineEventKind::Shot,
                player_id: Some(RemoteId::SplitScreen(0)),
                is_team_0: Some(true),
            },
            TimelineEvent {
                time: 1.07,
                frame: Some(10),
                kind: TimelineEventKind::Save,
                player_id: Some(RemoteId::SplitScreen(1)),
                is_team_0: Some(false),
            },
            TimelineEvent {
                time: 1.08,
                frame: Some(10),
                kind: TimelineEventKind::Assist,
                player_id: Some(RemoteId::SplitScreen(0)),
                is_team_0: Some(true),
            },
            TimelineEvent {
                time: 1.35,
                frame: Some(13),
                kind: TimelineEventKind::Kill,
                player_id: Some(RemoteId::SplitScreen(0)),
                is_team_0: Some(true),
            },
            TimelineEvent {
                time: 1.35,
                frame: Some(13),
                kind: TimelineEventKind::Death,
                player_id: Some(RemoteId::SplitScreen(1)),
                is_team_0: Some(false),
            },
        ],
        goal_context: vec![goal_context_event(10, 1.09)],
        mechanics: vec![normalized_mechanic(
            "speed_flip:15:0",
            "speed_flip",
            15,
            1.5,
        )],
        backboard: vec![backboard_event(11, 1.1)],
        whiff: vec![whiff_event(12, 1.2, 0)],
        boost_pickups: vec![boost_pickup_event(125, 1.25)],
        bump: vec![bump_event(13, 1.3, 0.42)],
        fifty_fifty: vec![fifty_fifty_event(9, 14, 1.4)],
        goal_tags: vec![
            goal_tag_event(GoalTagKind::FlickGoal, Some(RemoteId::SplitScreen(1))),
            goal_tag_event(GoalTagKind::AerialGoal, None),
        ],
        rush: vec![rush_event(8, 16, 1.6, true)],
        ..ReplayStatsTimelineEvents::default()
    };

    push_drainable_events_from_timeline(
        &mut pending_events,
        &mut emitted_mechanic_ids,
        &mut pending_team_events,
        &mut emitted_team_event_ids,
        &mut pending_goal_context_events,
        &mut emitted_goal_context_ids,
        &timeline_events,
    );

    assert_eq!(pending_events.len(), 13);
    assert_eq!(pending_events[0].kind, SaMechanicKind::Goal);
    assert_eq!(pending_events[0].frame_number, 10);
    assert_eq!(pending_events[0].player_index, 0);
    assert_eq!(pending_events[1].kind, SaMechanicKind::Shot);
    assert_eq!(pending_events[1].frame_number, 10);
    assert_eq!(pending_events[1].player_index, 0);
    assert_eq!(pending_events[2].kind, SaMechanicKind::Save);
    assert_eq!(pending_events[2].frame_number, 10);
    assert_eq!(pending_events[2].player_index, 1);
    assert_eq!(pending_events[3].kind, SaMechanicKind::Assist);
    assert_eq!(pending_events[3].frame_number, 10);
    assert_eq!(pending_events[3].player_index, 0);
    assert_eq!(pending_events[4].kind, SaMechanicKind::Backboard);
    assert_eq!(pending_events[4].frame_number, 11);
    assert_eq!(pending_events[4].player_index, 0);
    assert_eq!(pending_events[5].kind, SaMechanicKind::Whiff);
    assert_eq!(pending_events[5].frame_number, 12);
    assert_eq!(pending_events[5].player_index, 0);
    assert_eq!(pending_events[6].kind, SaMechanicKind::BoostPickup);
    assert_eq!(pending_events[6].frame_number, 125);
    assert_eq!(pending_events[6].player_index, 0);
    assert_eq!(pending_events[7].kind, SaMechanicKind::Bump);
    assert_eq!(pending_events[7].frame_number, 13);
    assert_eq!(pending_events[7].player_index, 0);
    assert_eq!(pending_events[7].confidence, 0.42);
    assert_eq!(pending_events[8].kind, SaMechanicKind::Demo);
    assert_eq!(pending_events[8].time, 1.35);
    assert_eq!(pending_events[8].frame_number, 13);
    assert_eq!(pending_events[8].player_index, 0);
    assert_eq!(pending_events[9].kind, SaMechanicKind::Death);
    assert_eq!(pending_events[9].time, 1.35);
    assert_eq!(pending_events[9].frame_number, 13);
    assert_eq!(pending_events[9].player_index, 1);
    assert_eq!(pending_events[9].is_team_0, 0);
    assert_eq!(pending_events[10].kind, SaMechanicKind::FlickGoal);
    assert_eq!(pending_events[10].time, 1.36);
    assert_eq!(pending_events[10].frame_number, 13);
    assert_eq!(pending_events[10].player_index, 1);
    assert_eq!(pending_events[10].is_team_0, 0);
    assert_eq!(pending_events[10].confidence, 0.72);
    assert_eq!(pending_events[11].kind, SaMechanicKind::FiftyFifty);
    assert_eq!(pending_events[11].frame_number, 14);
    assert_eq!(pending_events[11].player_index, 1);
    assert_eq!(pending_events[11].is_team_0, 0);
    assert_eq!(pending_events[12].kind, SaMechanicKind::SpeedFlip);
    assert_eq!(pending_team_events.len(), 1);
    assert_eq!(pending_team_events[0].kind, SaTeamEventKind::Rush);
    assert_eq!(pending_team_events[0].is_team_0, 1);
    assert_eq!(pending_team_events[0].start_frame, 8);
    assert_eq!(pending_team_events[0].end_frame, 16);
    assert_eq!(pending_team_events[0].start_time, 1.0);
    assert_eq!(pending_team_events[0].end_time, 1.6);
    assert_eq!(pending_team_events[0].attackers, 3);
    assert_eq!(pending_team_events[0].defenders, 2);
    assert_eq!(pending_goal_context_events.len(), 1);
    assert_eq!(pending_goal_context_events[0].frame_number, 10);
    assert_eq!(pending_goal_context_events[0].time, 1.09);
    assert_eq!(pending_goal_context_events[0].scoring_team_is_team_0, 0);
    assert_eq!(pending_goal_context_events[0].has_scorer, 1);
    assert_eq!(pending_goal_context_events[0].scorer_index, 1);
    assert_eq!(
        pending_goal_context_events[0].has_defending_team_most_back_player,
        1
    );
    assert_eq!(
        pending_goal_context_events[0].defending_team_most_back_player_index,
        0
    );
    assert_eq!(pending_goal_context_events[0].has_ball_position, 1);
    assert_eq!(pending_goal_context_events[0].ball_position.x, 1.0);
    assert_eq!(
        pending_goal_context_events[0].has_ball_air_time_before_goal,
        1
    );
    assert_eq!(
        pending_goal_context_events[0].goal_buildup,
        SaGoalBuildupKind::CounterAttack
    );

    pending_events.clear();
    pending_team_events.clear();
    pending_goal_context_events.clear();
    push_drainable_events_from_timeline(
        &mut pending_events,
        &mut emitted_mechanic_ids,
        &mut pending_team_events,
        &mut emitted_team_event_ids,
        &mut pending_goal_context_events,
        &mut emitted_goal_context_ids,
        &timeline_events,
    );
    assert!(pending_events.is_empty());
    assert!(pending_team_events.is_empty());
    assert!(pending_goal_context_events.is_empty());
}

#[test]
fn maps_normalized_timeline_mechanic_kinds_to_abi_kinds() {
    let expected_shared_graph_kinds = HashSet::from([
        "air_dribble",
        "ball_carry",
        "ceiling_shot",
        "center",
        "double_tap",
        "flick",
        "flip_reset",
        "half_flip",
        "half_volley",
        "musty_flick",
        "one_timer",
        "pass",
        "speed_flip",
        "wall_aerial",
        "wall_aerial_shot",
        "wavedash",
    ]);
    let shared_graph_kinds = STATS_TIMELINE_MECHANIC_KINDS
        .iter()
        .copied()
        .collect::<HashSet<_>>();
    assert_eq!(
        shared_graph_kinds, expected_shared_graph_kinds,
        "shared stats timeline mechanic kind set changed; update ABI mapping expectations"
    );
    for &kind in STATS_TIMELINE_MECHANIC_KINDS {
        assert!(
            mechanic_kind(kind).is_some(),
            "BakkesMod ABI mapping must cover shared stats timeline mechanic kind: {kind}"
        );
    }

    assert_eq!(
        mechanic_kind("air_dribble"),
        Some(SaMechanicKind::AirDribble)
    );
    assert_eq!(mechanic_kind("ball_carry"), Some(SaMechanicKind::BallCarry));
    assert_eq!(
        mechanic_kind("ceiling_shot"),
        Some(SaMechanicKind::CeilingShot)
    );
    assert_eq!(mechanic_kind("center"), Some(SaMechanicKind::Center));
    assert_eq!(mechanic_kind("double_tap"), Some(SaMechanicKind::DoubleTap));
    assert_eq!(mechanic_kind("flick"), Some(SaMechanicKind::Flick));
    assert_eq!(mechanic_kind("flip_reset"), Some(SaMechanicKind::FlipReset));
    assert_eq!(mechanic_kind("half_flip"), Some(SaMechanicKind::HalfFlip));
    assert_eq!(
        mechanic_kind("half_volley"),
        Some(SaMechanicKind::HalfVolley)
    );
    assert_eq!(
        mechanic_kind("musty_flick"),
        Some(SaMechanicKind::MustyFlick)
    );
    assert_eq!(mechanic_kind("one_timer"), Some(SaMechanicKind::OneTimer));
    assert_eq!(mechanic_kind("pass"), Some(SaMechanicKind::Pass));
    assert_eq!(mechanic_kind("speed_flip"), Some(SaMechanicKind::SpeedFlip));
    assert_eq!(
        mechanic_kind("wall_aerial"),
        Some(SaMechanicKind::WallAerial)
    );
    assert_eq!(
        mechanic_kind("wall_aerial_shot"),
        Some(SaMechanicKind::WallAerialShot)
    );
    assert_eq!(mechanic_kind("wavedash"), Some(SaMechanicKind::Wavedash));
    assert_eq!(mechanic_kind("unmapped"), None);
}

#[test]
fn maps_timeline_event_kinds_to_abi_kinds() {
    assert_eq!(
        timeline_event_kind(TimelineEventKind::Goal),
        SaMechanicKind::Goal
    );
    assert_eq!(
        timeline_event_kind(TimelineEventKind::Shot),
        SaMechanicKind::Shot
    );
    assert_eq!(
        timeline_event_kind(TimelineEventKind::Save),
        SaMechanicKind::Save
    );
    assert_eq!(
        timeline_event_kind(TimelineEventKind::Assist),
        SaMechanicKind::Assist
    );
    assert_eq!(
        timeline_event_kind(TimelineEventKind::Kill),
        SaMechanicKind::Demo
    );
    assert_eq!(
        timeline_event_kind(TimelineEventKind::Death),
        SaMechanicKind::Death
    );
}

#[test]
fn maps_goal_tag_kinds_to_abi_kinds() {
    assert_eq!(
        goal_tag_kind(GoalTagKind::AerialGoal),
        SaMechanicKind::AerialGoal
    );
    assert_eq!(
        goal_tag_kind(GoalTagKind::HighAerialGoal),
        SaMechanicKind::HighAerialGoal
    );
    assert_eq!(
        goal_tag_kind(GoalTagKind::LongDistanceGoal),
        SaMechanicKind::LongDistanceGoal
    );
    assert_eq!(
        goal_tag_kind(GoalTagKind::OwnHalfGoal),
        SaMechanicKind::OwnHalfGoal
    );
    assert_eq!(
        goal_tag_kind(GoalTagKind::EmptyNetGoal),
        SaMechanicKind::EmptyNetGoal
    );
    assert_eq!(
        goal_tag_kind(GoalTagKind::CounterAttackGoal),
        SaMechanicKind::CounterAttackGoal
    );
    assert_eq!(
        goal_tag_kind(GoalTagKind::FlickGoal),
        SaMechanicKind::FlickGoal
    );
    assert_eq!(
        goal_tag_kind(GoalTagKind::DoubleTapGoal),
        SaMechanicKind::DoubleTapGoal
    );
    assert_eq!(
        goal_tag_kind(GoalTagKind::OneTimerGoal),
        SaMechanicKind::OneTimerGoal
    );
    assert_eq!(
        goal_tag_kind(GoalTagKind::PassingGoal),
        SaMechanicKind::PassingGoal
    );
    assert_eq!(
        goal_tag_kind(GoalTagKind::AirDribbleGoal),
        SaMechanicKind::AirDribbleGoal
    );
    assert_eq!(
        goal_tag_kind(GoalTagKind::FlipResetGoal),
        SaMechanicKind::FlipResetGoal
    );
    assert_eq!(
        goal_tag_kind(GoalTagKind::HalfVolleyGoal),
        SaMechanicKind::HalfVolleyGoal
    );
}
