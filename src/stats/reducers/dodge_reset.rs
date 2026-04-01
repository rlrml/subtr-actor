use super::*;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DodgeResetStats {
    pub count: u32,
    pub on_ball_count: u32,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct DodgeResetReducer {
    player_stats: HashMap<PlayerId, DodgeResetStats>,
}

impl DodgeResetReducer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn player_stats(&self) -> &HashMap<PlayerId, DodgeResetStats> {
        &self.player_stats
    }

    fn on_ball_dodge_reset(sample: &StatsSample, player_id: &PlayerId) -> bool {
        const MIN_PLAYER_HEIGHT: f32 = 95.0;
        const MIN_BALL_HEIGHT: f32 = 80.0;
        const MAX_CENTER_DISTANCE: f32 = 180.0;
        const MAX_LOCAL_VERTICAL_OFFSET: f32 = 140.0;

        let Some(ball) = &sample.ball else {
            return false;
        };
        let Some(player) = sample
            .players
            .iter()
            .find(|player| &player.player_id == player_id)
        else {
            return false;
        };
        let Some(player_rigid_body) = &player.rigid_body else {
            return false;
        };

        let ball_position = vec_to_glam(&ball.rigid_body.location);
        let player_position = vec_to_glam(&player_rigid_body.location);
        if player_position.z < MIN_PLAYER_HEIGHT || ball_position.z < MIN_BALL_HEIGHT {
            return false;
        }

        let relative_ball_position = ball_position - player_position;
        let center_distance = relative_ball_position.length();
        if !center_distance.is_finite() || center_distance > MAX_CENTER_DISTANCE {
            return false;
        }

        let player_rotation = quat_to_glam(&player_rigid_body.rotation);
        let local_ball_position = player_rotation.inverse() * relative_ball_position;
        local_ball_position.z <= MAX_LOCAL_VERTICAL_OFFSET
    }
}

impl StatsReducer for DodgeResetReducer {
    fn on_sample(&mut self, sample: &StatsSample) -> SubtrActorResult<()> {
        for event in &sample.dodge_refreshed_events {
            let on_ball = Self::on_ball_dodge_reset(sample, &event.player);
            let stats = self.player_stats.entry(event.player.clone()).or_default();
            stats.count += 1;
            if on_ball {
                stats.on_ball_count += 1;
            }
        }
        Ok(())
    }
}
