use super::*;

impl TouchCalculator {
    pub(crate) fn begin_sample(&mut self, frame: &FrameInfo) {
        for stats in self.player_stats.values_mut() {
            stats.is_last_touch = false;
            stats.time_since_last_touch = stats
                .last_touch_time
                .map(|time| (frame.time - time).max(0.0));
            stats.frames_since_last_touch = stats
                .last_touch_frame
                .map(|last_frame| frame.frame_number.saturating_sub(last_frame));
        }
    }

    pub(crate) fn controlled_touch_kind(
        ball: &BallFrameState,
        players: &PlayerFrameState,
        player_id: &PlayerId,
    ) -> Option<BallCarryKind> {
        let ball = ball.sample()?;
        players
            .players
            .iter()
            .find(|player| &player.player_id == player_id)
            .and_then(|player| {
                BallCarryCalculator::carry_frame_sample(player, ball).map(|sample| sample.kind)
            })
    }

    pub(crate) fn player_position(
        players: &PlayerFrameState,
        player_id: &PlayerId,
    ) -> Option<glam::Vec3> {
        players
            .players
            .iter()
            .find(|player| &player.player_id == player_id)
            .and_then(PlayerSample::position)
    }

    pub(crate) fn player_dodge_active(players: &PlayerFrameState, player_id: &PlayerId) -> bool {
        players
            .players
            .iter()
            .find(|player| &player.player_id == player_id)
            .is_some_and(|player| player.dodge_active)
    }
}
