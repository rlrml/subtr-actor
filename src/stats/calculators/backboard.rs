use super::*;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct BackboardPlayerStats {
    pub count: u32,
    pub is_last_backboard: bool,
    pub last_backboard_time: Option<f32>,
    pub last_backboard_frame: Option<usize>,
    pub time_since_last_backboard: Option<f32>,
    pub frames_since_last_backboard: Option<usize>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct BackboardTeamStats {
    pub count: u32,
}

#[derive(Debug, Clone, Default)]
pub struct BackboardCalculator {
    player_stats: HashMap<PlayerId, BackboardPlayerStats>,
    team_zero_stats: BackboardTeamStats,
    team_one_stats: BackboardTeamStats,
    events: Vec<BackboardBounceEvent>,
    current_last_backboard_player: Option<PlayerId>,
}

impl BackboardCalculator {
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

    fn begin_sample(&mut self, frame: &FrameInfo) {
        for stats in self.player_stats.values_mut() {
            stats.is_last_backboard = false;
            stats.time_since_last_backboard = stats
                .last_backboard_time
                .map(|time| (frame.time - time).max(0.0));
            stats.frames_since_last_backboard = stats
                .last_backboard_frame
                .map(|last_frame| frame.frame_number.saturating_sub(last_frame));
        }
    }

    fn apply_events(&mut self, frame: &FrameInfo, events: &[BackboardBounceEvent]) {
        for event in events {
            let stats = self.player_stats.entry(event.player.clone()).or_default();
            stats.count += 1;
            stats.last_backboard_time = Some(event.time);
            stats.last_backboard_frame = Some(event.frame);
            stats.time_since_last_backboard = Some((frame.time - event.time).max(0.0));
            stats.frames_since_last_backboard =
                Some(frame.frame_number.saturating_sub(event.frame));

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

    pub fn update(
        &mut self,
        frame: &FrameInfo,
        backboard_bounce_state: &BackboardBounceState,
    ) -> SubtrActorResult<()> {
        self.begin_sample(frame);
        self.apply_events(frame, &backboard_bounce_state.bounce_events);
        Ok(())
    }
}
