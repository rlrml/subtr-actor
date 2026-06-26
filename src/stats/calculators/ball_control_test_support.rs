use super::*;

pub(crate) fn rigid_body(position: glam::Vec3, velocity: glam::Vec3) -> boxcars::RigidBody {
    boxcars::RigidBody {
        sleeping: false,
        location: glam_to_vec(&position),
        rotation: boxcars::Quaternion {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: 1.0,
        },
        linear_velocity: Some(glam_to_vec(&velocity)),
        angular_velocity: Some(glam_to_vec(&glam::Vec3::ZERO)),
    }
}

pub(crate) fn player(position: glam::Vec3, velocity: glam::Vec3) -> PlayerSample {
    PlayerSample {
        player_id: boxcars::RemoteId::Steam(1),
        is_team_0: true,
        hitbox: default_car_hitbox(),
        rigid_body: Some(rigid_body(position, velocity)),
        boost_amount: None,
        last_boost_amount: None,
        boost_active: false,
        dodge_active: false,
        dodge_torque: None,
        powerslide_active: false,
        match_goals: None,
        match_assists: None,
        match_saves: None,
        match_shots: None,
        match_score: None,
    }
}

pub(crate) fn ball(position: glam::Vec3, velocity: glam::Vec3) -> BallFrameState {
    BallFrameState::Present(BallSample {
        rigid_body: rigid_body(position, velocity),
    })
}

pub(crate) fn frame(frame_number: usize, time: f32) -> FrameInfo {
    FrameInfo {
        frame_number,
        time,
        dt: 0.2,
        seconds_remaining: None,
    }
}

pub(crate) fn touch_state() -> TouchState {
    TouchState {
        last_touch_player: Some(boxcars::RemoteId::Steam(1)),
        ..TouchState::default()
    }
}

pub(crate) fn touch_state_with_touch(frame: usize, time: f32) -> TouchState {
    let player_id = boxcars::RemoteId::Steam(1);
    TouchState {
        touch_events: vec![TouchEvent {
            touch_id: None,
            time,
            frame,
            team_is_team_0: true,
            player: Some(player_id.clone()),
            player_position: None,
            closest_approach_distance: None,
            contact_local_ball_position: None,
            contact_local_hitbox_point: None,
            contact_world_hitbox_point: None,
            dodge_contact: false,
        }],
        last_touch: Some(TouchEvent {
            touch_id: None,
            time,
            frame,
            team_is_team_0: true,
            player: Some(player_id.clone()),
            player_position: None,
            closest_approach_distance: None,
            contact_local_ball_position: None,
            contact_local_hitbox_point: None,
            contact_world_hitbox_point: None,
            dodge_contact: false,
        }),
        last_touch_player: Some(player_id),
        last_touch_team_is_team_0: Some(true),
    }
}

#[derive(Default)]
pub(crate) struct BallCarryHarness {
    tracker: ContinuousBallControlTracker<BallCarryKind>,
    state: ContinuousBallControlState,
    pub(crate) calculator: BallCarryCalculator,
}

impl BallCarryHarness {
    pub(crate) fn update(
        &mut self,
        frame: &FrameInfo,
        ball: &BallFrameState,
        players: &PlayerFrameState,
        touch_state: &TouchState,
        live_play_state: &LivePlayState,
    ) {
        let candidate = if frame.dt > 0.0 {
            BallCarryCalculator::control_candidate(ball, players, live_play_state, touch_state)
        } else {
            None
        };

        let player_statuses = BallCarryCalculator::control_player_statuses(players);
        let touches = BallCarryCalculator::control_touches(touch_state, players);
        self.state.completed_sequences.extend(self.tracker.update(
            frame,
            candidate,
            &player_statuses,
            &touches,
            BallCarryCalculator::min_duration_for_kind,
            BallCarryCalculator::kind_requires_airborne,
        ));
        self.calculator.update(&self.state).unwrap();
    }

    pub(crate) fn finish(&mut self) {
        if let Some(sequence) = self
            .tracker
            .finish(BallCarryCalculator::min_duration_for_kind)
        {
            self.state.completed_sequences.push(sequence);
        }
        self.calculator.update(&self.state).unwrap();
    }
}
