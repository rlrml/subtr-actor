use super::*;

impl MatchStatsCalculator {
    pub(super) fn update_boost_leadup_samples(
        &mut self,
        frame: &FrameInfo,
        players: &PlayerFrameState,
    ) {
        let cutoff_time = frame.time - GOAL_CONTEXT_BOOST_LEADUP_SECONDS;
        for player in &players.players {
            let Some(boost_amount) = player.boost_amount.or(player.last_boost_amount) else {
                continue;
            };
            let samples = self
                .boost_leadup_samples_by_player
                .entry(player.player_id.clone())
                .or_default();
            samples.push_back(BoostLeadupSample {
                time: frame.time,
                boost_amount,
            });
            while samples
                .front()
                .is_some_and(|sample| sample.time < cutoff_time)
            {
                samples.pop_front();
            }
        }

        self.boost_leadup_samples_by_player
            .retain(|_, samples| !samples.is_empty());
    }

    pub(super) fn boost_leadup_for_player(&self, player_id: &PlayerId) -> Option<BoostLeadupStats> {
        let samples = self.boost_leadup_samples_by_player.get(player_id)?;
        if samples.is_empty() {
            return None;
        }

        let mut sum = 0.0;
        let mut min_boost = f32::INFINITY;
        for sample in samples {
            sum += sample.boost_amount;
            min_boost = min_boost.min(sample.boost_amount);
        }

        Some(BoostLeadupStats {
            average_boost: sum / samples.len() as f32,
            min_boost,
        })
    }
}
