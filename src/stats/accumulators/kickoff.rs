use super::*;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct KickoffStats {
    pub count: u32,
    pub team_zero_wins: u32,
    pub team_one_wins: u32,
    pub neutral_outcomes: u32,
    pub team_zero_kickoff_possessions: u32,
    pub team_one_kickoff_possessions: u32,
    pub team_zero_kickoff_possession_advantages: u32,
    pub team_one_kickoff_possession_advantages: u32,
    pub contested_kickoff_possessions: u32,
    pub kickoff_goal_count: u32,
    pub team_zero_kickoff_goals: u32,
    pub team_one_kickoff_goals: u32,
    pub win_strength_sample_count: u32,
    pub cumulative_win_strength: f32,
    pub boost_after_sample_count: u32,
    pub cumulative_boost_after: f32,
    pub fake_count: u32,
    pub missed_count: u32,
    #[serde(default, skip_serializing_if = "LabeledCounts::is_empty")]
    pub labeled_event_counts: LabeledCounts,
    #[serde(default, skip_serializing_if = "LabeledCounts::is_empty")]
    pub labeled_player_counts: LabeledCounts,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct KickoffPlayerStats {
    pub count: u32,
    pub touches: u32,
    pub fakes: u32,
    pub misses: u32,
    pub support_go_for_boosts: u32,
    pub support_cheats: u32,
    pub support_other: u32,
    pub kickoff_goal_count: u32,
    pub boost_after_sample_count: u32,
    pub cumulative_boost_after: f32,
    #[serde(default, skip_serializing_if = "LabeledCounts::is_empty")]
    pub labeled_event_counts: LabeledCounts,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct KickoffTeamStats {
    pub count: u32,
    pub wins: u32,
    pub losses: u32,
    pub neutral_outcomes: u32,
    pub kickoff_possessions: u32,
    pub opponent_kickoff_possessions: u32,
    pub kickoff_possession_advantages: u32,
    pub opponent_kickoff_possession_advantages: u32,
    pub contested_kickoff_possessions: u32,
    pub kickoff_goal_count: u32,
    pub kickoff_goals_for: u32,
    pub kickoff_goals_against: u32,
    pub win_strength_sample_count: u32,
    pub cumulative_win_strength: f32,
    pub boost_after_sample_count: u32,
    pub cumulative_boost_after: f32,
    pub fake_count: u32,
    pub missed_count: u32,
}

impl KickoffStats {
    pub fn average_win_strength(&self) -> f32 {
        if self.win_strength_sample_count == 0 {
            0.0
        } else {
            self.cumulative_win_strength / self.win_strength_sample_count as f32
        }
    }

    pub fn average_boost_after(&self) -> f32 {
        if self.boost_after_sample_count == 0 {
            0.0
        } else {
            self.cumulative_boost_after / self.boost_after_sample_count as f32
        }
    }

    pub(crate) fn record_event(&mut self, event: &KickoffEvent) {
        self.count += 1;
        self.labeled_event_counts.increment(event.labels());
        match event.outcome {
            KickoffOutcome::TeamZeroWin => self.team_zero_wins += 1,
            KickoffOutcome::TeamOneWin => self.team_one_wins += 1,
            KickoffOutcome::Neutral => self.neutral_outcomes += 1,
            KickoffOutcome::Unknown => {}
        }
        match event.kickoff_possession_outcome {
            KickoffPossessionOutcome::TeamZeroPossession => self.team_zero_kickoff_possessions += 1,
            KickoffPossessionOutcome::TeamOnePossession => self.team_one_kickoff_possessions += 1,
            KickoffPossessionOutcome::TeamZeroAdvantage => {
                self.team_zero_kickoff_possession_advantages += 1
            }
            KickoffPossessionOutcome::TeamOneAdvantage => {
                self.team_one_kickoff_possession_advantages += 1
            }
            KickoffPossessionOutcome::Contested => self.contested_kickoff_possessions += 1,
        }
        if event.kickoff_goal {
            self.kickoff_goal_count += 1;
            match event.scoring_team_is_team_0 {
                Some(true) => self.team_zero_kickoff_goals += 1,
                Some(false) => self.team_one_kickoff_goals += 1,
                None => {}
            }
        }
        if let Some(win_strength) = event.win_strength {
            self.win_strength_sample_count += 1;
            self.cumulative_win_strength += win_strength;
        }
        for player in event.player_events() {
            self.labeled_player_counts.increment(player.labels());
            if let Some(taker) = player.as_taker() {
                if let Some(boost_after) = taker.boost_after {
                    self.boost_after_sample_count += 1;
                    self.cumulative_boost_after += boost_after;
                }
                match taker.outcome {
                    KickoffTakerOutcome::Fake => self.fake_count += 1,
                    KickoffTakerOutcome::Missed => self.missed_count += 1,
                    _ => {}
                }
            }
        }
    }

    pub fn complete_labeled_event_counts(&self) -> LabeledCounts {
        LabeledCounts::complete_from_label_sets(
            &[
                &KICKOFF_OUTCOME_LABELS,
                &KICKOFF_TYPE_LABELS,
                &KICKOFF_DIRECTION_LABELS,
                &KICKOFF_WIN_STRENGTH_LABELS,
                &KICKOFF_POSSESSION_OUTCOME_LABELS,
                &KICKOFF_GOAL_LABELS,
                &KICKOFF_SETTLEMENT_LABELS,
            ],
            &self.labeled_event_counts,
        )
    }

    pub fn complete_labeled_player_counts(&self) -> LabeledCounts {
        LabeledCounts::complete_from_label_sets(
            &[
                &KICKOFF_SPAWN_LABELS,
                &KICKOFF_TAKER_OUTCOME_LABELS,
                &KICKOFF_APPROACH_LABELS,
                &KICKOFF_SUPPORT_BEHAVIOR_LABELS,
                &KICKOFF_BALL_DIRECTION_LABELS,
            ],
            &self.labeled_player_counts,
        )
    }

    pub fn for_team(&self, is_team_zero: bool) -> KickoffTeamStats {
        let wins = if is_team_zero {
            self.team_zero_wins
        } else {
            self.team_one_wins
        };
        let losses = if is_team_zero {
            self.team_one_wins
        } else {
            self.team_zero_wins
        };
        let kickoff_possessions = if is_team_zero {
            self.team_zero_kickoff_possessions
        } else {
            self.team_one_kickoff_possessions
        };
        let opponent_kickoff_possessions = if is_team_zero {
            self.team_one_kickoff_possessions
        } else {
            self.team_zero_kickoff_possessions
        };
        let kickoff_possession_advantages = if is_team_zero {
            self.team_zero_kickoff_possession_advantages
        } else {
            self.team_one_kickoff_possession_advantages
        };
        let opponent_kickoff_possession_advantages = if is_team_zero {
            self.team_one_kickoff_possession_advantages
        } else {
            self.team_zero_kickoff_possession_advantages
        };
        let kickoff_goals_for = if is_team_zero {
            self.team_zero_kickoff_goals
        } else {
            self.team_one_kickoff_goals
        };
        let kickoff_goals_against = if is_team_zero {
            self.team_one_kickoff_goals
        } else {
            self.team_zero_kickoff_goals
        };
        KickoffTeamStats {
            count: self.count,
            wins,
            losses,
            neutral_outcomes: self.neutral_outcomes,
            kickoff_possessions,
            opponent_kickoff_possessions,
            kickoff_possession_advantages,
            opponent_kickoff_possession_advantages,
            contested_kickoff_possessions: self.contested_kickoff_possessions,
            kickoff_goal_count: self.kickoff_goal_count,
            kickoff_goals_for,
            kickoff_goals_against,
            win_strength_sample_count: self.win_strength_sample_count,
            cumulative_win_strength: self.cumulative_win_strength,
            boost_after_sample_count: self.boost_after_sample_count,
            cumulative_boost_after: self.cumulative_boost_after,
            fake_count: self.fake_count,
            missed_count: self.missed_count,
        }
    }
}

impl KickoffPlayerStats {
    pub(crate) fn record_event(&mut self, event: &KickoffEvent, player: KickoffPlayerEventRef<'_>) {
        self.count += 1;
        self.labeled_event_counts.increment(player.labels());
        if let Some(taker) = player.as_taker() {
            match taker.outcome {
                KickoffTakerOutcome::Touched => self.touches += 1,
                KickoffTakerOutcome::Fake => self.fakes += 1,
                KickoffTakerOutcome::Missed => self.misses += 1,
                KickoffTakerOutcome::Unknown => {}
            }
        }
        if let Some(support) = player.as_support() {
            if support.first_touch_time.is_some() {
                self.touches += 1;
            }
            match support.support_behavior {
                KickoffSupportBehavior::GoForBoost => self.support_go_for_boosts += 1,
                KickoffSupportBehavior::Cheat => self.support_cheats += 1,
                KickoffSupportBehavior::Other => self.support_other += 1,
                KickoffSupportBehavior::Unknown => {}
            }
        }
        if event.kickoff_goal && event.scoring_team_is_team_0 == Some(player.is_team_0()) {
            self.kickoff_goal_count += 1;
        }
        if let Some(boost_after) = player.boost_after() {
            self.boost_after_sample_count += 1;
            self.cumulative_boost_after += boost_after;
        }
    }

    pub fn average_boost_after(&self) -> f32 {
        if self.boost_after_sample_count == 0 {
            0.0
        } else {
            self.cumulative_boost_after / self.boost_after_sample_count as f32
        }
    }
}

impl KickoffTeamStats {
    pub fn average_win_strength(&self) -> f32 {
        if self.win_strength_sample_count == 0 {
            0.0
        } else {
            self.cumulative_win_strength / self.win_strength_sample_count as f32
        }
    }

    pub fn average_boost_after(&self) -> f32 {
        if self.boost_after_sample_count == 0 {
            0.0
        } else {
            self.cumulative_boost_after / self.boost_after_sample_count as f32
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct KickoffStatsAccumulator {
    stats: KickoffStats,
    player_stats: HashMap<PlayerId, KickoffPlayerStats>,
}

impl KickoffStatsAccumulator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn stats(&self) -> &KickoffStats {
        &self.stats
    }

    pub fn player_stats(&self) -> &HashMap<PlayerId, KickoffPlayerStats> {
        &self.player_stats
    }

    pub fn apply_event(&mut self, event: &KickoffEvent) {
        self.stats.record_event(event);
        for player in event.player_events() {
            self.player_stats
                .entry(player.player().clone())
                .or_default()
                .record_event(event, player);
        }
    }
}
