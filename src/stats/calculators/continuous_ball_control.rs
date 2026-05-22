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
    pub sample: ContinuousBallControlSample<K>,
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
}

#[derive(Debug, Clone)]
pub struct ContinuousBallControlTracker<K> {
    active_sequence: Option<ActiveBallControlSequence<K>>,
}

impl<K> Default for ContinuousBallControlTracker<K> {
    fn default() -> Self {
        Self {
            active_sequence: None,
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
        }
    }

    fn extend_sequence(
        active_sequence: &mut ActiveBallControlSequence<K>,
        frame: &FrameInfo,
        sample: ContinuousBallControlSample<K>,
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
        }
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

    pub fn update<F>(
        &mut self,
        frame: &FrameInfo,
        candidate: Option<ContinuousBallControlCandidate<K>>,
        min_duration_for_kind: F,
    ) -> Vec<CompletedBallControlSequence<K>>
    where
        F: Fn(K) -> f32 + Copy,
    {
        let mut completed = Vec::new();
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
                Self::extend_sequence(active_sequence, frame, candidate.sample);
            }
        } else {
            if let Some(sequence) = self.finish_active_sequence(min_duration_for_kind) {
                completed.push(sequence);
            }
            self.active_sequence = Some(Self::begin_sequence(frame, candidate));
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
