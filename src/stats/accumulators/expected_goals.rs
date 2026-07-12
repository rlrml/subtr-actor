use super::*;

pub(crate) fn threat_metric_threat_added_label() -> StatLabel {
    StatLabel::new("metric", "threat_added")
}

pub(crate) fn threat_metric_xg_label() -> StatLabel {
    StatLabel::new("metric", "xg")
}

pub(crate) fn threat_team_label(is_team_0: bool) -> StatLabel {
    if is_team_0 {
        StatLabel::new("team", "team_zero")
    } else {
        StatLabel::new("team", "team_one")
    }
}

/// Per-player accumulated threat stats. The labeled sums are the canonical
/// record (`metric=threat_added` / `metric=xg`); the plain fields are
/// convenience projections kept in sync.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct ExpectedGoalsPlayerStats {
    /// Sum of positive detection-frame threat deltas (detection-frame V minus
    /// preceding-live-frame V, from the toucher's team's perspective) over the
    /// player's touches. This is an observed one-frame delta, not a causal
    /// estimate of each touch's multi-frame impulse.
    pub threat_added: f32,
    /// Sum of episode xG time integrals (`sum(V * dt) / tau` per episode)
    /// over episodes credited to this player.
    pub xg: f32,
    pub credited_episode_count: u32,
    pub credited_goal_episode_count: u32,
    #[serde(default, skip_serializing_if = "LabeledFloatSums::is_empty")]
    pub labeled_sums: LabeledFloatSums,
}

impl ExpectedGoalsPlayerStats {
    fn record_touch(&mut self, is_team_0: bool, positive_delta: f32) {
        self.labeled_sums.add(
            [
                threat_metric_threat_added_label(),
                threat_team_label(is_team_0),
            ],
            positive_delta,
        );
        self.sync_projections();
    }

    fn record_episode(&mut self, event: &ThreatEpisodeEvent) {
        self.labeled_sums.add(
            [
                threat_metric_xg_label(),
                threat_team_label(event.team_is_team_0),
            ],
            event.xg,
        );
        self.credited_episode_count += 1;
        if event.ended_in_goal {
            self.credited_goal_episode_count += 1;
        }
        self.sync_projections();
    }

    fn sync_projections(&mut self) {
        self.threat_added = self
            .labeled_sums
            .sum_matching(&[threat_metric_threat_added_label()]);
        self.xg = self.labeled_sums.sum_matching(&[threat_metric_xg_label()]);
    }
}

/// Per-team accumulated threat stats.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct ExpectedGoalsTeamStats {
    /// The team's full-match xG time integral (`sum(V * dt) / tau` over every
    /// evaluated live frame, sub-threshold frames included), fed from
    /// [`ExpectedGoalsCalculator::team_xg_integrals`]. NOT a sum of episode
    /// xG: per-player `xg` sums to LESS than this, because diffuse
    /// sub-threshold threat is not attributed to any player (empirically only
    /// ~62% of the integral falls inside above-threshold episodes).
    pub xg: f32,
    pub episode_count: u32,
    pub goal_episode_count: u32,
}

/// Accumulates threat/expected-goals stats over the replay from
/// [`ThreatTouchEvent`]s and [`ThreatEpisodeEvent`]s.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ExpectedGoalsStatsAccumulator {
    player_stats: HashMap<PlayerId, ExpectedGoalsPlayerStats>,
    team_stats: [ExpectedGoalsTeamStats; 2],
}

impl ExpectedGoalsStatsAccumulator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn player_stats(&self) -> &HashMap<PlayerId, ExpectedGoalsPlayerStats> {
        &self.player_stats
    }

    pub fn team_stats(&self, is_team_0: bool) -> &ExpectedGoalsTeamStats {
        &self.team_stats[usize::from(!is_team_0)]
    }

    /// Fold one detection-frame touch threat delta: only positive deltas count
    /// toward the toucher's threat-added sum. The delta is an observed
    /// one-frame state change, not a causal multi-frame impulse estimate.
    pub fn apply_touch_event(&mut self, event: &ThreatTouchEvent) {
        let delta = event.delta();
        if delta <= 0.0 {
            return;
        }
        let Some(player) = event.player.as_ref() else {
            return;
        };
        self.player_stats
            .entry(player.clone())
            .or_default()
            .record_touch(event.team_is_team_0, delta);
    }

    /// Fold one closed threat episode: its xG (the within-episode time
    /// integral) is credited to the episode's player when one is known, and
    /// the team's episode counters advance. Team `xg` is NOT summed from
    /// episodes -- it is the full-match integral set through
    /// [`Self::set_team_xg_integrals`].
    pub fn apply_episode_event(&mut self, event: &ThreatEpisodeEvent) {
        if let Some(player) = event.credited_player.as_ref() {
            self.player_stats
                .entry(player.clone())
                .or_default()
                .record_episode(event);
        }
        let team = &mut self.team_stats[usize::from(!event.team_is_team_0)];
        team.episode_count += 1;
        if event.ended_in_goal {
            team.goal_episode_count += 1;
        }
    }

    /// Overwrite both teams' accumulated xG with the calculator's full-match
    /// integrals (`[team zero, team one]`). Called with the current absolute
    /// totals each projection step, so it is idempotent per frame.
    pub fn set_team_xg_integrals(&mut self, integrals: [f64; 2]) {
        self.team_stats[0].xg = integrals[0] as f32;
        self.team_stats[1].xg = integrals[1] as f32;
    }
}
