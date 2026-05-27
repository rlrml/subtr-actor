use super::*;

impl SpeedFlipCalculator {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn insert_candidate(
        &mut self,
        frame: &FrameInfo,
        player: &PlayerSample,
        rigid_body: &boxcars::RigidBody,
        is_kickoff: bool,
        kickoff_start_time: Option<f32>,
        player_position: glam::Vec3,
        start_speed: f32,
        best_alignment: f32,
        start_velocity_xy: glam::Vec2,
        start_forward_xy: glam::Vec2,
    ) {
        let rotation = quat_to_glam(&rigid_body.rotation);
        let local_angular_velocity = rigid_body
            .angular_velocity
            .as_ref()
            .map(vec_to_glam)
            .map(|angular_velocity| rotation.inverse() * angular_velocity)
            .unwrap_or(glam::Vec3::ZERO);
        let best_diagonal_score = Self::diagonal_score(local_angular_velocity);
        let forward_z = (rotation * glam::Vec3::X).z;

        self.active_candidates.insert(
            player.player_id.clone(),
            ActiveSpeedFlipCandidate {
                is_team_0: player.is_team_0,
                is_kickoff,
                kickoff_start_time,
                start_time: frame.time,
                start_frame: frame.frame_number,
                start_position: player_position.to_array(),
                end_position: player_position.to_array(),
                start_velocity_xy,
                start_forward_xy,
                start_speed,
                max_speed: start_speed,
                best_alignment,
                best_boost_alignment: Self::boost_alignment(player).unwrap_or(best_alignment),
                boost_alignment_sample_count: u32::from(player.boost_active),
                best_dodge_forward_delta: 0.0,
                best_dodge_delta_alignment: -1.0,
                dodge_acceleration_sample_count: 0,
                best_diagonal_score,
                min_forward_z: forward_z,
                latest_forward_z: forward_z,
                latest_time: frame.time,
                latest_frame: frame.frame_number,
            },
        );
    }

    pub(super) fn update_candidate(
        candidate: &mut ActiveSpeedFlipCandidate,
        frame: &FrameInfo,
        ball: &BallFrameState,
        player: &PlayerSample,
    ) {
        let Some(rigid_body) = player.rigid_body.as_ref() else {
            return;
        };

        if let Some(player_position) = player.position() {
            candidate.end_position = player_position.to_array();
        }
        candidate.max_speed = candidate.max_speed.max(player.speed().unwrap_or(0.0));
        if let Some(alignment) = Self::candidate_alignment(ball, player, candidate.is_kickoff) {
            candidate.best_alignment = candidate.best_alignment.max(alignment);
        }
        if let Some(boost_alignment) = Self::boost_alignment(player) {
            candidate.best_boost_alignment = candidate.best_boost_alignment.max(boost_alignment);
            candidate.boost_alignment_sample_count += 1;
        }
        Self::update_candidate_dodge_acceleration(candidate, frame, player);
        Self::update_candidate_rotation(candidate, frame, rigid_body);
    }
}
