use super::*;

impl BallCarryCalculator {
    pub(crate) fn kind_requires_airborne(kind: BallCarryKind) -> bool {
        AirDribblePolicy::kind_requires_airborne(kind)
    }

    pub(crate) fn control_player_statuses(
        players: &PlayerFrameState,
    ) -> Vec<ContinuousBallControlPlayerStatus> {
        players
            .players
            .iter()
            .filter_map(|player| {
                Some(ContinuousBallControlPlayerStatus {
                    player_id: player.player_id.clone(),
                    is_airborne: AirDribblePolicy::is_air_touch_position(player.position()?),
                })
            })
            .collect()
    }

    pub(crate) fn control_touches(
        touch_state: &TouchState,
        players: &PlayerFrameState,
    ) -> Vec<ContinuousBallControlTouch> {
        touch_state
            .touch_events
            .iter()
            .filter_map(|touch| {
                let player_id = touch.player.clone()?;
                let player = players
                    .players
                    .iter()
                    .find(|player| player.player_id == player_id)?;
                Some(ContinuousBallControlTouch {
                    player_id,
                    is_airborne: AirDribblePolicy::is_air_touch_position(player.position()?),
                })
            })
            .collect()
    }

    pub(crate) fn min_duration_for_kind(kind: BallCarryKind) -> f32 {
        match kind {
            BallCarryKind::Carry => BALL_CARRY_MIN_DURATION,
            BallCarryKind::AirDribble => AIR_DRIBBLE_MIN_DURATION,
        }
    }

    pub(crate) fn control_candidate(
        ball: &BallFrameState,
        players: &PlayerFrameState,
        live_play: bool,
        touch_state: &TouchState,
    ) -> Option<ContinuousBallControlCandidate<BallCarryKind>> {
        if !live_play {
            return None;
        }
        let ball = ball.sample()?;
        let player_id = touch_state.last_touch_player.as_ref()?;
        let touch_count = touch_count_for_player(touch_state, player_id);
        players
            .players
            .iter()
            .find(|player| &player.player_id == player_id)
            .and_then(|player| control_candidate_for_player(player, ball, touch_count))
    }
}

fn touch_count_for_player(touch_state: &TouchState, player_id: &PlayerId) -> u32 {
    touch_state
        .touch_events
        .iter()
        .filter(|event| event.player.as_ref() == Some(player_id))
        .count() as u32
}

fn control_candidate_for_player(
    player: &PlayerSample,
    ball: &BallSample,
    touch_count: u32,
) -> Option<ContinuousBallControlCandidate<BallCarryKind>> {
    BallCarryCalculator::carry_frame_sample(player, ball).map(|sample| {
        let air_touch_count = if AirDribblePolicy::is_air_touch_position(sample.player_position) {
            touch_count
        } else {
            0
        };
        ContinuousBallControlCandidate {
            player_id: player.player_id.clone(),
            is_team_0: player.is_team_0,
            touch_count,
            air_touch_count,
            sample,
        }
    })
}
