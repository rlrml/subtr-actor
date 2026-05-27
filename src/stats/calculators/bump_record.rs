use super::*;

impl BumpCalculator {
    pub(super) fn record_bump(&mut self, event: BumpEvent) {
        let initiator_stats = self
            .player_stats
            .entry(event.initiator.clone())
            .or_default();
        initiator_stats.bumps_inflicted += 1;
        if event.is_team_bump {
            initiator_stats.team_bumps_inflicted += 1;
        }
        initiator_stats.last_bump_time = Some(event.time);
        initiator_stats.last_bump_frame = Some(event.frame);
        initiator_stats.last_bump_strength = Some(event.strength);
        initiator_stats.max_bump_strength = initiator_stats.max_bump_strength.max(event.strength);
        initiator_stats.cumulative_bump_strength += event.strength;

        let victim_stats = self.player_stats.entry(event.victim.clone()).or_default();
        victim_stats.bumps_taken += 1;
        if event.is_team_bump {
            victim_stats.team_bumps_taken += 1;
        }

        match event.initiator_is_team_0 {
            true => self.record_team_zero_bump(event.is_team_bump),
            false => self.record_team_one_bump(event.is_team_bump),
        }

        self.events.push(event);
    }

    fn record_team_zero_bump(&mut self, is_team_bump: bool) {
        self.team_zero_stats.bumps_inflicted += 1;
        if is_team_bump {
            self.team_zero_stats.team_bumps_inflicted += 1;
        }
    }

    fn record_team_one_bump(&mut self, is_team_bump: bool) {
        self.team_one_stats.bumps_inflicted += 1;
        if is_team_bump {
            self.team_one_stats.team_bumps_inflicted += 1;
        }
    }
}
