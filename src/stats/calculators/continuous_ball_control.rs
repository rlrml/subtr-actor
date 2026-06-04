use super::*;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ContinuousBallControlState {
    pub completed_sequences: Vec<CompletedBallControlSequence<BallCarryKind>>,
}

#[derive(Debug, Clone, Copy)]
pub struct ContinuousBallControlSample<K> {
    pub kind: K,
    pub player_position: glam::Vec3,
    pub horizontal_gap: f32,
    pub vertical_gap: f32,
    pub speed: f32,
}

#[derive(Debug, Clone)]
pub struct ContinuousBallControlCandidate<K> {
    pub player_id: PlayerId,
    pub is_team_0: bool,
    pub touch_count: u32,
    pub air_touch_count: u32,
    pub sample: ContinuousBallControlSample<K>,
}

#[derive(Debug, Clone)]
pub struct ContinuousBallControlPlayerStatus {
    pub player_id: PlayerId,
    pub is_airborne: bool,
}

#[derive(Debug, Clone)]
pub struct ContinuousBallControlTouch {
    pub player_id: PlayerId,
    pub is_airborne: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CompletedBallControlSequence<K> {
    pub player_id: PlayerId,
    pub is_team_0: bool,
    pub kind: K,
    pub start_frame: usize,
    pub end_frame: usize,
    pub start_time: f32,
    pub end_time: f32,
    pub duration: f32,
    pub straight_line_distance: f32,
    pub path_distance: f32,
    pub average_horizontal_gap: f32,
    pub average_vertical_gap: f32,
    pub average_speed: f32,
    pub start_position: glam::Vec3,
    pub end_position: glam::Vec3,
    pub touch_count: u32,
    pub air_touch_count: u32,
}

#[derive(Debug, Clone)]
struct ActiveBallControlSequence<K> {
    player_id: PlayerId,
    is_team_0: bool,
    kind: K,
    start_frame: usize,
    last_frame: usize,
    start_time: f32,
    last_time: f32,
    start_position: glam::Vec3,
    last_position: glam::Vec3,
    duration: f32,
    path_distance: f32,
    horizontal_gap_integral: f32,
    vertical_gap_integral: f32,
    speed_integral: f32,
    touch_count: u32,
    air_touch_count: u32,
}

#[derive(Debug, Clone)]
pub struct ContinuousBallControlTracker<K> {
    active_sequence: Option<ActiveBallControlSequence<K>>,
    pending_takeoff_touches: HashMap<PlayerId, u32>,
}

impl<K> Default for ContinuousBallControlTracker<K> {
    fn default() -> Self {
        Self {
            active_sequence: None,
            pending_takeoff_touches: HashMap::new(),
        }
    }
}

impl<K> ContinuousBallControlTracker<K>
where
    K: Copy + PartialEq,
{
    fn begin_sequence(
        frame: &FrameInfo,
        candidate: ContinuousBallControlCandidate<K>,
        takeoff_touch_count: u32,
    ) -> ActiveBallControlSequence<K> {
        let sample = candidate.sample;
        ActiveBallControlSequence {
            player_id: candidate.player_id,
            is_team_0: candidate.is_team_0,
            kind: sample.kind,
            start_frame: frame.frame_number.saturating_sub(1),
            last_frame: frame.frame_number,
            start_time: (frame.time - frame.dt).max(0.0),
            last_time: frame.time,
            start_position: sample.player_position,
            last_position: sample.player_position,
            duration: frame.dt,
            path_distance: 0.0,
            horizontal_gap_integral: sample.horizontal_gap * frame.dt,
            vertical_gap_integral: sample.vertical_gap * frame.dt,
            speed_integral: sample.speed * frame.dt,
            touch_count: candidate.touch_count + takeoff_touch_count,
            air_touch_count: candidate.air_touch_count,
        }
    }

    fn extend_sequence(
        active_sequence: &mut ActiveBallControlSequence<K>,
        frame: &FrameInfo,
        sample: ContinuousBallControlSample<K>,
        touch_count: u32,
        air_touch_count: u32,
    ) {
        active_sequence.duration += frame.dt;
        active_sequence.path_distance += sample
            .player_position
            .distance(active_sequence.last_position);
        active_sequence.last_position = sample.player_position;
        active_sequence.last_time = frame.time;
        active_sequence.last_frame = frame.frame_number;
        active_sequence.horizontal_gap_integral += sample.horizontal_gap * frame.dt;
        active_sequence.vertical_gap_integral += sample.vertical_gap * frame.dt;
        active_sequence.speed_integral += sample.speed * frame.dt;
        active_sequence.touch_count += touch_count;
        active_sequence.air_touch_count += air_touch_count;
    }

    fn complete_sequence(
        active_sequence: ActiveBallControlSequence<K>,
    ) -> CompletedBallControlSequence<K> {
        CompletedBallControlSequence {
            player_id: active_sequence.player_id,
            is_team_0: active_sequence.is_team_0,
            kind: active_sequence.kind,
            start_frame: active_sequence.start_frame,
            end_frame: active_sequence.last_frame,
            start_time: active_sequence.start_time,
            end_time: active_sequence.last_time,
            duration: active_sequence.duration,
            straight_line_distance: active_sequence
                .start_position
                .truncate()
                .distance(active_sequence.last_position.truncate()),
            path_distance: active_sequence.path_distance,
            average_horizontal_gap: active_sequence.horizontal_gap_integral
                / active_sequence.duration,
            average_vertical_gap: active_sequence.vertical_gap_integral / active_sequence.duration,
            average_speed: active_sequence.speed_integral / active_sequence.duration,
            start_position: active_sequence.start_position,
            end_position: active_sequence.last_position,
            touch_count: active_sequence.touch_count,
            air_touch_count: active_sequence.air_touch_count,
        }
    }

    fn track_touch_contacts(&mut self, touches: &[ContinuousBallControlTouch]) {
        if touches.is_empty() {
            return;
        }

        let touched_players = touches
            .iter()
            .map(|touch| touch.player_id.clone())
            .collect::<HashSet<_>>();
        self.pending_takeoff_touches
            .retain(|player_id, _| touched_players.contains(player_id));

        for touch in touches {
            if !touch.is_airborne {
                *self
                    .pending_takeoff_touches
                    .entry(touch.player_id.clone())
                    .or_default() += 1;
            }
        }
    }

    fn active_player_is_non_airborne<G>(
        &self,
        player_statuses: &[ContinuousBallControlPlayerStatus],
        requires_airborne_for_kind: G,
    ) -> bool
    where
        G: Fn(K) -> bool,
    {
        self.active_sequence
            .as_ref()
            .is_some_and(|active_sequence| {
                requires_airborne_for_kind(active_sequence.kind)
                    && player_statuses
                        .iter()
                        .find(|status| status.player_id == active_sequence.player_id)
                        .is_some_and(|status| !status.is_airborne)
            })
    }

    fn finish_active_sequence<F>(
        &mut self,
        min_duration_for_kind: F,
    ) -> Option<CompletedBallControlSequence<K>>
    where
        F: Fn(K) -> f32,
    {
        let active_sequence = self.active_sequence.take()?;
        if active_sequence.duration < min_duration_for_kind(active_sequence.kind) {
            return None;
        }
        Some(Self::complete_sequence(active_sequence))
    }

    pub fn update<F, G>(
        &mut self,
        frame: &FrameInfo,
        candidate: Option<ContinuousBallControlCandidate<K>>,
        player_statuses: &[ContinuousBallControlPlayerStatus],
        touches: &[ContinuousBallControlTouch],
        min_duration_for_kind: F,
        requires_airborne_for_kind: G,
    ) -> Vec<CompletedBallControlSequence<K>>
    where
        F: Fn(K) -> f32 + Copy,
        G: Fn(K) -> bool + Copy,
    {
        let mut completed = Vec::new();
        self.track_touch_contacts(touches);

        if self.active_player_is_non_airborne(player_statuses, requires_airborne_for_kind) {
            if let Some(sequence) = self.finish_active_sequence(min_duration_for_kind) {
                completed.push(sequence);
            }
        }

        let Some(candidate) = candidate else {
            if let Some(sequence) = self.finish_active_sequence(min_duration_for_kind) {
                completed.push(sequence);
            }
            return completed;
        };

        let same_sequence = self
            .active_sequence
            .as_ref()
            .is_some_and(|active_sequence| {
                active_sequence.player_id == candidate.player_id
                    && active_sequence.kind == candidate.sample.kind
            });

        if same_sequence {
            if let Some(active_sequence) = self.active_sequence.as_mut() {
                Self::extend_sequence(
                    active_sequence,
                    frame,
                    candidate.sample,
                    candidate.touch_count,
                    candidate.air_touch_count,
                );
            }
        } else {
            if let Some(sequence) = self.finish_active_sequence(min_duration_for_kind) {
                completed.push(sequence);
            }
            let takeoff_touch_count = if requires_airborne_for_kind(candidate.sample.kind) {
                self.pending_takeoff_touches
                    .remove(&candidate.player_id)
                    .unwrap_or(0)
            } else {
                0
            };
            self.active_sequence =
                Some(Self::begin_sequence(frame, candidate, takeoff_touch_count));
        }

        completed
    }

    pub fn finish<F>(&mut self, min_duration_for_kind: F) -> Option<CompletedBallControlSequence<K>>
    where
        F: Fn(K) -> f32,
    {
        self.finish_active_sequence(min_duration_for_kind)
    }
}

#[cfg(test)]
#[path = "continuous_ball_control_tests.rs"]
mod tests;
