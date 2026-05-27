use super::*;

impl<K> ContinuousBallControlTracker<K>
where
    K: Copy + PartialEq,
{
    pub(crate) fn begin_sequence(
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

    pub(crate) fn extend_sequence(
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

    pub(crate) fn complete_sequence(
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
}
