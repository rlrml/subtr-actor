use super::*;

pub(crate) fn test_rigid_body(location: SaVec3, linear_velocity: SaVec3) -> SaRigidBody {
    SaRigidBody {
        location,
        rotation: SaQuat::default(),
        linear_velocity,
        angular_velocity: SaVec3::default(),
        has_linear_velocity: 1,
        has_angular_velocity: 1,
        sleeping: 0,
    }
}

pub(crate) fn live_frame(
    frame_number: u64,
    ball: SaRigidBody,
    players: &[SaPlayerFrame],
) -> SaLiveFrame {
    SaLiveFrame {
        frame_number,
        time: frame_number as f32 * 0.1,
        dt: 0.1,
        seconds_remaining: 299,
        has_seconds_remaining: 1,
        game_state: 0,
        has_game_state: 0,
        kickoff_countdown_time: 0,
        has_kickoff_countdown_time: 0,
        ball_has_been_hit: 1,
        has_ball_has_been_hit: 1,
        live_play: 1,
        has_ball: 1,
        ball,
        players: players.as_ptr(),
        player_count: players.len(),
        ..SaLiveFrame::default()
    }
}

pub(crate) fn player_at_index(
    player_index: u32,
    is_team_0: bool,
    location: SaVec3,
) -> SaPlayerFrame {
    SaPlayerFrame {
        player_index,
        player_name: ptr::null(),
        is_team_0: is_team_0 as u8,
        has_rigid_body: 1,
        rigid_body: test_rigid_body(location, SaVec3::default()),
        boost_amount: 33.0,
        last_boost_amount: 33.0,
        boost_active: 0,
        jump_active: 0,
        double_jump_active: 0,
        dodge_active: 0,
        powerslide_active: 0,
        has_match_stats: 1,
        match_goals: player_index as i32,
        match_assists: player_index as i32 + 1,
        match_saves: player_index as i32 + 2,
        match_shots: player_index as i32 + 3,
        match_score: player_index as i32 + 100,
    }
}

pub(crate) fn player_at(location: SaVec3) -> SaPlayerFrame {
    player_at_index(0, true, location)
}

pub(crate) fn normalized_mechanic(id: &str, kind: &str, frame: usize, time: f32) -> MechanicEvent {
    MechanicEvent {
        id: id.to_owned(),
        kind: kind.to_owned(),
        player_id: RemoteId::SplitScreen(0),
        is_team_0: true,
        timing: MechanicTiming::Moment { frame, time },
        properties: Vec::new(),
    }
}

pub(crate) fn whiff_event(frame: usize, time: f32, player_index: u32) -> WhiffEvent {
    WhiffEvent {
        kind: WhiffEventKind::Whiff,
        time,
        frame,
        resolved_time: time,
        resolved_frame: frame,
        player: RemoteId::SplitScreen(player_index),
        is_team_0: player_index == 0,
        closest_approach_distance: 42.0,
        forward_alignment: 0.7,
        approach_speed: 900.0,
        dodge_active: false,
        aerial: false,
    }
}

pub(crate) fn bump_event(frame: usize, time: f32, confidence: f32) -> BumpEvent {
    BumpEvent {
        time,
        frame,
        initiator: RemoteId::SplitScreen(0),
        victim: RemoteId::SplitScreen(1),
        initiator_is_team_0: true,
        victim_is_team_0: false,
        is_team_bump: false,
        strength: 800.0,
        confidence,
        contact_distance: 120.0,
        closing_speed: 500.0,
        victim_impulse: 220.0,
        initiator_position: [0.0, 0.0, 0.0],
        victim_position: [100.0, 0.0, 0.0],
    }
}

pub(crate) fn backboard_event(frame: usize, time: f32) -> BackboardBounceEvent {
    BackboardBounceEvent {
        time,
        frame,
        player: RemoteId::SplitScreen(0),
        is_team_0: true,
    }
}

pub(crate) fn boost_pickup_event(frame: usize, time: f32) -> BoostPickupComparisonEvent {
    BoostPickupComparisonEvent {
        comparison: BoostPickupComparison::Both,
        frame,
        time,
        player_id: RemoteId::SplitScreen(0),
        is_team_0: true,
        pad_type: BoostPickupPadType::Big,
        field_half: BoostPickupFieldHalf::Opponent,
        activity: BoostPickupActivity::Active,
        reported_frame: Some(frame),
        reported_time: Some(time),
        inferred_frame: None,
        inferred_time: None,
        boost_before: Some(20.0),
        boost_after: Some(100.0),
    }
}

pub(crate) fn fifty_fifty_event(
    start_frame: usize,
    resolve_frame: usize,
    resolve_time: f32,
) -> FiftyFiftyEvent {
    FiftyFiftyEvent {
        start_time: 1.0,
        start_frame,
        resolve_time,
        resolve_frame,
        is_kickoff: false,
        team_zero_player: Some(RemoteId::SplitScreen(0)),
        team_one_player: Some(RemoteId::SplitScreen(1)),
        team_zero_touch_time: None,
        team_zero_touch_frame: None,
        team_zero_dodge_contact: false,
        team_one_touch_time: None,
        team_one_touch_frame: None,
        team_one_dodge_contact: false,
        team_zero_position: [0.0, 0.0, 0.0],
        team_one_position: [100.0, 0.0, 0.0],
        midpoint: [50.0, 0.0, 0.0],
        plane_normal: [1.0, 0.0, 0.0],
        winning_team_is_team_0: Some(false),
        possession_team_is_team_0: Some(false),
    }
}

pub(crate) fn goal_tag_event(kind: GoalTagKind, scorer: Option<RemoteId>) -> GoalTagEvent {
    GoalTagEvent {
        goal_index: 0,
        time: 1.36,
        frame: 13,
        kind,
        scoring_team_is_team_0: false,
        scorer,
        confidence: 0.72,
        modifiers: Vec::new(),
        evidence: Vec::new(),
    }
}

pub(crate) fn rush_event(
    start_frame: usize,
    end_frame: usize,
    end_time: f32,
    is_team_0: bool,
) -> RushEvent {
    RushEvent {
        start_time: 1.0,
        start_frame,
        end_time,
        end_frame,
        is_team_0,
        attackers: 3,
        defenders: 2,
    }
}

pub(crate) fn goal_context_event(frame: usize, time: f32) -> GoalContextEvent {
    GoalContextEvent {
        time,
        frame,
        scoring_team_is_team_0: false,
        scorer: Some(RemoteId::SplitScreen(1)),
        scoring_team_most_back_player: Some(RemoteId::SplitScreen(1)),
        defending_team_most_back_player: Some(RemoteId::SplitScreen(0)),
        ball_position: Some(subtr_actor::GoalContextPosition {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        }),
        ball_air_time_before_goal: Some(1.25),
        goal_buildup: GoalBuildupKind::CounterAttack,
        scorer_last_touch: None,
        players: Vec::new(),
    }
}
