use super::*;

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct BackboardPlayerStats {
    pub count: u32,
    pub is_last_backboard: bool,
    pub last_backboard_time: Option<f32>,
    pub last_backboard_frame: Option<usize>,
    pub time_since_last_backboard: Option<f32>,
    pub frames_since_last_backboard: Option<usize>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct BackboardTeamStats {
    pub count: u32,
}

#[derive(Debug, Clone, Default)]
pub struct BackboardReducer {
    player_stats: HashMap<PlayerId, BackboardPlayerStats>,
    team_zero_stats: BackboardTeamStats,
    team_one_stats: BackboardTeamStats,
    events: Vec<BackboardBounceEvent>,
    current_last_backboard_player: Option<PlayerId>,
}

impl BackboardReducer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn player_stats(&self) -> &HashMap<PlayerId, BackboardPlayerStats> {
        &self.player_stats
    }

    pub fn team_zero_stats(&self) -> &BackboardTeamStats {
        &self.team_zero_stats
    }

    pub fn team_one_stats(&self) -> &BackboardTeamStats {
        &self.team_one_stats
    }

    pub fn events(&self) -> &[BackboardBounceEvent] {
        &self.events
    }

    fn begin_sample(&mut self, sample: &StatsSample) {
        for stats in self.player_stats.values_mut() {
            stats.is_last_backboard = false;
            stats.time_since_last_backboard = stats
                .last_backboard_time
                .map(|time| (sample.time - time).max(0.0));
            stats.frames_since_last_backboard = stats
                .last_backboard_frame
                .map(|frame| sample.frame_number.saturating_sub(frame));
        }
    }

    fn apply_events(&mut self, sample: &StatsSample, events: &[BackboardBounceEvent]) {
        for event in events {
            let stats = self.player_stats.entry(event.player.clone()).or_default();
            stats.count += 1;
            stats.last_backboard_time = Some(event.time);
            stats.last_backboard_frame = Some(event.frame);
            stats.time_since_last_backboard = Some((sample.time - event.time).max(0.0));
            stats.frames_since_last_backboard =
                Some(sample.frame_number.saturating_sub(event.frame));

            let team_stats = if event.is_team_0 {
                &mut self.team_zero_stats
            } else {
                &mut self.team_one_stats
            };
            team_stats.count += 1;
            self.events.push(event.clone());
        }

        if let Some(last_event) = events.last() {
            self.current_last_backboard_player = Some(last_event.player.clone());
        }

        if let Some(player_id) = self.current_last_backboard_player.as_ref() {
            if let Some(stats) = self.player_stats.get_mut(player_id) {
                stats.is_last_backboard = true;
            }
        }
    }
}

impl StatsReducer for BackboardReducer {
    fn required_derived_signals(&self) -> Vec<DerivedSignalId> {
        vec![BACKBOARD_BOUNCE_STATE_SIGNAL_ID]
    }

    fn on_sample_with_context(
        &mut self,
        sample: &StatsSample,
        ctx: &AnalysisContext,
    ) -> SubtrActorResult<()> {
        self.begin_sample(sample);
        let state = ctx
            .get::<BackboardBounceState>(BACKBOARD_BOUNCE_STATE_SIGNAL_ID)
            .cloned()
            .unwrap_or_default();
        self.apply_events(sample, &state.bounce_events);
        Ok(())
    }
}
