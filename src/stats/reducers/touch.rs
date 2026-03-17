use super::*;

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct TouchStats {
    pub touch_count: u32,
    pub is_last_touch: bool,
    pub last_touch_time: Option<f32>,
    pub last_touch_frame: Option<usize>,
    pub time_since_last_touch: Option<f32>,
    pub frames_since_last_touch: Option<usize>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct TouchReducer {
    player_stats: HashMap<PlayerId, TouchStats>,
    current_last_touch_player: Option<PlayerId>,
    live_play_tracker: LivePlayTracker,
}

impl TouchReducer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn player_stats(&self) -> &HashMap<PlayerId, TouchStats> {
        &self.player_stats
    }
}

impl StatsReducer for TouchReducer {
    fn on_sample(&mut self, sample: &StatsSample) -> SubtrActorResult<()> {
        if !self.live_play_tracker.is_live_play(sample) {
            return Ok(());
        }

        for stats in self.player_stats.values_mut() {
            stats.is_last_touch = false;
            stats.time_since_last_touch = stats
                .last_touch_time
                .map(|time| (sample.time - time).max(0.0));
            stats.frames_since_last_touch = stats
                .last_touch_frame
                .map(|frame| sample.frame_number.saturating_sub(frame));
        }

        for touch_event in &sample.touch_events {
            let Some(player_id) = touch_event.player.as_ref() else {
                continue;
            };
            let stats = self.player_stats.entry(player_id.clone()).or_default();
            stats.touch_count += 1;
            stats.last_touch_time = Some(touch_event.time);
            stats.last_touch_frame = Some(touch_event.frame);
            stats.time_since_last_touch = Some((sample.time - touch_event.time).max(0.0));
            stats.frames_since_last_touch =
                Some(sample.frame_number.saturating_sub(touch_event.frame));
        }

        if let Some(last_touch) = sample.touch_events.last() {
            self.current_last_touch_player = last_touch.player.clone();
        }

        if let Some(player_id) = self.current_last_touch_player.as_ref() {
            if let Some(stats) = self.player_stats.get_mut(player_id) {
                stats.is_last_touch = true;
            }
        }

        Ok(())
    }

    fn on_sample_with_context(
        &mut self,
        sample: &StatsSample,
        ctx: &AnalysisContext,
    ) -> SubtrActorResult<()> {
        if !self.live_play_tracker.is_live_play(sample) {
            return Ok(());
        }

        let touch_state = ctx
            .get::<TouchState>(TOUCH_STATE_SIGNAL_ID)
            .cloned()
            .unwrap_or_default();

        for stats in self.player_stats.values_mut() {
            stats.is_last_touch = false;
            stats.time_since_last_touch = stats
                .last_touch_time
                .map(|time| (sample.time - time).max(0.0));
            stats.frames_since_last_touch = stats
                .last_touch_frame
                .map(|frame| sample.frame_number.saturating_sub(frame));
        }

        for touch_event in &touch_state.touch_events {
            let Some(player_id) = touch_event.player.as_ref() else {
                continue;
            };
            let stats = self.player_stats.entry(player_id.clone()).or_default();
            stats.touch_count += 1;
            stats.last_touch_time = Some(touch_event.time);
            stats.last_touch_frame = Some(touch_event.frame);
            stats.time_since_last_touch = Some((sample.time - touch_event.time).max(0.0));
            stats.frames_since_last_touch =
                Some(sample.frame_number.saturating_sub(touch_event.frame));
        }

        if let Some(player_id) = touch_state.last_touch_player.as_ref() {
            if let Some(stats) = self.player_stats.get_mut(player_id) {
                stats.is_last_touch = true;
            }
        }

        Ok(())
    }
}
