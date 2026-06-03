use super::*;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct PositioningStats {
    pub active_game_time: f32,
    pub tracked_time: f32,
    pub sum_distance_to_teammates: f32,
    pub sum_distance_to_ball: f32,
    pub sum_distance_to_ball_has_possession: f32,
    pub time_has_possession: f32,
    pub sum_distance_to_ball_no_possession: f32,
    pub time_no_possession: f32,
    pub time_demolished: f32,
    pub time_no_teammates: f32,
    pub time_most_back: f32,
    pub time_most_forward: f32,
    pub time_mid_role: f32,
    pub time_other_role: f32,
    #[serde(rename = "time_defensive_third")]
    pub time_defensive_zone: f32,
    #[serde(rename = "time_neutral_third")]
    pub time_neutral_zone: f32,
    #[serde(rename = "time_offensive_third")]
    pub time_offensive_zone: f32,
    pub time_defensive_half: f32,
    pub time_offensive_half: f32,
    pub time_closest_to_ball: f32,
    pub time_farthest_from_ball: f32,
    pub time_behind_ball: f32,
    pub time_level_with_ball: f32,
    pub time_in_front_of_ball: f32,
    pub times_caught_ahead_of_play_on_conceded_goals: u32,
}

impl PositioningStats {
    pub fn average_distance_to_teammates(&self) -> f32 {
        if self.tracked_time == 0.0 {
            0.0
        } else {
            self.sum_distance_to_teammates / self.tracked_time
        }
    }

    pub fn average_distance_to_ball(&self) -> f32 {
        if self.tracked_time == 0.0 {
            0.0
        } else {
            self.sum_distance_to_ball / self.tracked_time
        }
    }

    pub fn average_distance_to_ball_has_possession(&self) -> f32 {
        if self.time_has_possession == 0.0 {
            0.0
        } else {
            self.sum_distance_to_ball_has_possession / self.time_has_possession
        }
    }

    pub fn average_distance_to_ball_no_possession(&self) -> f32 {
        if self.time_no_possession == 0.0 {
            0.0
        } else {
            self.sum_distance_to_ball_no_possession / self.time_no_possession
        }
    }

    fn pct(&self, value: f32) -> f32 {
        if self.tracked_time == 0.0 {
            0.0
        } else {
            value * 100.0 / self.tracked_time
        }
    }

    pub fn most_back_pct(&self) -> f32 {
        self.pct(self.time_most_back)
    }

    pub fn most_forward_pct(&self) -> f32 {
        self.pct(self.time_most_forward)
    }

    pub fn mid_role_pct(&self) -> f32 {
        self.pct(self.time_mid_role)
    }

    pub fn other_role_pct(&self) -> f32 {
        self.pct(self.time_other_role)
    }

    pub fn defensive_third_pct(&self) -> f32 {
        self.pct(self.time_defensive_zone)
    }

    pub fn neutral_third_pct(&self) -> f32 {
        self.pct(self.time_neutral_zone)
    }

    pub fn offensive_third_pct(&self) -> f32 {
        self.pct(self.time_offensive_zone)
    }

    pub fn defensive_zone_pct(&self) -> f32 {
        self.defensive_third_pct()
    }

    pub fn neutral_zone_pct(&self) -> f32 {
        self.neutral_third_pct()
    }

    pub fn offensive_zone_pct(&self) -> f32 {
        self.offensive_third_pct()
    }

    pub fn defensive_half_pct(&self) -> f32 {
        self.pct(self.time_defensive_half)
    }

    pub fn offensive_half_pct(&self) -> f32 {
        self.pct(self.time_offensive_half)
    }

    pub fn closest_to_ball_pct(&self) -> f32 {
        self.pct(self.time_closest_to_ball)
    }

    pub fn farthest_from_ball_pct(&self) -> f32 {
        self.pct(self.time_farthest_from_ball)
    }

    pub fn behind_ball_pct(&self) -> f32 {
        self.pct(self.time_behind_ball)
    }

    pub fn level_with_ball_pct(&self) -> f32 {
        self.pct(self.time_level_with_ball)
    }

    pub fn in_front_of_ball_pct(&self) -> f32 {
        self.pct(self.time_in_front_of_ball)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct PositioningStatsAccumulator {
    player_stats: HashMap<PlayerId, PositioningStats>,
}

impl PositioningStatsAccumulator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn player_stats(&self) -> &HashMap<PlayerId, PositioningStats> {
        &self.player_stats
    }

    pub fn apply_event(&mut self, event: &PositioningEvent) {
        let stats = self.player_stats.entry(event.player.clone()).or_default();
        stats.active_game_time += event.active_game_time;
        stats.tracked_time += event.tracked_time;
        stats.sum_distance_to_teammates += event.sum_distance_to_teammates;
        stats.sum_distance_to_ball += event.sum_distance_to_ball;
        stats.sum_distance_to_ball_has_possession += event.sum_distance_to_ball_has_possession;
        stats.time_has_possession += event.time_has_possession;
        stats.sum_distance_to_ball_no_possession += event.sum_distance_to_ball_no_possession;
        stats.time_no_possession += event.time_no_possession;
        stats.time_demolished += event.time_demolished;
        stats.time_no_teammates += event.time_no_teammates;
        stats.time_most_back += event.time_most_back;
        stats.time_most_forward += event.time_most_forward;
        stats.time_mid_role += event.time_mid_role;
        stats.time_other_role += event.time_other_role;
        stats.time_defensive_zone += event.time_defensive_zone;
        stats.time_neutral_zone += event.time_neutral_zone;
        stats.time_offensive_zone += event.time_offensive_zone;
        stats.time_defensive_half += event.time_defensive_half;
        stats.time_offensive_half += event.time_offensive_half;
        stats.time_closest_to_ball += event.time_closest_to_ball;
        stats.time_farthest_from_ball += event.time_farthest_from_ball;
        stats.time_behind_ball += event.time_behind_ball;
        stats.time_level_with_ball += event.time_level_with_ball;
        stats.time_in_front_of_ball += event.time_in_front_of_ball;
        stats.times_caught_ahead_of_play_on_conceded_goals +=
            event.times_caught_ahead_of_play_on_conceded_goals;
    }

    pub fn apply_events<'a>(&mut self, events: impl IntoIterator<Item = &'a PositioningEvent>) {
        for event in events {
            self.apply_event(event);
        }
    }
}
