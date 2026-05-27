use super::positioning_ball_depth::{
    record_ball_depth_positioning, BallDepthPositioningSample,
};
use super::positioning_player_totals::{
    record_field_positioning, record_live_player_totals, record_possession_distance,
};
use super::*;

impl PositioningCalculator {
    pub(crate) fn record_live_player_sample(
        &mut self,
        frame: &FrameInfo,
        player: &PlayerSample,
        position: glam::Vec3,
        ball_position: glam::Vec3,
        possession_player_before_sample: Option<&PlayerId>,
        event_deltas: &mut HashMap<PlayerId, PositioningEvent>,
    ) {
        let previous_position = self
            .previous_player_positions
            .get(&player.player_id)
            .copied()
            .unwrap_or(position);
        let previous_ball_position = self.previous_ball_position.unwrap_or(ball_position);
        let distance_to_ball = position.distance(ball_position);
        let stats = self
            .player_stats
            .entry(player.player_id.clone())
            .or_default();
        let delta = Self::event_delta(event_deltas, frame, &player.player_id, player.is_team_0);

        record_live_player_totals(stats, delta, frame.dt, distance_to_ball);
        record_possession_distance(
            stats,
            delta,
            frame.dt,
            distance_to_ball,
            possession_player_before_sample == Some(&player.player_id),
            possession_player_before_sample.is_some(),
        );
        record_field_positioning(
            stats,
            delta,
            frame.dt,
            player.is_team_0,
            previous_position,
            position,
        );
        record_ball_depth_positioning(
            stats,
            delta,
            BallDepthPositioningSample {
                dt: frame.dt,
                level_margin: self.config.level_ball_depth_margin,
                is_team_0: player.is_team_0,
                previous_position,
                position,
                previous_ball_position,
                ball_position,
            },
        );
    }
}
